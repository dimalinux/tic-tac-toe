mod game;
mod tests;
mod transaction;
mod util;

use solana_client::rpc_client::RpcClient;
use solana_program::native_token;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair, signer::Signer};
use util::{get_payer_key, get_program_id};

use crate::util::get_rpc_url;

fn main() {
    let program_id = get_program_id();
    let payer = get_payer_key();
    let rpc_client = RpcClient::new_with_commitment(get_rpc_url(), CommitmentConfig::processed());

    // Fund the players
    let player_one = Keypair::new();
    let player_two = Keypair::new();
    let lamports = native_token::sol_to_lamports(0.01);
    transaction::transfer(&rpc_client, lamports, &payer, &player_one.pubkey());
    transaction::transfer(&rpc_client, lamports, &payer, &player_two.pubkey());

    tests::play_player_one_wins_game(&program_id, &rpc_client, &player_one, &player_two);
    tests::tie_game(&program_id, &rpc_client, &player_one, &player_two);

    // Sweep funds back from temporary accounts before they disappear
    transaction::sweep(&rpc_client, &player_one, &payer.pubkey());
    transaction::sweep(&rpc_client, &player_two, &payer.pubkey());
}
