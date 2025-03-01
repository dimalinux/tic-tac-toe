use std::thread;

use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    account::ReadableAccount,
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

use crate::{transaction::send_transaction_and_print_logs, util, util::get_anchor_discriminator};
//type Tile = (usize, usize); // (x, y) coordinates for a play

// Todo: remove BorshSerialize
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Copy, Clone)]
enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}

const ACTIVE_STATE: GameState = GameState::Active;
//const TIE_STATE: GameState = GameState::Tie;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    X,
    O,
}

type Board = [[Option<Sign>; 3]; 3];

static ACCOUNT_GAME_DISCRIMINATOR: Lazy<[u8; 8]> =
    Lazy::new(|| get_anchor_discriminator("account:Game"));

#[derive(BorshSerialize, BorshDeserialize, Copy, Clone)]
pub struct GameAccount {
    players: [Pubkey; 2],          // (32 * 2)
    turn: u8,                      // 1
    board: [[Option<Sign>; 3]; 3], // 9 * (1 + 1) = 18
    state: GameState,              // 32 + 1
}

pub struct Game {
    program_id: Pubkey,
    rpc_client_url: String,
    print_balances: bool,
    game_keypair: Keypair,
    player_one: Keypair,
    player_two: Keypair,
    turn_number: usize,
    expected_board: Board,
}

impl Game {
    pub fn new(
        program_id: Pubkey,
        rpc_client_url: String,
        player_one: Keypair,
        player_two: Keypair,
    ) -> Self {
        Self {
            program_id,
            rpc_client_url,
            print_balances: true,
            game_keypair: Keypair::new(),
            player_one,
            player_two,
            turn_number: 1,
            expected_board: Board::default(),
        }
    }

    pub fn game_id(&self) -> Pubkey {
        self.game_keypair.pubkey()
    }

    fn rpc_client(&self, config: CommitmentConfig) -> RpcClient {
        RpcClient::new_with_commitment(&self.rpc_client_url, config)
    }

    pub fn print_balance(&self, name: &str, public_key: &Pubkey) {
        util::print_balance(
            &self.rpc_client(CommitmentConfig::processed()),
            name,
            public_key,
        );
    }

    pub fn get_game_state(&mut self) -> GameAccount {
        let rpc_client =
            RpcClient::new_with_commitment(&self.rpc_client_url, CommitmentConfig::processed());
        let game_state = rpc_client.get_account(&self.game_keypair.pubkey()).unwrap();
        let account_data = game_state.data();
        assert!(account_data.len() > 8);
        let set_discriminator = &account_data[0..8];
        let mut game_state = &account_data[8..];
        assert_eq!(set_discriminator, *ACCOUNT_GAME_DISCRIMINATOR);
        GameAccount::deserialize(&mut game_state).unwrap()
    }

    pub fn setup_game(&mut self) {
        self.turn_number = 1;

        if self.print_balances {
            self.print_balance("player one at start", &self.player_one.pubkey());
            self.print_balance("player two at start", &self.player_two.pubkey());
        }

        let player_two_pub: Pubkey = self.player_two.pubkey();

        let mut instruction_data = get_anchor_discriminator("global:setup_game").to_vec();
        player_two_pub.serialize(&mut instruction_data).unwrap();

        let setup_game_instruction = Instruction::new_with_bytes(
            self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(self.game_keypair.pubkey(), true),
                AccountMeta::new(self.player_one.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let rpc_client = self.rpc_client(CommitmentConfig::finalized());
        let recent_block_hash = rpc_client.get_latest_blockhash().unwrap();
        let payer = &self.player_one;

        let mut transaction =
            Transaction::new_with_payer(&[setup_game_instruction], Some(&payer.pubkey()));
        transaction.sign(&[payer, &self.game_keypair], recent_block_hash);

        match send_transaction_and_print_logs(&self.rpc_client_url, &transaction) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error setting up game: {:?}", e);
                std::process::exit(1);
            }
        }

        self.expected_board = [[None; 3]; 3];

        thread::sleep(std::time::Duration::from_secs(10));
        let game_state = self.get_game_state();
        assert_eq!(game_state.turn, 1);
        assert_eq!(game_state.players[0], self.player_one.pubkey());
        assert_eq!(game_state.players[1], self.player_two.pubkey());
        assert_eq!(game_state.state, ACTIVE_STATE);
        assert_eq!(game_state.board, self.expected_board);

        if self.print_balances {
            self.print_balance("game after setup", &self.game_keypair.pubkey());
            self.print_balance("player one after setup", &self.player_one.pubkey());
            self.print_balance("player two after setup", &self.player_two.pubkey());
        }
    }
}
