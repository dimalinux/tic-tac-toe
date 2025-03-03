use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token,
    signature::{Keypair, Signature},
    signer::Signer,
    system_transaction,
    transaction::Transaction,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status_client_types::{UiTransactionEncoding, UiTransactionStatusMeta};

pub fn send_transaction_and_print_logs(
    rpc_client: &RpcClient,
    transaction: &Transaction,
) -> solana_client::client_error::Result<()> {
    let signature = match rpc_client.send_and_confirm_transaction_with_spinner(transaction) {
        Ok(sig) => sig,
        Err(err) => {
            eprintln!("Transaction failed: {:?}", err);
            return Err(err);
        }
    };
    println!("Transaction signature: {:?}", signature);

    let meta = get_transaction_meta(rpc_client, &signature);
    println!(
        "Transaction fee: {} SOL",
        native_token::lamports_to_sol(meta.fee)
    );
    if let OptionSerializer::Some(logs) = meta.log_messages {
        if !logs.is_empty() {
            println!("Logs:");
            for log_message in logs {
                println!("  {}", log_message);
            }
        }
    }

    Ok(())
}

fn get_transaction_meta(rpc_client: &RpcClient, signature: &Signature) -> UiTransactionStatusMeta {
    let rpc_trans_config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        ..RpcTransactionConfig::default()
    };
    let transaction_with_meta = rpc_client
        .get_transaction_with_config(signature, rpc_trans_config)
        .expect("Failed to get transaction metadata");
    transaction_with_meta
        .transaction
        .meta
        .expect("Transaction metadata not found")
}

pub(crate) fn transfer(rpc_client: &RpcClient, lamports: u64, from: &Keypair, to: &Pubkey) {
    let recent_block_hash = rpc_client.get_latest_blockhash().unwrap();
    let tx = system_transaction::transfer(from, to, lamports, recent_block_hash);
    send_transaction_and_print_logs(rpc_client, &tx).unwrap();
}

pub(crate) fn sweep(rpc_client: &RpcClient, from: &Keypair, to: &Pubkey) {
    let balance = rpc_client.get_balance(&from.pubkey()).unwrap();
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    let tx = system_transaction::transfer(from, to, 1, recent_blockhash);

    let fee = rpc_client.get_fee_for_message(&tx.message).unwrap();
    let transfer_amount = balance.saturating_sub(fee);

    if transfer_amount > 0 {
        let tx = system_transaction::transfer(from, to, transfer_amount, recent_blockhash);
        send_transaction_and_print_logs(rpc_client, &tx).unwrap();
    } else {
        println!("Balance too low to sweep funds.");
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::{
        native_token,
        signature::{Keypair, Signer},
    };

    use super::*;
    use crate::util::get_payer_key;

    // To change to devnet, use: "https://api.devnet.solana.com";
    const RPC_URL: &str = "http://localhost:8899";

    #[test]
    fn test_transfer_sol() {
        let payer = get_payer_key();
        let dest_account = Keypair::new();

        let rpc_client = RpcClient::new_with_commitment(RPC_URL, CommitmentConfig::processed());
        let lamports = native_token::sol_to_lamports(0.1);

        let start_bal_from = rpc_client.get_balance(&payer.pubkey()).unwrap();
        let start_bal_to = rpc_client.get_balance(&dest_account.pubkey()).unwrap();

        transfer(&rpc_client, lamports, &payer, &dest_account.pubkey());

        let end_bal_from = rpc_client.get_balance(&payer.pubkey()).unwrap();
        let end_bal_to = rpc_client.get_balance(&dest_account.pubkey()).unwrap();
        let fees = start_bal_from - (end_bal_from + lamports);

        assert_eq!(start_bal_from - end_bal_from, lamports + fees);
        assert_eq!(end_bal_to - start_bal_to, lamports);

        // sweep funds back from temp account to payer
        sweep(
            &rpc_client,
            &dest_account,   // from account, since we are sweeping back
            &payer.pubkey(), // sweep destination
        );

        // if we successfully swept all the funds, the validator removes the account
        let err = rpc_client
            .get_account(&dest_account.pubkey())
            .err()
            .unwrap();
        assert!(err.to_string().contains("AccountNotFound"));
    }
}
