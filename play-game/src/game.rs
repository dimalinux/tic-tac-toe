use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

use crate::{
    transaction::send_transaction_and_print_logs, util, util::get_anchor_instruction_bytes,
};

//type Tile = (usize, usize); // (x, y) coordinates for a play

#[derive(BorshDeserialize, Debug, PartialEq, Copy, Clone)]
enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}

const ACTIVE_STATE: GameState = GameState::Active;
//const TIE_STATE: GameState = GameState::Tie;

#[derive(BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    X,
    O,
    None,
}

type Board = [[Option<Sign>; 3]; 3];

#[derive(BorshDeserialize, Copy, Clone)]
pub struct GameAccount {
    players: [Pubkey; 2],          // (32 * 2)
    turn: u8,                      // 1
    board: [[Option<Sign>; 3]; 3], // 9 * (1 + 1) = 18
    state: GameState,              // 32 + 1
}

pub struct Game {
    program_id: Pubkey,
    rpc_client: RpcClient,
    print_balances: bool,
    game_keypair: Keypair,
    player_one: Keypair,
    player_two: Keypair,
    turn_number: usize,
    expected_board: Board,
}

impl Game {
    pub fn new(program_id: Pubkey, rpc_client: RpcClient, player_one: Keypair) -> Self {
        Self {
            program_id,
            rpc_client,
            print_balances: true,
            game_keypair: Keypair::new(),
            player_one,
            player_two: Keypair::new(),
            turn_number: 1,
            expected_board: Board::default(),
        }
    }

    pub fn print_balance(&self, name: &str, public_key: &Pubkey) {
        util::print_balance(&self.rpc_client, name, public_key);
    }

    pub fn get_game_state(&mut self) -> GameAccount {
        let game_state = self
            .rpc_client
            .get_account(&self.game_keypair.pubkey())
            .unwrap();
        GameAccount::try_from_slice(&game_state.data).unwrap()
    }

    pub fn setup_game(&mut self) {
        self.turn_number = 1;
        let payer = &self.player_one;

        if self.print_balances {
            self.print_balance("game key at start", &self.game_keypair.pubkey());
            self.print_balance("player one at start", &self.player_one.pubkey());
            self.print_balance("player two at start", &self.player_two.pubkey());
        }

        let player_two_pub: Pubkey = self.player_two.pubkey();

        let mut instruction_data = get_anchor_instruction_bytes("global:setup_game");
        player_two_pub.serialize(&mut instruction_data).unwrap();

        let setup_game_instruction = Instruction::new_with_bytes(
            self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(self.game_keypair.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(self.player_two.pubkey(), false),
            ],
        );

        let recent_block_hash = self.rpc_client.get_latest_blockhash().unwrap();
        let payer = &self.player_one;

        let mut transaction =
            Transaction::new_with_payer(&[setup_game_instruction], Some(&payer.pubkey()));
        transaction.sign(&[payer, &self.game_keypair], recent_block_hash);

        send_transaction_and_print_logs(&self.rpc_client, &transaction).unwrap();

        self.expected_board = [[Some(Sign::None); 3]; 3];

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
