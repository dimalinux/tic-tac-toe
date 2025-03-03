use borsh::{BorshDeserialize, BorshSerialize};
use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    account::ReadableAccount,
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

use crate::{transaction::send_transaction_and_print_logs, util, util::get_anchor_discriminator};

type Tile = (u8, u8); // (x, y) coordinates for a play

#[derive(BorshDeserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}

#[derive(BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    X,
    O,
}

type Board = [[Option<Sign>; 3]; 3];

static ACCOUNT_GAME_DISCRIMINATOR: Lazy<[u8; 8]> =
    Lazy::new(|| get_anchor_discriminator("account:Game"));

#[derive(BorshDeserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameAccount {
    pub players: [Pubkey; 2], // (32 * 2)
    pub turn: u8,             // 1
    pub board: Board,         // 9 * (1 + 1) = 18
    pub state: GameState,     // 32 + 1
}

pub struct Game<'a> {
    pub program_id: &'a Pubkey,
    pub rpc_client: &'a RpcClient,
    pub print_balances: bool,
    pub game_keypair: Keypair,
    pub player_one: &'a Keypair,
    pub player_two: &'a Keypair,
}

impl<'a> Game<'a> {
    pub fn new(
        program_id: &'a Pubkey,
        rpc_client: &'a RpcClient,
        player_one: &'a Keypair,
        player_two: &'a Keypair,
    ) -> Self {
        Self {
            program_id,
            rpc_client,
            print_balances: true,
            game_keypair: Keypair::new(),
            player_one,
            player_two,
        }
    }

    pub fn game_id(&self) -> Pubkey {
        self.game_keypair.pubkey()
    }

    pub fn players(&self) -> [Pubkey; 2] {
        [self.player_one.pubkey(), self.player_two.pubkey()]
    }

    pub fn print_balance(&self, name: &str, public_key: &Pubkey) {
        util::print_balance(self.rpc_client, name, public_key);
    }

    pub fn get_game_account(&self) -> GameAccount {
        let game_state = self.rpc_client.get_account(&self.game_id()).unwrap();
        let account_data = game_state.data();
        assert!(account_data.len() > 8);
        let set_discriminator = &account_data[0..8];
        let mut game_state = &account_data[8..];
        assert_eq!(set_discriminator, *ACCOUNT_GAME_DISCRIMINATOR);
        GameAccount::deserialize(&mut game_state).unwrap()
    }

    pub fn setup_game(&self) {
        if self.print_balances {
            self.print_balance("player one at start", &self.player_one.pubkey());
            self.print_balance("player two at start", &self.player_two.pubkey());
        }

        let player_two_pub: Pubkey = self.player_two.pubkey();

        let mut instruction_data = get_anchor_discriminator("global:setup_game").to_vec();
        player_two_pub.serialize(&mut instruction_data).unwrap();

        let setup_game_instruction = Instruction::new_with_bytes(
            *self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(self.game_id(), true),
                AccountMeta::new(self.player_one.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let recent_block_hash = self.rpc_client.get_latest_blockhash().unwrap();
        let payer = &self.player_one;

        let mut transaction =
            Transaction::new_with_payer(&[setup_game_instruction], Some(&payer.pubkey()));
        transaction.sign(&[payer, &self.game_keypair], recent_block_hash);

        match send_transaction_and_print_logs(self.rpc_client, &transaction) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error setting up game: {:?}", e);
                std::process::exit(1);
            }
        }

        let game_state = self.get_game_account();
        assert_eq!(game_state.turn, 1);
        assert_eq!(game_state.players[0], self.player_one.pubkey());
        assert_eq!(game_state.players[1], self.player_two.pubkey());
        assert_eq!(game_state.state, GameState::Active);
        assert_eq!(game_state.board, [[None; 3]; 3]);

        if self.print_balances {
            self.print_balance("game after setup", &self.game_id());
            self.print_balance("player one after setup", &self.player_one.pubkey());
            self.print_balance("player two after setup", &self.player_two.pubkey());
        }
    }

    pub fn play(&mut self, tile: Tile) -> GameAccount {
        let is_player_one = self.get_game_account().turn % 2 == 1;
        let player_pub_key = if is_player_one {
            self.player_one.pubkey()
        } else {
            self.player_two.pubkey()
        };

        if self.print_balances {
            self.print_balance("before play", &player_pub_key);
        }

        let mut instruction_data = get_anchor_discriminator("global:play").to_vec();
        tile.serialize(&mut instruction_data).unwrap();

        let play_instruction = Instruction::new_with_bytes(
            *self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(self.game_id(), false),
                AccountMeta::new(player_pub_key, true),
            ],
        );

        let recent_block_hash = self.rpc_client.get_latest_blockhash().unwrap();

        let transaction = if is_player_one {
            Transaction::new_signed_with_payer(
                &[play_instruction],
                Some(&self.player_one.pubkey()),
                &vec![&self.player_one],
                recent_block_hash,
            )
        } else {
            Transaction::new_signed_with_payer(
                &[play_instruction],
                None,
                &vec![&self.player_two],
                recent_block_hash,
            )
        };

        match send_transaction_and_print_logs(self.rpc_client, &transaction) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error in play: {:?}", e);
                std::process::exit(1);
            }
        }

        if self.print_balances {
            self.print_balance("game after play", &self.game_id());
            self.print_balance("player one after play", &self.player_one.pubkey());
            self.print_balance("player two after play", &self.player_two.pubkey());
        }

        self.get_game_account()
    }
}
