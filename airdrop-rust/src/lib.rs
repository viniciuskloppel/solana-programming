mod programs;
const RPC_URL: &str = "https://api.devnet.solana.com";

fn get_transaction_url(signature: &str) -> String {
    format!("https://explorer.solana.com/tx/{signature}?cluster=devnet",)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::programs::wba_prereq::{CompleteArgs, UpdateArgs, WbaPrereqProgram};
    use bs58;
    use solana_client::rpc_client::{self, RpcClient};
    use solana_program::{pubkey::Pubkey, system_instruction::transfer};
    use solana_sdk::{
        message::Message,
        native_token::sol_to_lamports,
        signature::{self, read_keypair_file, Keypair, Signer},
        system_program,
        transaction::Transaction,
    };
    use std::io::{self, BufRead, Read};
    use std::str::FromStr;

    fn create_transfer_instruction(
        from: &Pubkey,
        to: &Pubkey,
        lamports: u64,
    ) -> [solana_sdk::instruction::Instruction; 1] {
        [transfer(&from, &to, lamports)]
    }

    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!(
            "You've generated a new Solana wallet: {}",
            kp.pubkey().to_string()
        );
        println!("");
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }
    #[test]
    fn airdrop() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        let client = RpcClient::new(RPC_URL);
        match client.request_airdrop(&keypair.pubkey(), sol_to_lamports(0.5f64)) {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!(
                    "https://explorer.solana.com/tx/{}?cluster=devnet",
                    s.to_string()
                );
            }
            Err(e) => println!("Oops, something went wrong: {}", e.to_string()),
        }
    }
    #[test]
    fn transfer_sol() {
        let keypair = read_keypair_file("./dev-wallet.json").expect("Couldn't find wallet file");
        let recipient =
            read_keypair_file("./recipient-wallet.json").expect("Couldn't find wallet file");
        let to_pubkey = recipient.pubkey();

        let rpc_client = RpcClient::new(RPC_URL);
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        let balance = get_balance(&keypair.pubkey().to_string());

        let transfer_instructions =
            create_transfer_instruction(&keypair.pubkey(), &to_pubkey, balance);

        let message = Message::new_with_blockhash(
            &transfer_instructions,
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        let transfer_instructions =
            create_transfer_instruction(&keypair.pubkey(), &to_pubkey, balance - fee);

        let transaction = Transaction::new_signed_with_payer(
            &transfer_instructions,
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        let transaction_url = get_transaction_url(&signature.to_string());
        println!("Success! Check out your TX here:\n{}", transaction_url);
    }
    #[test]
    fn enroll() {
        let signer =
            read_keypair_file("./recipient-wallet.json").expect("Couldn't find wallet file");
        let prereq = WbaPrereqProgram::derive_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ]);

        let args = CompleteArgs {
            github: b"viniciuskloppel".to_vec(),
        };
        let rpc_client = RpcClient::new(RPC_URL);
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Couldn't get recent blockhash");

        let transaction = WbaPrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        let transaction_url = get_transaction_url(&signature.to_string());
        println!("Success! Check your TX here:\n{}", transaction_url);
    }
    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet)
    }
    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        println!("Your private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }
    #[test]
    fn get_public_key() {
        let stdin = io::stdin();
        let wallet_file_path = stdin.lock().lines().next().unwrap().unwrap();
        let keypair = read_keypair_file(wallet_file_path).expect("Couldn't find wallet file");
        println!("Your public key is:");
        println!("{:?}", keypair.pubkey());
    }
    fn get_balance(address: &str) -> u64 {
        let rpc_client = RpcClient::new(RPC_URL);
        let pubkey = Pubkey::from_str(address).unwrap();

        rpc_client
            .get_balance(&pubkey)
            .expect("Couldn't fetch balance")
    }
}
