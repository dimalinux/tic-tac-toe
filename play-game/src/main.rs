#![allow(unexpected_cfgs)]

mod game;
mod transaction;
mod util;

use game::Game;
use solana_program::system_program;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token,
    signature::{Keypair, Signer},
};
use util::{get_payer_key, get_program_id, get_rpc_client};

use crate::util::{get_rpc_url, print_balance};

fn main() {
    let program_id = get_program_id();
    let rpc_client = get_rpc_client(CommitmentConfig::finalized());
    let payer = get_payer_key();

    print_balance(&rpc_client, "payer", &payer.pubkey());

    assert_eq!(
        rpc_client.get_account(&payer.pubkey()).unwrap().owner,
        system_program::id()
    );

    let player_one = Keypair::new();
    let player_two = Keypair::new();

    let lamports = native_token::sol_to_lamports(1.0);
    transaction::transfer(&rpc_client, lamports, &payer, &player_one.pubkey());
    transaction::transfer(&rpc_client, lamports, &payer, &player_two.pubkey());

    print_balance(&rpc_client, "payer", &payer.pubkey());
    print_balance(&rpc_client, "player one", &player_one.pubkey());
    print_balance(&rpc_client, "player two", &player_two.pubkey());

    let mut game = Game::new(program_id, get_rpc_url(), player_one, player_two);
    assert_eq!(
        rpc_client
            .get_account(&get_payer_key().pubkey())
            .unwrap()
            .owner,
        system_program::id()
    );
    println!("Game ID: {}", game.game_id());
    game.setup_game();
    assert_eq!(
        rpc_client
            .get_account(&get_payer_key().pubkey())
            .unwrap()
            .owner,
        system_program::id()
    );
}
