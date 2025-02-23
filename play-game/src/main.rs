#![allow(unexpected_cfgs)]

use borsh::BorshSerialize;
use dirs::home_dir;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_sdk::{
    instruction::Instruction,
    signature::{read_keypair_file, Keypair, Signer},
    signer::EncodableKey,
    transaction::Transaction,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::UiTransactionEncoding;

const PROGRAM_KEYPAIR_PATH: &str = "../game/target/deploy/tic_tac_toe-keypair.json";

fn get_anchor_instruction_bytes(input: &str) -> Vec<u8> {
    let h = solana_sdk::hash::hash(input.as_bytes());
    h.as_ref()[0..8].to_vec()
}

async fn get_payer_key(rpc_client: &RpcClient) -> Keypair {
    let payer =
        Keypair::read_from_file(home_dir().unwrap().join(".config/solana/id.json")).unwrap();
    let balance = rpc_client.get_balance(&payer.pubkey()).unwrap();
    println!("Balance: {}", balance);
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
        }
        for log_message in logs {
            println!("  {}", log_message);
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
            AccountMeta::new(game_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
            //AccountMeta::new(player_two, false),
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
    let program_id = read_keypair_file(PROGRAM_KEYPAIR_PATH).unwrap().pubkey();
    println!("Program ID: {}", program_id);

    let use_devnet = false;
    let rpc_url: &str = if use_devnet {
        "https://api.devnet.solana.com"
    } else {
        "http://localhost:8899"
    };

    let player_two = Keypair::new();

    let rpc_client = RpcClient::new(rpc_url);
    let payer = get_payer_key(&rpc_client).await;
    let game_key_pair = setup_game(&rpc_client, program_id, &payer, player_two.pubkey()).await;
    println!("Game keypair: {}", game_key_pair.pubkey());
}
