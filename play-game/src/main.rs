#![allow(unexpected_cfgs)]

mod game;
mod transaction;
mod util;

use game::Game;
use solana_sdk::native_token;
use solana_sdk::signature::{Keypair, Signer};
use util::{get_payer_key, get_program_id, get_rpc_client};

use crate::util::print_balance;

fn main() {
    let program_id = get_program_id();
    let rpc_client = get_rpc_client();
    let payer = get_payer_key();

    let player_one = Keypair::new();
    let lamports = native_token::sol_to_lamports(1.0);
    transaction::transfer(&rpc_client, lamports, &payer, &player_one.pubkey());
    print_balance(&rpc_client, "player one", &player_one.pubkey());

    let mut game = Game::new(program_id, rpc_client, player_one);
    game.setup_game();
}
