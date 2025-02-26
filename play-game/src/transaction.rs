use std::thread;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    signature::Keypair, system_transaction, transaction::Transaction,
};
use solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding};

pub fn send_transaction_and_print_logs(
    rpc_client: &RpcClient,
    transaction: &Transaction,
) -> solana_client::client_error::Result<()> {
    let signature = match rpc_client.send_and_confirm_transaction(transaction) {
        Ok(sig) => sig,
        Err(err) => {
            eprintln!("Transaction failed: {:?}", err);
            return Err(err);
        }
    };
    println!("Transaction signature: {:?}", signature);
    thread::sleep(std::time::Duration::from_secs(5));

    let transaction_with_meta = match
        rpc_client.get_transaction(&signature, UiTransactionEncoding::Json) {
        Ok(transaction_with_meta) => transaction_with_meta,
        Err(err) => {
            eprintln!("Failed to get transaction: {:?}", err);
            return Err(err.into());
        }
    };
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

pub(crate) fn transfer(rpc_client: &RpcClient, lamports: u64, from: &Keypair, to: &Pubkey) {
    let recent_block_hash = rpc_client.get_latest_blockhash().unwrap();
    let tx = system_transaction::transfer(from, to, lamports, recent_block_hash);
    let signature = rpc_client.send_and_confirm_transaction(&tx).unwrap();
    println!(
        "Transferred {} lamports to new account (sig: {:?})",
        lamports, signature
    );
}

#[cfg(test)]
mod tests {
    use solana_sdk::{
        native_token,
        signature::{Keypair, Signer},
    };

    use super::*;
    use crate::util::{get_payer_key, get_rpc_client};

    #[test]
    fn test_transfer_sol() {
        let payer = get_payer_key();
        let dest_account = Keypair::new();

        let rpc_client = get_rpc_client();
        let lamports = native_token::sol_to_lamports(0.1);

        let start_bal_from = rpc_client.get_balance(&payer.pubkey()).unwrap();
        let start_bal_to = rpc_client.get_balance(&dest_account.pubkey()).unwrap();

        transfer(&rpc_client, lamports, &payer, &dest_account.pubkey());

        let end_bal_from = rpc_client.get_balance(&payer.pubkey()).unwrap();
        let end_bal_to = rpc_client.get_balance(&dest_account.pubkey()).unwrap();
        let fees = start_bal_from - (end_bal_from + lamports);

        assert_eq!(start_bal_from - end_bal_from, lamports + fees);
        assert_eq!(end_bal_to - start_bal_to, lamports);

        println!("Transfer fee was {} lamports ({} SOL)", fees, native_token::lamports_to_sol(fees));
    }
}
