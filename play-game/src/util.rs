use std::fs;

use dirs::home_dir;
use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token,
    signature::{read_keypair_file, EncodableKey, Keypair, Signer},
};

#[derive(serde::Deserialize)]
pub struct Config {
    pub json_rpc_url: String,
    pub keypair_path: String,
}

pub fn get_program_id() -> Pubkey {
    const PROGRAM_KEYPAIR_PATH: &str = "../game/target/deploy/tic_tac_toe-keypair.json";
    *Lazy::new(|| {
        let program_id = read_keypair_file(PROGRAM_KEYPAIR_PATH).unwrap().pubkey();
        match get_rpc_client(CommitmentConfig::processed()).get_account(&program_id) {
            Ok(account) => {
                println!("Program ID: {}", program_id);
                account
            }
            Err(err) => {
                eprintln!("ERROR PROGRAM ID {} NOT DEPLOYED: {}", program_id, err);
                std::process::exit(1);
            }
        };
        program_id
    })
}

static SOLANA_CONFIG: Lazy<Config> = Lazy::new(|| {
    let path = home_dir().unwrap().join(".config/solana/cli/config.yml");
    let config_content = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&config_content).unwrap()
});

pub fn get_rpc_url() -> String {
    SOLANA_CONFIG.json_rpc_url.clone()
}

pub fn get_rpc_client(config: CommitmentConfig) -> RpcClient {
    RpcClient::new_with_commitment(get_rpc_url(), config)
}

pub fn get_anchor_discriminator(input: &str) -> [u8; 8] {
    let h = solana_sdk::hash::hash(input.as_bytes());
    h.as_ref()[0..8].try_into().unwrap()
}

fn address_string(public_key: &Pubkey, name: &str) -> String {
    let address = public_key.to_string();
    let shortened_address = format!("{}...{}", &address[0..4], &address[address.len() - 4..]);
    if !name.is_empty() {
        format!("{} ({})", shortened_address, name)
    } else {
        shortened_address
    }
}

pub fn print_balance(rpc_client: &RpcClient, name: &str, public_key: &Pubkey) {
    let balance = rpc_client.get_balance(public_key).unwrap();
    let printed_addr = address_string(public_key, name);

    match rpc_client.get_account(public_key) {
        Ok(account) => {
            let printed_owner = address_string(&account.owner, "owner");
            println!(
                "Balance of {}: {} SOL (owner: {})",
                printed_addr,
                native_token::lamports_to_sol(balance),
                printed_owner
            );
        }
        Err(err) => {
            eprintln!("Unable to get balance for {}: {}", printed_addr, err);
        }
    }
}

pub fn get_payer_key() -> Keypair {
    let path = &*SOLANA_CONFIG.keypair_path;
    Keypair::read_from_file(path).unwrap()
}
