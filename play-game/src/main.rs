#![allow(unexpected_cfgs)]

use std::fs;

use borsh::BorshSerialize;
use dirs::home_dir;
use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::lamports_to_sol,
    signature::{read_keypair_file, Keypair, Signer},
    signer::EncodableKey,
    system_program,
    transaction::Transaction,
};
use solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding};

#[derive(serde::Deserialize)]
pub struct Config {
    pub json_rpc_url: String,
    pub keypair_path: String,
}

fn get_program_id() -> Pubkey {
    const PROGRAM_KEYPAIR_PATH: &str = "../game/target/deploy/tic_tac_toe-keypair.json";
    *Lazy::new(|| {
        let program_id = read_keypair_file(PROGRAM_KEYPAIR_PATH).unwrap().pubkey();
        let program_account = get_rpc_client().get_account(&program_id).unwrap();
        if program_account.executable {
            println!("Program ID: {}", program_id);
        } else {
            println!("ERROR PROGRAM ID NOT DEPLOYED: {}", program_id);
        }
        program_id
    })
}

static SOLANA_CONFIG: Lazy<Config> = Lazy::new(|| {
    let path = home_dir().unwrap().join(".config/solana/cli/config.yml");
    let config_content = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&config_content).unwrap()
});

fn get_rpc_client() -> RpcClient {
    RpcClient::new(&SOLANA_CONFIG.json_rpc_url)
}

fn get_anchor_instruction_bytes(input: &str) -> Vec<u8> {
    let h = solana_sdk::hash::hash(input.as_bytes());
    h.as_ref()[0..8].to_vec()
}

fn print_balance(account_name: &str, pubkey: Pubkey) {
    let balance_lamports = get_rpc_client().get_balance(&pubkey).unwrap();
    println!(
        "{} balance: {} SOL",
        account_name,
        lamports_to_sol(balance_lamports)
    );
}

fn get_payer_key() -> Keypair {
    let path = &*SOLANA_CONFIG.keypair_path;
    let payer = Keypair::read_from_file(path).unwrap();
    print_balance("Payer", payer.pubkey());
    payer
}

async fn send_transaction_and_print_logs(
    rpc_client: &RpcClient,
    transaction: &Transaction,
) -> solana_client::client_error::Result<()> {
    let signature = rpc_client.send_and_confirm_transaction(transaction)?;
    println!("Transaction signature: {:?}", signature);

    let transaction_with_meta =
        rpc_client.get_transaction(&signature, UiTransactionEncoding::Json)?;
    let meta = transaction_with_meta.transaction.meta.unwrap();
    if let OptionSerializer::Some(logs) = meta.log_messages {
        if logs.len() > 0 {
            println!("Logs:");
            for log_message in logs {
                println!("  {}", log_message);
            }
        }
    }

    Ok(())
}

async fn setup_game(
    rpc_client: &RpcClient,
    program_id: Pubkey,
    payer: &Keypair,
    player_two: Pubkey,
) -> Keypair {
    let mut instruction_data = get_anchor_instruction_bytes("global:setup_game");
    player_two.serialize(&mut instruction_data).unwrap();

    let game_keypair = Keypair::new();
    println!("Game keypair: {}", game_keypair.pubkey());

    let increment_instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(game_keypair.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(player_two, false),
        ],
    );

    let recent_block_hash = rpc_client.get_latest_blockhash().unwrap();

    // Send transaction with increment instruction
    let mut transaction =
        Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));
    transaction.sign(&[payer, &game_keypair], recent_block_hash);

    send_transaction_and_print_logs(rpc_client, &transaction)
        .await
        .unwrap();

    game_keypair
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let program_id = get_program_id();
    let rpc_client = get_rpc_client();
    let payer = get_payer_key();
    let player_two = Keypair::new();
    let game_key_pair = setup_game(&rpc_client, program_id, &payer, player_two.pubkey()).await;
    println!("Game keypair: {}", game_key_pair.pubkey());
}
