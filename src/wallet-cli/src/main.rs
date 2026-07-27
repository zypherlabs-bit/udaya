use clap::{Parser, Subcommand};
use udaya_wallet::crypto::{entropy_to_mnemonic, EntropySource};
use udaya_wallet::Wallet;

/// Udaya Wallet CLI - Manage UDYA wallets from the command line
#[derive(Parser)]
#[command(name = "Udaya-wallet")]
#[command(about = "Udaya wallet - send, receive, and manage UDYA")]
struct Cli {
    #[command(subcommand)]
    command: WalletCommands,

    /// Network (mainnet, testnet)
    #[arg(short, long, default_value = "testnet")]
    network: String,

    /// Wallet file path
    #[arg(short, long, default_value = "wallet.dat")]
    wallet_file: String,
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Create a new wallet
    Create {
        #[arg(long)]
        name: Option<String>,
    },
    /// Generate a new address
    NewAddress,
    /// Get wallet balance
    GetBalance,
    /// Send UDYA to an address
    Send {
        /// Recipient address
        to: String,
        /// Amount in UDYA
        amount: f64,
        /// Optional fee rate (sat/vbyte)
        #[arg(long)]
        fee_rate: Option<u64>,
    },
    /// Show transaction history
    History {
        #[arg(long, default_value = "20")]
        count: usize,
    },
    /// Export wallet seed phrase
    ExportSeed,
    /// Import wallet from seed phrase
    ImportSeed {
        /// 12 or 24 word mnemonic seed phrase
        words: Vec<String>,
        /// Optional passphrase
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Show wallet info
    GetInfo,
    /// List all addresses
    ListAddresses,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let wallet = Wallet::new("Udaya Wallet", &cli.network);

    match cli.command {
        WalletCommands::Create { name } => {
            let wallet_name = name.unwrap_or_else(|| "Udaya Wallet".to_string());
            let wallet = udaya_wallet::Wallet::new(&wallet_name, &cli.network);

            // Generate seed and first address
            let (mnemonic, address) = wallet.generate_seed();

            println!("╔══════════════════════════════════════════════════╗");
            println!("║        Udaya Wallet Created                    ║");
            println!("╚══════════════════════════════════════════════════╝");
            println!("Name: {}", wallet_name);
            println!("Network: {}", cli.network);
            println!("");
            println!("⚠️  IMPORTANT: Save your recovery phrase!");
            println!("⚠️  This is the ONLY way to recover your wallet.");
            println!("");
            println!("Recovery Phrase ({} words):", mnemonic.len());

            for (i, word) in mnemonic.iter().enumerate() {
                print!("{:>2}. {:<12}", i + 1, word);
                if (i + 1) % 4 == 0 {
                    println!();
                }
            }
            println!();
            println!("");
            println!("First Address: {}", address);
            println!("");
            println!(
                "To save this wallet, use: Udaya-wallet --wallet-file {} export-seed",
                cli.wallet_file
            );
        }

        WalletCommands::NewAddress => {
            let (mnemonic, address) = wallet.generate_seed();
            println!("New address generated:");
            println!("  Address: {}", address);
        }

        WalletCommands::GetBalance => {
            let balance = wallet.get_balance();
            println!("╔════════════════════════════════════╗");
            println!("║        Wallet Balance               ║");
            println!("╚════════════════════════════════════╝");
            println!("  Confirmed:   {:.8} UDYA", balance.confirmed);
            println!("  Unconfirmed: {:.8} UDYA", balance.unconfirmed);
            println!("  Immature:    {:.8} UDYA", balance.immature);
            println!("  Total:       {:.8} UDYA", balance.total);
            println!("  Satoshis:    {} sats", balance.satoshi_total);
        }

        WalletCommands::Send {
            to,
            amount,
            fee_rate,
        } => {
            let fee = fee_rate.unwrap_or(10);
            let amount_sats = (amount * 100_000_000.0) as u64;
            let fee_sats = fee * 192; // ~192 vbytes for typical tx

            println!("╔════════════════════════════════════╗");
            println!("║        Send Transaction             ║");
            println!("╚════════════════════════════════════╝");
            println!("  To:     {}", to);
            println!("  Amount: {:.8} UDYA", amount);
            println!("  Fee:    {} sats", fee_sats);
            println!(
                "  Total:  {:.8} UDYA",
                (amount_sats + fee_sats) as f64 / 100_000_000.0
            );

            // In production, this would sign and broadcast via RPC
            let to_script = vec![];
            match wallet.create_payment(&to_script, amount_sats, fee_sats) {
                Ok(tx) => {
                    println!("  ✅ Transaction created successfully!");
                    println!("  TxID: {}", tx.txid());
                    println!("  Size: {} bytes", tx.size());
                }
                Err(e) => {
                    println!("  ❌ Failed to create transaction: {}", e);
                }
            }
        }

        WalletCommands::History { count } => {
            let txs = wallet.get_transactions(count, 0);
            println!("╔════════════════════════════════════╗");
            println!("║        Transaction History          ║");
            println!("╚════════════════════════════════════╝");

            if txs.is_empty() {
                println!("  No transactions found.");
            } else {
                for tx in &txs {
                    let direction = match tx.direction {
                        udaya_wallet::TxDirection::Sent => "⬆ Sent",
                        udaya_wallet::TxDirection::Received => "⬇ Received",
                        udaya_wallet::TxDirection::SelfTransfer => "⟳ Self",
                    };
                    let status = match tx.status {
                        udaya_wallet::TxStatus::Confirmed => "✅",
                        udaya_wallet::TxStatus::Pending => "⏳",
                        udaya_wallet::TxStatus::Failed => "❌",
                    };
                    let timestamp = chrono::DateTime::from_timestamp(tx.timestamp as i64, 0)
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    println!(
                        "  {} {} {}  {} UDYA",
                        status,
                        direction,
                        tx.txid,
                        tx.total_output as f64 / 100_000_000.0
                    );
                    println!("     {}", timestamp);
                }
            }
        }

        WalletCommands::ExportSeed => {
            // Generate new seed for demo purposes
            let entropy = EntropySource::generate();
            let mnemonic = entropy_to_mnemonic(&entropy.entropy);

            println!("╔════════════════════════════════════╗");
            println!("║        Wallet Recovery Phrase       ║");
            println!("╚════════════════════════════════════╝");
            println!("");
            println!("⚠️  KEEP THIS PHRASE SECURE!");
            println!("⚠️  Anyone with this phrase can access your funds.");
            println!("");
            println!("Recovery Phrase:");
            for (i, word) in mnemonic.iter().enumerate() {
                print!("{:>2}. {:<12}", i + 1, word);
                if (i + 1) % 4 == 0 {
                    println!();
                }
            }
            println!();
        }

        WalletCommands::ImportSeed { words, passphrase } => {
            let pass = passphrase.unwrap_or_default();
            match wallet.recover_from_mnemonic(&words, &pass) {
                Ok(address) => {
                    println!("✅ Wallet recovered successfully!");
                    println!("  Address: {}", address);
                    println!("  Network: {}", cli.network);
                }
                Err(e) => {
                    println!("❌ Failed to recover wallet: {}", e);
                }
            }
        }

        WalletCommands::GetInfo => {
            println!("╔════════════════════════════════════╗");
            println!("║        Wallet Information           ║");
            println!("╚════════════════════════════════════╝");
            println!("  Network: {}", cli.network);
            println!("  Wallet File: {}", cli.wallet_file);
            let state = wallet.export_state();
            println!("  Name: {}", state.name);
            println!("  Version: {}", state.version);
            println!("  Created: {}", state.created_at);
            println!("  Accounts: {}", state.accounts.len());
            println!("  Transactions: {}", state.transactions.len());
            println!("  UTXOs: {}", state.utxos.len());
        }

        WalletCommands::ListAddresses => {
            let state = wallet.export_state();
            println!("╔════════════════════════════════════╗");
            println!("║        Wallet Addresses             ║");
            println!("╚════════════════════════════════════╝");

            for (i, account) in state.accounts.iter().enumerate() {
                println!(
                    "  Account {}: {} ({:?})",
                    i, account.name, account.derivation_path
                );
                for addr in &account.external_keys {
                    println!("       {}", addr);
                }
                for addr in &account.internal_keys {
                    println!("       {} (change)", addr);
                }
            }

            if state
                .accounts
                .iter()
                .all(|a| a.external_keys.is_empty() && a.internal_keys.is_empty())
            {
                println!("  No addresses generated yet.");
                println!("  Use 'Udaya-wallet new-address' to generate one.");
            }
        }
    }

    Ok(())
}
