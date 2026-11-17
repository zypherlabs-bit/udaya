//! `ug-node` command line interface.

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use ug_core::crypto::{self, Hash32};
use ug_core::genesis::{Allocation, Genesis, GenesisConfig, GENESIS_VERSION};
use ug_node::config::NodeConfig;
use ug_node::store::StorePaths;
use ug_node::{load_seed, read_disk_status, save_seed, verify_data_dir, Node};

#[derive(Parser)]
#[command(
    name = "ug-node",
    version,
    about = "Udaya Global node (independent experimental L1)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new independent genesis and node configuration.
    Init {
        #[arg(long)]
        data_dir: PathBuf,
        /// Allocation as HEX_PUBKEY:SATS. May be repeated. Never defaulted.
        #[arg(long = "allocation", value_name = "HEX:SATS")]
        allocations: Vec<String>,
        /// Genesis validator Ed25519 public key (64 hex). May be repeated.
        #[arg(long = "validator", value_name = "HEX")]
        validators: Vec<String>,
        /// Unix timestamp for genesis. Defaults to the current time.
        #[arg(long)]
        timestamp: Option<u64>,
        #[arg(long, default_value = ug_core::DEVNET_CHAIN_ID)]
        chain_id: String,
        #[arg(long, default_value = ug_core::DEVNET_ADDRESS_HRP)]
        address_hrp: String,
        #[arg(long, default_value_t = ug_core::DEVNET_P2P_PORT)]
        p2p_port: u16,
        #[arg(long, default_value_t = ug_core::DEFAULT_MIN_FEE_PER_BYTE)]
        min_fee_per_byte: u64,
        /// Four-byte network magic as 8 hex characters.
        #[arg(long)]
        network_magic: Option<String>,
    },
    /// Generate an Ed25519 validator/peer seed file.
    Keygen {
        #[arg(long)]
        out: PathBuf,
        /// Deterministic seed for test networks. Production keys must use the
        /// operating system CSPRNG.
        #[arg(long)]
        seed: Option<String>,
    },
    /// Run a validator node.
    Run {
        #[arg(long)]
        data_dir: PathBuf,
        /// Seed file. Defaults to <data-dir>/validator.key.
        #[arg(long)]
        key_file: Option<PathBuf>,
        /// Override the configured P2P listen address.
        #[arg(long)]
        listen: Option<SocketAddr>,
        /// Additional bootstrap peer address. May be repeated.
        #[arg(long)]
        bootstrap: Vec<SocketAddr>,
    },
    /// Print status from disk without opening the network.
    Status {
        #[arg(long)]
        data_dir: PathBuf,
    },
    /// Fully replay and verify all blocks and the state root.
    Verify {
        #[arg(long)]
        data_dir: PathBuf,
    },
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn parse_hash(text: &str, what: &str) -> Result<Hash32, String> {
    if text.len() != 64 {
        return Err(format!("{what} must be 64 hexadecimal characters"));
    }
    let bytes = hex::decode(text).map_err(|_| format!("{what} is not hexadecimal"))?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    if out == [0u8; 32] {
        return Err(format!("{what} must not be all zero"));
    }
    Ok(out)
}

fn parse_magic(text: &str) -> Result<[u8; 4], String> {
    let bytes = hex::decode(text).map_err(|_| "network magic is not hexadecimal".to_string())?;
    if bytes.len() != 4 {
        return Err("network magic must be exactly 4 bytes (8 hex characters)".into());
    }
    let mut out = [0u8; 4];
    out.copy_from_slice(&bytes);
    Ok(out)
}

// CLI entry point mirrors the documented init flags one-to-one; grouping them
// into a struct would only obscure the clap-derived argument mapping.
#[allow(clippy::too_many_arguments)]
fn cmd_init(
    data_dir: PathBuf,
    allocations: Vec<String>,
    validators: Vec<String>,
    timestamp: Option<u64>,
    chain_id: String,
    address_hrp: String,
    p2p_port: u16,
    min_fee_per_byte: u64,
    network_magic: Option<String>,
) -> Result<(), String> {
    let paths = StorePaths::new(&data_dir);
    if paths.genesis.exists() || paths.node_config.exists() {
        return Err(format!(
            "{} already contains a genesis or node configuration; refusing to overwrite",
            data_dir.display()
        ));
    }
    if allocations.is_empty() {
        return Err("at least one explicit --allocation is required".into());
    }
    if validators.len() < 4 {
        return Err("at least four genesis validators are required".into());
    }
    let allocations: Vec<Allocation> = allocations
        .iter()
        .map(|entry| {
            let (recipient, value) = entry
                .split_once(':')
                .ok_or_else(|| "allocation must be HEX:SATS".to_string())?;
            let recipient = parse_hash(recipient, "allocation recipient")?;
            let value: u64 = value
                .parse()
                .map_err(|_| "allocation value is not a u64".to_string())?;
            Ok(Allocation { recipient, value })
        })
        .collect::<Result<_, String>>()?;
    let validators: Vec<Hash32> = validators
        .iter()
        .map(|entry| parse_hash(entry, "validator key"))
        .collect::<Result<_, String>>()?;
    let network_magic = match network_magic {
        Some(text) => parse_magic(&text)?,
        None => ug_core::GLOBAL_MAGIC,
    };
    let config = GenesisConfig {
        version: GENESIS_VERSION,
        chain_id,
        network_magic,
        address_hrp,
        p2p_port,
        min_fee_per_byte,
        timestamp: timestamp.unwrap_or_else(unix_now),
        allocations,
        validators,
    };
    let genesis = Genesis::build(config).map_err(|error| error.to_string())?;
    paths.ensure().map_err(|error| error.to_string())?;
    genesis
        .save(&paths.genesis)
        .map_err(|error| error.to_string())?;
    let node_config = NodeConfig {
        listen: format!("127.0.0.1:{p2p_port}")
            .parse()
            .map_err(|_| "invalid default listen address".to_string())?,
        genesis_hash: Some(hex::encode(genesis.hash())),
        ..NodeConfig::default()
    };
    node_config
        .save(&paths.node_config)
        .map_err(|error| error.to_string())?;
    println!("chain_id:      {}", genesis.chain_params().chain_id);
    println!("genesis_hash:  {}", hex::encode(genesis.hash()));
    println!("supply:        {}", genesis.total_allocated());
    println!("validators:    {}", genesis.validators().len());
    println!("data_dir:      {}", data_dir.display());
    println!("genesis.json and node.json written; pin the genesis hash out of band.");
    Ok(())
}

fn cmd_keygen(out: PathBuf, seed: Option<String>) -> Result<(), String> {
    let seed: [u8; 32] = match seed {
        Some(text) => {
            let bytes = hex::decode(&text).map_err(|_| "seed is not hexadecimal".to_string())?;
            if bytes.len() != 32 {
                return Err("seed must be exactly 32 bytes".into());
            }
            let mut out = [0u8; 32];
            out.copy_from_slice(&bytes);
            out
        }
        None => {
            let mut seed: [u8; 32] = rand::random();
            while seed == [0u8; 32] {
                seed = rand::random();
            }
            seed
        }
    };
    if seed == [0u8; 32] {
        return Err("refusing an all-zero seed".into());
    }
    save_seed(&out, &seed).map_err(|error| error.to_string())?;
    let key = ed25519_dalek::SigningKey::from_bytes(&seed);
    let public = crypto::public_key(&key);
    println!("key_file:      {}", out.display());
    println!("validator_key: {}", hex::encode(public));
    println!("peer_id:       {}", hex::encode(public));
    println!("Back up this file. It controls validator votes and peer identity.");
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("ug-node: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            data_dir,
            allocations,
            validators,
            timestamp,
            chain_id,
            address_hrp,
            p2p_port,
            min_fee_per_byte,
            network_magic,
        } => cmd_init(
            data_dir,
            allocations,
            validators,
            timestamp,
            chain_id,
            address_hrp,
            p2p_port,
            min_fee_per_byte,
            network_magic,
        ),
        Commands::Keygen { out, seed } => cmd_keygen(out, seed),
        Commands::Run {
            data_dir,
            key_file,
            listen,
            bootstrap,
        } => {
            let paths = StorePaths::new(&data_dir);
            let mut config =
                NodeConfig::load(&paths.node_config).map_err(|error| error.to_string())?;
            if let Some(listen) = listen {
                config.listen = listen;
            }
            for address in bootstrap {
                if !config.bootstrap.contains(&address) {
                    config.bootstrap.push(address);
                }
            }
            let key_file = key_file.unwrap_or_else(|| paths.data_dir.join("validator.key"));
            // Validate the seed before spending time on state replay.
            let _ = load_seed(&key_file).map_err(|error| error.to_string())?;
            let node = Node::open(&data_dir, &key_file, config)
                .await
                .map_err(|error| error.to_string())?;
            let handle = node.spawn();
            println!(
                "ug-node running: peer_id={} listen={} genesis={}",
                handle.peer_id,
                handle.listen_addr,
                hex::encode(handle_genesis(&data_dir)?)
            );
            let status = handle.status().await.map_err(|error| error.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&status).map_err(|error| error.to_string())?
            );
            tokio::signal::ctrl_c()
                .await
                .map_err(|error| error.to_string())?;
            handle.shutdown().await.map_err(|error| error.to_string())?;
            println!("ug-node stopped cleanly");
            Ok(())
        }
        Commands::Status { data_dir } => {
            let status = read_disk_status(&data_dir).map_err(|error| error.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&status).map_err(|error| error.to_string())?
            );
            Ok(())
        }
        Commands::Verify { data_dir } => {
            let status = verify_data_dir(&data_dir).map_err(|error| error.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&status).map_err(|error| error.to_string())?
            );
            println!("verification: OK");
            Ok(())
        }
    }
}

fn handle_genesis(data_dir: &std::path::Path) -> Result<Hash32, String> {
    let paths = StorePaths::new(data_dir);
    let genesis = Genesis::load(&paths.genesis, None, None).map_err(|error| error.to_string())?;
    Ok(genesis.hash())
}
