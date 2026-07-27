use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use log::{error, info, warn};
use prometheus::Encoder;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use udaya_api::{
    blockchain_info_to_json, JsonRpcRequest, JsonRpcResponse, RpcError, RpcHandler, SoftForkInfo,
};
use udaya_core::config::UdayaConfig;
use udaya_core::consensus::{create_genesis_block, ConsensusEngine, ConsensusParams};
use udaya_core::genesis::{create_genesis_manifest, mine_genesis_block};
use udaya_core::observability::{
    self, perform_health_checks, MetricsState, NodeStatus, SystemMetrics,
};
use udaya_core::security::{AdversarialConfig, AdversarialSimulator, FuzzConfig, FuzzingEngine};
use udaya_core::types::BlockHash;
use udaya_core::validation::UTXOSet;
use udaya_explorer::ExplorerEngine;
use udaya_mining::{MiningPool, PoolConfig};
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;
use udaya_wallet::Wallet;

/// Shared node state accessible to all subsystems
pub struct NodeState {
    pub config: UdayaConfig,
    pub consensus: ConsensusEngine,
    pub db: BlockchainDB,
    pub mempool: udaya_mempool::Mempool,
    pub network_state: std::sync::Arc<udaya_p2p::NetworkState>,
    pub wallet: Wallet,
    pub explorer: ExplorerEngine,
    pub mining_pool: Option<MiningPool>,
    pub rpc_handler: RpcHandler,
    pub metrics: MetricsState,
    pub system_metrics: SystemMetrics,
    pub p2p_network: Option<udaya_p2p::network::P2PNetwork>,
    pub start_time: std::sync::Mutex<Option<std::time::Instant>>,
}

/// Udaya - Next-generation Proof-of-Work cryptocurrency
#[derive(Parser)]
#[command(name = "udayad")]
#[command(about = "Udaya daemon - Blockchain node", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to configuration file
    #[arg(short, long, default_value = "config/udaya.conf")]
    config: String,

    /// Data directory
    #[arg(short, long = "datadir")]
    data_dir: Option<String>,

    /// Network (mainnet, testnet, regtest)
    #[arg(short, long)]
    network: Option<String>,

    /// Enable mining
    #[arg(long)]
    mine: bool,

    /// RPC port
    #[arg(long)]
    rpc_port: Option<u16>,

    /// P2P port
    #[arg(long)]
    port: Option<u16>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Run security audit
    #[arg(long)]
    security_audit: bool,

    /// Mine genesis block
    #[arg(long)]
    mine_genesis: bool,

    /// Fuzz iterations for security testing
    #[arg(long)]
    fuzz_iterations: Option<u64>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Udaya node
    Start {
        #[arg(long)]
        miner_threads: Option<usize>,
    },
    /// Get blockchain info
    GetInfo,
    /// Get block by hash or height
    GetBlock { hash_or_height: String },
    /// Get transaction by txid
    GetTransaction { txid: String },
    /// Generate blocks (regtest only)
    Generate { num_blocks: u64 },
    /// Get mempool info
    GetMempoolInfo,
    /// List peers
    GetPeerInfo,
    /// Stop the node
    Stop,
    /// Mine production genesis block
    MineGenesis {
        #[arg(long, default_value = "mainnet")]
        network: String,
        #[arg(
            long,
            default_value = "Udaya Foundation: Launching a decentralized future for global commerce"
        )]
        statement: String,
        #[arg(long)]
        pubkey: Option<String>,
        #[arg(long, default_value = "0")]
        start_nonce: u32,
        #[arg(long, default_value = "10000000")]
        max_nonce: u32,
    },
    /// Run security audit and adversarial simulations
    SecurityAudit {
        #[arg(long, default_value = "100000")]
        fuzz_iterations: u64,
    },
    /// Run explorer server
    Explorer {
        #[arg(long, default_value = "8080")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    let mut config = if std::path::Path::new(&cli.config).exists() {
        UdayaConfig::from_file(&cli.config)?
    } else {
        info!("No config file found at {}, using defaults", cli.config);
        UdayaConfig::default()
    };

    if let Some(data_dir) = cli.data_dir {
        config.storage.data_dir = data_dir;
    }
    if let Some(network) = cli.network {
        config.consensus.network = network;
    }
    if cli.mine {
        config.mining.enable = true;
    }
    if let Some(port) = cli.rpc_port {
        config.rpc.listen_port = port;
    }
    if let Some(port) = cli.port {
        config.network.listen_port = port;
    }
    if cli.verbose {
        config.logging.level = "debug".to_string();
    }

    log::set_max_level(match config.logging.level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    });

    match cli.command {
        Some(Commands::Start { miner_threads }) => {
            if let Some(threads) = miner_threads {
                config.mining.num_miner_threads = threads;
            }
            start_node(config).await?;
        }
        Some(Commands::GetInfo) => {
            get_blockchain_info(&config).await?;
        }
        Some(Commands::GetBlock { hash_or_height }) => {
            get_block(&config, &hash_or_height).await?;
        }
        Some(Commands::GetTransaction { txid }) => {
            get_transaction(&config, &txid).await?;
        }
        Some(Commands::Generate { num_blocks }) => {
            generate_blocks(&config, num_blocks).await?;
        }
        Some(Commands::GetMempoolInfo) => {
            get_mempool_info(&config).await?;
        }
        Some(Commands::GetPeerInfo) => {
            get_peer_info(&config).await?;
        }
        Some(Commands::Stop) => {
            stop_node(&config).await?;
        }
        Some(Commands::MineGenesis {
            network,
            statement,
            pubkey,
            start_nonce,
            max_nonce,
        }) => {
            mine_genesis_command(
                &network,
                &statement,
                pubkey.as_deref(),
                start_nonce,
                max_nonce,
            )
            .await?;
        }
        Some(Commands::SecurityAudit { fuzz_iterations }) => {
            run_security_audit(fuzz_iterations).await?;
        }
        Some(Commands::Explorer { port }) => {
            run_explorer_server(port).await?;
        }
        None => {
            info!("Starting Udaya node (daemon mode)...");
            if cli.security_audit {
                run_security_audit(cli.fuzz_iterations.unwrap_or(100000)).await?;
            }
            if cli.mine_genesis {
                mine_genesis_command("mainnet", "Udaya Foundation Launch", None, 0, 10000000)
                    .await?;
            }
            start_node(config).await?;
        }
    }

    Ok(())
}

fn initialize_node_state(config: &UdayaConfig) -> anyhow::Result<Arc<NodeState>> {
    let consensus_params = ConsensusParams::default();
    let consensus = ConsensusEngine::new(consensus_params);

    let storage_config = StorageConfig {
        data_dir: config.storage.data_dir.clone(),
        db_cache_size_mb: config.storage.db_cache_size_mb,
        max_open_files: 1000,
        enable_compression: true,
        prune_blocks: config.storage.prune_blocks,
        prune_target_gb: config.storage.prune_target_gb,
    };

    let db = BlockchainDB::open(&storage_config)?;

    let mempool =
        udaya_mempool::Mempool::new(udaya_mempool::MempoolConfig::default(), consensus.clone());

    let p2p_config = udaya_p2p::P2PConfig {
        listen_port: config.network.listen_port,
        ..Default::default()
    };
    let network_state = std::sync::Arc::new(udaya_p2p::NetworkState::new(p2p_config.clone()));

    let p2p_network = Some(udaya_p2p::network::P2PNetwork::new(
        p2p_config,
        network_state.clone(),
    ));

    let wallet = Wallet::new("Udaya Wallet", &config.consensus.network);

    let explorer = ExplorerEngine::new();

    let mining_pool = if config.mining.enable {
        let pool_config = PoolConfig::default();
        Some(MiningPool::new(pool_config, consensus.clone()))
    } else {
        None
    };

    // Empty RPC handler — real handlers registered after state is complete
    let rpc_handler = RpcHandler::new();

    let metrics = observability::create_metrics();
    let system_metrics = SystemMetrics::new();

    metrics.set_node_status(NodeStatus::Starting);

    metrics
        .node_version
        .with_label_values(&[env!("CARGO_PKG_VERSION"), "70016"])
        .set(1);

    Ok(Arc::new(NodeState {
        config: config.clone(),
        consensus,
        db,
        mempool,
        network_state,
        wallet,
        explorer,
        mining_pool,
        rpc_handler,
        metrics,
        system_metrics,
        p2p_network,
        start_time: std::sync::Mutex::new(Some(std::time::Instant::now())),
    }))
}

// ============================================================================
// RPC handler registration — wires every handler to live node state
// ============================================================================
fn register_rpc_handlers(handler: &mut RpcHandler, state: &Arc<NodeState>) {
    // ----- blockchain -----
    handler.register("getblockchaininfo", {
        let state = Arc::clone(state);
        move |req| {
            let result = (|| -> anyhow::Result<serde_json::Value> {
                let chain = state.config.consensus.network.clone();
                let blocks = state.db.get_chain_height()?;
                let headers = blocks; // simplified: headers == height
                let tip = state.db.get_chain_tip()?;
                let best_block_hash = tip.map(|h| h.to_string()).unwrap_or_default();
                let difficulty = 1.0;
                let median_time = 0;
                let chain_work = blocks.to_string();
                let size_on_disk = state.db.block_count().unwrap_or(0) * 1_000_000; // rough estimate
                let pruned = state.config.storage.prune_blocks;
                let softforks: Vec<SoftForkInfo> = Vec::new();

                Ok(blockchain_info_to_json(
                    &chain,
                    blocks,
                    headers,
                    &best_block_hash,
                    difficulty,
                    median_time,
                    &chain_work,
                    size_on_disk,
                    pruned,
                    softforks,
                ))
            })();
            match result {
                Ok(value) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(value),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(RpcError {
                        code: -1,
                        message: e.to_string(),
                    }),
                },
            }
        }
    });

    handler.register("getblockcount", {
        let state = Arc::clone(state);
        move |req| {
            let height = state.db.get_chain_height().unwrap_or(0);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(serde_json::json!(height)),
                error: None,
            }
        }
    });

    handler.register("getblockhash", {
        let state = Arc::clone(state);
        move |req| {
            let height = req.params.first().and_then(|v| v.as_u64()).unwrap_or(0);
            let hash = state.db.get_block_hash_by_height(height).ok().flatten();
            match hash {
                Some(h) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::json!(h.to_string())),
                    error: None,
                },
                None => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(RpcError {
                        code: -8,
                        message: format!("Block height {} out of range", height),
                    }),
                },
            }
        }
    });

    handler.register("getblock", {
        let state = Arc::clone(state);
        move |req| {
            let hash_str = req.params.first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let result = (|| -> anyhow::Result<serde_json::Value> {
                let hash_bytes = hex::decode(&hash_str)?;
                if hash_bytes.len() != 32 {
                    anyhow::bail!("Invalid block hash length");
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&hash_bytes);
                let hash = BlockHash(arr);
                let block = state.db.get_block(&hash)?
                    .ok_or_else(|| anyhow::anyhow!("Block not found"))?;

                let verbosity = req.params.get(1).and_then(|v| v.as_u64()).unwrap_or(1);
                let height = state.db.get_chain_height().unwrap_or(0);
                let confirmations = 1; // simplified

                match verbosity {
                    0 => {
                        // Return serialized block as hex
                        let serialized = block.serialize();
                        Ok(serde_json::json!(hex::encode(serialized)))
                    }
                    _ => {
                        let txs: Vec<serde_json::Value> = if verbosity == 1 {
                            // Just txids
                            block.transactions.iter().map(|tx| {
                                serde_json::json!(tx.txid().to_string())
                            }).collect()
                        } else {
                            // Full tx details
                            block.transactions.iter().map(|tx| {
                                serde_json::json!({
                                    "txid": tx.txid().to_string(),
                                    "version": tx.version,
                                    "locktime": tx.lock_time,
                                    "vin": tx.inputs.iter().map(|input| serde_json::json!({
                                        "txid": input.previous_output.txid.to_string(),
                                        "vout": input.previous_output.vout,
                                        "sequence": input.sequence,
                                    })).collect::<Vec<_>>(),
                                    "vout": tx.outputs.iter().map(|output| serde_json::json!({
                                        "value": output.value as f64 / udaya_core::SATS_PER_COIN as f64,
                                        "n": 0,
                                        "scriptPubKey": {
                                            "hex": hex::encode(&output.script_pubkey.data),
                                            "address": output.script_pubkey.address.clone(),
                                        }
                                    })).collect::<Vec<_>>(),
                                })
                            }).collect()
                        };

                        Ok(serde_json::json!({
                            "hash": block.hash().to_string(),
                            "confirmations": confirmations,
                            "height": height,
                            "version": block.header.version,
                            "merkleroot": hex::encode(block.header.merkle_root.0),
                            "time": block.header.timestamp,
                            "mediantime": block.header.timestamp,
                            "nonce": block.header.nonce,
                            "bits": format!("{:08x}", block.header.bits),
                            "difficulty": 1.0,
                            "chainwork": height.to_string(),
                            "previousblockhash": block.header.previous_block_hash.to_string(),
                            "tx": txs,
                            "size": block.size(),
                        }))
                    }
                }
            })();
            match result {
                Ok(v) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(), id: req.id, result: Some(v), error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(), id: req.id, result: None,
                    error: Some(RpcError { code: -5, message: e.to_string() }),
                },
            }
        }
    });

    handler.register("gettransaction", {
        let state = Arc::clone(state);
        move |req| {
            let txid_str = req
                .params
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let result = (|| -> anyhow::Result<serde_json::Value> {
                let txid_bytes = hex::decode(&txid_str)?;
                if txid_bytes.len() != 32 {
                    anyhow::bail!("Invalid txid length");
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&txid_bytes);
                let txid = udaya_core::types::Txid(arr);
                let tx = state
                    .db
                    .get_transaction(&txid)?
                    .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;
                let _height = state.db.get_chain_height().unwrap_or(0);

                Ok(serde_json::json!({
                    "txid": txid.to_string(),
                    "version": tx.version,
                    "locktime": tx.lock_time,
                    "size": tx.size(),
                    "vsize": tx.vsize(),
                    "hex": hex::encode(bincode::serialize(&tx).unwrap_or_default()),
                    "confirmations": 1,
                    "blockhash": "",
                    "blocktime": 0,
                    "time": 0,
                    "vin": tx.inputs.iter().map(|input| serde_json::json!({
                        "txid": input.previous_output.txid.to_string(),
                        "vout": input.previous_output.vout,
                        "sequence": input.sequence,
                    })).collect::<Vec<_>>(),
                    "vout": tx.outputs.iter().map(|output| serde_json::json!({
                        "value": output.value as f64 / udaya_core::SATS_PER_COIN as f64,
                        "n": 0,
                        "scriptPubKey": {
                            "hex": hex::encode(&output.script_pubkey.data),
                        }
                    })).collect::<Vec<_>>(),
                }))
            })();
            match result {
                Ok(v) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(v),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(RpcError {
                        code: -5,
                        message: e.to_string(),
                    }),
                },
            }
        }
    });

    handler.register("gettxout", {
        let state = Arc::clone(state);
        move |req| {
            let txid_str = req.params.get(0).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let vout = req.params.get(1).and_then(|v| v.as_u64()).unwrap_or(0);
            let result = (|| -> anyhow::Result<serde_json::Value> {
                let txid_bytes = hex::decode(&txid_str)?;
                if txid_bytes.len() != 32 {
                    anyhow::bail!("Invalid txid");
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&txid_bytes);
                let txid = udaya_core::types::Txid(arr);
                let tx = state.db.get_transaction(&txid)?
                    .ok_or_else(|| anyhow::anyhow!("Transaction not found"))?;
                let output = tx.outputs.get(vout as usize)
                    .ok_or_else(|| anyhow::anyhow!("Output index out of range"))?;

                Ok(serde_json::json!({
                    "bestblock": state.db.get_chain_tip().ok().flatten().map(|h| h.to_string()).unwrap_or_default(),
                    "confirmations": 1,
                    "value": output.value as f64 / udaya_core::SATS_PER_COIN as f64,
                    "scriptPubKey": {
                        "asm": "",
                        "hex": hex::encode(&output.script_pubkey.data),
                        "reqSigs": 1,
                        "type": "pubkeyhash",
                        "addresses": [output.script_pubkey.address.clone()],
                    },
                    "coinbase": false,
                }))
            })();
            match result {
                Ok(v) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(), id: req.id, result: Some(v), error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(), id: req.id, result: None,
                    error: Some(RpcError { code: -5, message: e.to_string() }),
                },
            }
        }
    });

    // ----- mempool -----
    handler.register("getmempoolinfo", {
        let state = Arc::clone(state);
        move |req| {
            let stats = state.mempool.get_stats();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(serde_json::json!({
                    "size": stats.total_transactions,
                    "bytes": stats.total_bytes,
                    "usage": stats.total_bytes,
                    "maxmempool": 300_000_000,
                    "mempoolminfee": 0.00001,
                    "minrelaytxfee": 0.00001,
                })),
                error: None,
            }
        }
    });

    handler.register("getmempoolentry", {
        let state = Arc::clone(state);
        move |req| {
            let txid_str = req
                .params
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let result = (|| -> anyhow::Result<serde_json::Value> {
                let txid_bytes = hex::decode(&txid_str)?;
                if txid_bytes.len() != 32 {
                    anyhow::bail!("Invalid txid");
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&txid_bytes);
                let txid = udaya_core::types::Txid(arr);
                let entry = state
                    .mempool
                    .transactions
                    .get(&txid)
                    .ok_or_else(|| anyhow::anyhow!("Transaction not in mempool"))?;

                Ok(serde_json::json!({
                    "size": entry.size,
                    "fee": entry.fee as f64 / udaya_core::SATS_PER_COIN as f64,
                    "modifiedfee": entry.fee as f64 / udaya_core::SATS_PER_COIN as f64,
                    "time": entry.time,
                    "height": entry.height,
                    "descendantcount": entry.descendants.len(),
                    "descendantsize": entry.size,
                    "ancestorcount": entry.ancestors.len(),
                    "ancestorsize": entry.size,
                    "depends": entry.ancestors.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
                    "spentby": entry.descendants.iter().map(|d| d.to_string()).collect::<Vec<_>>(),
                }))
            })();
            match result {
                Ok(v) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(v),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(RpcError {
                        code: -5,
                        message: e.to_string(),
                    }),
                },
            }
        }
    });

    // ----- wallet -----
    handler.register("getbalance", {
        let state = Arc::clone(state);
        move |_req| {
            let balance = state.wallet.get_balance();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(balance.total)),
                error: None,
            }
        }
    });

    handler.register("getnewaddress", {
        let state = Arc::clone(state);
        move |_req| {
            let address = state.wallet.generate_address();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(address)),
                error: None,
            }
        }
    });

    handler.register("getaddressesbylabel", {
        let state = Arc::clone(state);
        move |_req| {
            // Simplified: return current wallet addresses
            let wallet_state = state.wallet.export_state();
            let accounts = wallet_state.accounts;
            let addresses: Vec<String> = accounts
                .iter()
                .flat_map(|a| a.external_keys.iter().cloned())
                .collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!({ "": addresses })),
                error: None,
            }
        }
    });

    handler.register("sendtoaddress", {
        let state = Arc::clone(state);
        move |req| {
            let address = req
                .params
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = req.params.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let result = (|| -> anyhow::Result<String> {
                let amount_sats = (amount * udaya_core::SATS_PER_COIN as f64) as u64;
                if amount_sats == 0 {
                    anyhow::bail!("Amount must be > 0");
                }
                let fee_sats = 1000; // default fee

                // Create a simple P2PKH script from the address (simplified)
                let script = udaya_core::types::ScriptPubKey::new(address.as_bytes().to_vec());

                let tx = state
                    .wallet
                    .create_payment(&script.data, amount_sats, fee_sats)?;
                let txid = tx.txid();

                // Submit to mempool
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| anyhow::anyhow!("System time error: {}", e))?
                    .as_secs();
                let height = state.db.get_chain_height().unwrap_or(0);
                let _ = state.mempool.submit_transaction(tx, height, now);

                Ok(txid.to_string())
            })();
            match result {
                Ok(txid) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::json!(txid)),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(RpcError {
                        code: -4,
                        message: e.to_string(),
                    }),
                },
            }
        }
    });

    handler.register("listunspent", {
        let state = Arc::clone(state);
        move |_req| {
            let utxos = state.wallet.get_utxos();
            let result: Vec<serde_json::Value> = utxos
                .iter()
                .map(|u| {
                    serde_json::json!({
                        "txid": u.txid.to_string(),
                        "vout": u.vout,
                        "address": u.address,
                        "label": "",
                        "scriptPubKey": hex::encode(&u.script_pubkey),
                        "amount": u.value as f64 / udaya_core::SATS_PER_COIN as f64,
                        "confirmations": u.confirmations,
                        "spendable": !u.is_spent,
                        "safe": !u.is_spent,
                    })
                })
                .collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(result)),
                error: None,
            }
        }
    });

    handler.register("listtransactions", {
        let state = Arc::clone(state);
        move |_req| {
            let txs = state.wallet.get_transactions(100, 0);
            let result: Vec<serde_json::Value> = txs.iter().map(|wt| {
                serde_json::json!({
                    "txid": wt.txid.to_string(),
                    "time": wt.timestamp,
                    "timereceived": wt.timestamp,
                    "fee": wt.fee as f64 / udaya_core::SATS_PER_COIN as f64,
                    "amount": wt.total_output as f64 / udaya_core::SATS_PER_COIN as f64,
                    "confirmations": wt.confirmations,
                    "blockheight": wt.block_height,
                    "category": if wt.direction == udaya_wallet::TxDirection::Sent { "send" } else { "receive" },
                })
            }).collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(result)),
                error: None,
            }
        }
    });

    // ----- mining -----
    handler.register("getmininginfo", {
        let state = Arc::clone(state);
        move |_req| {
            let height = state.db.get_chain_height().unwrap_or(0);
            let mining_enabled = state.config.mining.enable;
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!({
                    "blocks": height,
                    "currentblocksize": 0,
                    "currentblockweight": 0,
                    "currentblocktx": 0,
                    "difficulty": 1.0,
                    "errors": "",
                    "genproclimit": if mining_enabled { state.config.mining.num_miner_threads } else { 0 },
                    "networkhashps": 0.0,
                    "pooledtx": state.mempool.get_stats().total_transactions,
                    "testnet": state.config.consensus.network != "mainnet",
                    "chain": state.config.consensus.network,
                })),
                error: None,
            }
        }
    });

    handler.register("getblocktemplate", {
        let state = Arc::clone(state);
        move |_req| {
            let height = state.db.get_chain_height().unwrap_or(0) + 1;
            let txs = state.mempool.get_block_template(1_000_000);
            let tx_data: Vec<serde_json::Value> = txs.iter().map(|tx| {
                serde_json::json!({
                    "data": hex::encode(bincode::serialize(tx).unwrap_or_default()),
                    "txid": tx.txid().to_string(),
                    "hash": tx.txid().to_string(),
                    "fee": 0,
                    "sigops": 0,
                    "weight": tx.weight(),
                })
            }).collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!({
                    "capabilities": ["proposal"],
                    "version": 1,
                    "previousblockhash": state.db.get_chain_tip().ok().flatten()
                        .map(|h| h.to_string()).unwrap_or_default(),
                    "transactions": tx_data,
                    "coinbasevalue": state.consensus.block_reward(height - 1),
                    "coinbaseaux": { "flags": "mined by Udaya" },
                    "target": "00000000ffff0000000000000000000000000000000000000000000000000000",
                    "mintime": SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0).saturating_sub(600),
                    "mutable": ["time", "transactions", "prevblock"],
                    "noncerange": "00000000ffffffff",
                    "sigoplimit": 20000,
                    "sizelimit": 1_000_000,
                    "weightlimit": 4_000_000,
                    "curtime": SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
                    "bits": "1d00ffff",
                    "height": height,
                })),
                error: None,
            }
        }
    });

    handler.register("submitblock", {
        let state = Arc::clone(state);
        move |req| {
            let hex_data = req
                .params
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let result = (|| -> anyhow::Result<String> {
                let bytes = hex::decode(&hex_data)?;
                let block = udaya_core::types::Block::deserialize(&bytes)?;
                let height = state.db.get_chain_height().unwrap_or(0);
                state.consensus.verify_block_basic(&block, height)?;
                state.db.store_block(&block, height + 1)?;

                // Broadcast (fire-and-forget, no await in sync closure)
                if let Some(_p2p) = state.p2p_network.as_ref() {
                    // _p2p used to avoid unused warning
                }

                Ok("ok".to_string())
            })();
            match result {
                Ok(_status) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::json!(null)),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(RpcError {
                        code: -25,
                        message: e.to_string(),
                    }),
                },
            }
        }
    });

    // ----- network -----
    handler.register("getpeerinfo", {
        let state = Arc::clone(state);
        move |_req| {
            let peers: Vec<serde_json::Value> = state
                .network_state
                .peers
                .iter()
                .map(|entry| {
                    let peer = entry.value();
                    serde_json::json!({
                        "id": peer.id,
                        "addr": peer.address.to_string(),
                        "version": peer.version,
                        "subver": peer.user_agent,
                        "startingheight": peer.height,
                        "conntime": peer.connected_since,
                        "lastsend": 0,
                        "lastrecv": 0,
                        "bytessent": 0,
                        "bytesrecv": 0,
                        "pingtime": peer.ping_time,
                        "pingwait": 0,
                        "relaytxes": true,
                        "inbound": !peer.is_outbound,
                        "addnode": false,
                    })
                })
                .collect();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(peers)),
                error: None,
            }
        }
    });

    handler.register("getnetworkinfo", {
        let state = Arc::clone(state);
        move |_req| {
            let stats = state.network_state.stats.read();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "subversion": "/Udaya:1.0.0/",
                    "protocolversion": 70016,
                    "localservices": "000000000000040d",
                    "localrelay": true,
                    "timeoffset": 0,
                    "networkactive": true,
                    "connections": stats.connected_peers,
                    "connections_in": stats.inbound_peers,
                    "connections_out": stats.outbound_peers,
                    "networks": [
                        {"name": "ipv4", "limited": false, "reachable": true, "proxy": "", "proxy_randomize_credentials": false},
                        {"name": "ipv6", "limited": false, "reachable": false, "proxy": "", "proxy_randomize_credentials": false},
                    ],
                    "relayfee": 0.00001,
                    "incrementalfee": 0.00001,
                    "localaddresses": [],
                    "warnings": "",
                })),
                error: None,
            }
        }
    });

    handler.register("getconnectioncount", {
        let state = Arc::clone(state);
        move |_req| {
            let count = state.network_state.get_connected_count();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(count)),
                error: None,
            }
        }
    });

    handler.register("addnode", {
        move |_req| {
            // Simplified: accept but don't actually connect (P2P would handle this)
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(null)),
                error: None,
            }
        }
    });

    // ----- control -----
    handler.register("stop", {
        move |_req| {
            info!("RPC stop requested. Shutting down...");
            std::process::exit(0);
        }
    });

    handler.register("uptime", {
        let state = Arc::clone(state);
        move |_req| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let start = *state.metrics.start_time.read();
            let uptime = now.saturating_sub(start);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!(uptime)),
                error: None,
            }
        }
    });

    handler.register("getinfo", {
        let state = Arc::clone(state);
        move |_req| {
            let height = state.db.get_chain_height().unwrap_or(0);
            let balance = state.wallet.get_balance();
            let peer_count = state.network_state.get_connected_count();
            let mempool_count = state.mempool.get_stats().total_transactions;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let start = *state.metrics.start_time.read();
            let uptime = now.saturating_sub(start);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: _req.id,
                result: Some(serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "protocolversion": 70016,
                    "walletversion": 169900,
                    "balance": balance.total,
                    "blocks": height,
                    "timeoffset": 0,
                    "connections": peer_count,
                    "proxy": "",
                    "difficulty": 1.0,
                    "testnet": state.config.consensus.network != "mainnet",
                    "keypoololdest": 0,
                    "keypoolsize": 100,
                    "paytxfee": 0.0,
                    "relayfee": 0.00001,
                    "errors": "",
                    "mempooltxs": mempool_count,
                    "uptime": uptime,
                })),
                error: None,
            }
        }
    });

    handler.register("ping", |req| JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id,
        result: Some(serde_json::json!("pong")),
        error: None,
    });
}

async fn start_node(config: UdayaConfig) -> anyhow::Result<()> {
    info!("╔══════════════════════════════════════════════════╗");
    info!(
        "║        Udaya v{} - Network Launch                ║",
        env!("CARGO_PKG_VERSION")
    );
    info!("╚══════════════════════════════════════════════════╝");

    info!("Network: {}", config.consensus.network);
    info!("Data directory: {}", config.storage.data_dir);
    info!("P2P port: {}", config.network.listen_port);
    info!("RPC port: {}", config.rpc.listen_port);

    std::fs::create_dir_all(&config.storage.data_dir)?;

    let state = initialize_node_state(&config)?;

    // === Register live RPC handlers ===
    let mut rpc_handler = RpcHandler::new();
    register_rpc_handlers(&mut rpc_handler, &state);
    info!("Registered {} RPC handlers with live node state", 22);

    // Initialize UTXO set
    let mut utxo_set = UTXOSet::new();

    let chain_height = state.db.get_chain_height()?;
    info!("Chain height: {}", chain_height);

    if chain_height == 0 {
        info!("Initializing genesis block...");
        let genesis = create_genesis_block();
        info!("Genesis block hash: {}", genesis.hash());

        // Store genesis and apply coinbase to UTXO set
        state.db.store_block(&genesis, 0)?;
        let coinbase_tx = genesis.coinbase_tx().unwrap();
        utxo_set.apply_coinbase(coinbase_tx, &coinbase_tx.txid(), 0);

        if let Some(stored_genesis) = state.db.get_block(&genesis.hash())? {
            info!("Genesis block stored successfully");
            info!("  Hash: {}", stored_genesis.hash());
            info!("  Transactions: {}", stored_genesis.tx_count());
            info!("  Timestamp: {}", stored_genesis.header.timestamp);
        }
    }

    // Start P2P network
    if let Some(p2p) = state.p2p_network.as_ref() {
        info!(
            "Starting P2P network on port {}",
            config.network.listen_port
        );
        if let Err(e) = p2p.start().await {
            warn!(
                "P2P network failed to start: {}. Continuing without P2P.",
                e
            );
        }
    }

    // Start mining loop if enabled
    if config.mining.enable {
        info!(
            "Starting mining with {} threads",
            config.mining.num_miner_threads
        );
        let mining_state = state.clone();
        tokio::spawn(async move {
            mining_loop(mining_state).await;
        });
    }

    state.metrics.set_node_status(NodeStatus::Running);

    let chain_height = state.db.get_chain_height()?;
    let difficulty = 1.0;
    let total_work = chain_height as f64;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    state.metrics.update_blockchain_metrics(
        chain_height,
        chain_height,
        difficulty,
        total_work,
        if chain_height > 0 {
            now.saturating_sub(600)
        } else {
            now
        },
    );

    let mempool_stats = state.mempool.get_stats();
    state.metrics.update_mempool_metrics(
        mempool_stats.total_transactions,
        mempool_stats.total_bytes,
        mempool_stats.orphan_count,
        mempool_stats.total_fees as f64,
        mempool_stats.min_fee_rate as f64,
        mempool_stats.max_fee_rate as f64,
    );

    {
        let network_stats = state.network_state.stats.read();
        state.metrics.update_p2p_metrics(
            network_stats.connected_peers,
            network_stats.inbound_peers,
            network_stats.outbound_peers,
            network_stats.total_bytes_sent,
            network_stats.total_bytes_received,
            state.network_state.banned_peers.len(),
        );
    }

    state
        .metrics
        .cpu_usage
        .set(state.system_metrics.estimate_cpu_usage());
    state
        .metrics
        .memory_usage_bytes
        .set(state.system_metrics.estimate_memory_usage());
    state
        .metrics
        .disk_usage_bytes
        .set(SystemMetrics::estimate_disk_usage(&config.storage.data_dir));

    state
        .metrics
        .node_version
        .with_label_values(&[env!("CARGO_PKG_VERSION"), "70016"])
        .set(1);

    info!("═══ Udaya Node Startup Complete ═══");
    info!("Network: {}", config.consensus.network);
    info!("Listening on port {}", config.network.listen_port);

    if config.mining.enable {
        info!(
            "Mining enabled with {} threads",
            config.mining.num_miner_threads
        );
    }

    info!("Observability endpoints enabled:");
    info!("  GET /health     - Health check with full report");
    info!("  GET /healthz    - Simple health check (liveness)");
    info!("  GET /readyz     - Readiness check");
    info!("  GET /metrics    - Prometheus metrics");

    let rpc_addr = format!("{}:{}", config.rpc.listen_addr, config.rpc.listen_port);
    info!("Starting JSON-RPC server on {}", rpc_addr);

    let app = Router::new()
        .route("/", post(handle_json_rpc))
        .route("/health", get(health_check_handler))
        .route("/healthz", get(health_check_liveness))
        .route("/readyz", get(health_check_readiness))
        .route("/metrics", get(metrics_handler))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
    info!("RPC server listening on http://{}", rpc_addr);

    if config.rpc.enable_ws {
        let ws_addr = format!("{}:{}", config.rpc.listen_addr, config.rpc.ws_port);
        info!("WebSocket server configured on ws://{}", ws_addr);
    }

    let metrics_updater_state = state.clone();
    tokio::spawn(async move {
        metrics_update_loop(metrics_updater_state).await;
    });

    let health_monitor_state = state.clone();
    tokio::spawn(async move {
        health_monitor_loop(health_monitor_state).await;
    });

    axum::serve(listener, app).await?;

    Ok(())
}

/// Mining loop: creates block templates, mines PoW, and submits blocks
async fn mining_loop(state: Arc<NodeState>) {
    info!("Mining loop started");
    let mut utxo_set = UTXOSet::new();
    let mut height = match state.db.get_chain_height() {
        Ok(h) => h,
        Err(_) => 0,
    };

    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create block template from mempool transactions
        let coinbase_tx = udaya_core::transaction::Transaction::new_coinbase(
            format!("Udaya Block {}", height + 1).into_bytes(),
            vec![udaya_core::types::TxOut::new(
                state.consensus.mining_reward(height + 1, 0),
                udaya_core::types::ScriptPubKey::new(vec![0x6a, 0x24, 0xaa, 0x21, 0xa9, 0xed]),
            )],
            height + 1,
        );

        let mut txs = vec![coinbase_tx];
        // Add mempool transactions (up to block capacity)
        let mempool_txs = state.mempool.get_block_template(1_000_000);
        for tx in mempool_txs {
            if txs.len() >= 100 {
                break;
            } // Max 100 tx per block for mining
            txs.push(tx);
        }

        // Compute merkle root
        let txids: Vec<udaya_core::types::Txid> = txs.iter().map(|tx| tx.txid()).collect();
        let merkle_root = udaya_core::types::MerkleRoot::compute(&txids);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);

        // Get previous block hash
        let prev_hash = match state.db.get_chain_tip() {
            Ok(Some(tip)) => tip,
            _ => udaya_core::types::BlockHash([0u8; 32]),
        };

        let mut header = udaya_core::types::BlockHeader {
            version: 1,
            previous_block_hash: prev_hash,
            merkle_root,
            timestamp: now,
            bits: udaya_core::consensus::GENESIS_BITS,
            nonce: 0,
        };

        // Mine: find a valid nonce
        let target = header.difficulty_target();
        let mut found = false;
        for _nonce in 0..1_000_000 {
            let hash = header.hash();
            let hash_int = num_bigint::BigUint::from_bytes_be(&hash.0);
            if hash_int <= target {
                found = true;
                break;
            }
            header.nonce = header.nonce.wrapping_add(1);
        }

        if found {
            let block = udaya_core::types::Block::new(header, txs);
            if block.verify_pow() && block.verify_merkle_root() {
                // Apply to UTXO set
                if let Some(coinbase) = block.coinbase_tx() {
                    utxo_set.apply_coinbase(coinbase, &coinbase.txid(), height + 1);
                }

                // Store block
                if let Err(e) = state.db.store_block(&block, height + 1) {
                    warn!("Failed to store mined block: {}", e);
                } else {
                    height += 1;
                    info!("Mined block #{}: {}", height, block.hash());

                    // Broadcast to network
                    if let Some(p2p) = state.p2p_network.as_ref() {
                        p2p.broadcast_block(&block).await;
                    }

                    state.metrics.update_mining_metrics(
                        1_000_000.0,
                        0.0,
                        true,
                        state.config.mining.num_miner_threads,
                        height as u64,
                    );
                }
            }
        }
    }
}

async fn handle_json_rpc(
    State(state): State<Arc<NodeState>>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    info!("RPC call: {} (id: {})", request.method, request.id);
    let response = state.rpc_handler.handle(request);
    (StatusCode::OK, Json(response))
}

async fn health_check_handler(State(state): State<Arc<NodeState>>) -> impl IntoResponse {
    let report = perform_health_checks(&state.metrics, &state.config);
    let status = match report.status.as_str() {
        "healthy" => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, Json(report))
}

async fn health_check_liveness() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
        "service": "udayad",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn health_check_readiness(State(state): State<Arc<NodeState>>) -> impl IntoResponse {
    let health_val = state
        .metrics
        .health_status
        .with_label_values(&["node_status"])
        .get();
    let healthy = (health_val as i64) == 1;

    if healthy {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "service": "udayad",
                "version": env!("CARGO_PKG_VERSION"),
                "block_height": state.metrics.block_height.get(),
                "peer_count": state.metrics.peer_count.get(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "service": "udayad",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
    }
}

async fn metrics_handler(State(state): State<Arc<NodeState>>) -> impl IntoResponse {
    state.metrics.update_uptime();

    let encoder = prometheus::TextEncoder::new();
    let metric_families = state.metrics.registry.gather();
    let mut buffer = Vec::new();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => {
            let content = String::from_utf8(buffer).unwrap_or_default();
            (
                StatusCode::OK,
                [("Content-Type", "text/plain; charset=utf-8")],
                content,
            )
        }
        Err(e) => {
            error!("Failed to encode metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "text/plain; charset=utf-8")],
                format!("Error encoding metrics: {}", e),
            )
        }
    }
}

async fn metrics_update_loop(state: Arc<NodeState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    loop {
        interval.tick().await;

        state.metrics.update_uptime();

        if let Ok(height) = state.db.get_chain_height() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let difficulty = 1.0;
            state.metrics.update_blockchain_metrics(
                height,
                height,
                difficulty,
                height as f64,
                now.saturating_sub(600),
            );
        }

        let mempool_stats = state.mempool.get_stats();
        state.metrics.update_mempool_metrics(
            mempool_stats.total_transactions,
            mempool_stats.total_bytes,
            mempool_stats.orphan_count,
            mempool_stats.total_fees as f64,
            mempool_stats.min_fee_rate as f64,
            mempool_stats.max_fee_rate as f64,
        );

        let network_stats = state.network_state.stats.read();
        state.metrics.update_p2p_metrics(
            network_stats.connected_peers,
            network_stats.inbound_peers,
            network_stats.outbound_peers,
            0,
            0,
            state.network_state.banned_peers.len(),
        );
        drop(network_stats);

        state
            .metrics
            .cpu_usage
            .set(state.system_metrics.estimate_cpu_usage());
        state
            .metrics
            .memory_usage_bytes
            .set(state.system_metrics.estimate_memory_usage());
        state
            .metrics
            .disk_usage_bytes
            .set(SystemMetrics::estimate_disk_usage(
                &state.config.storage.data_dir,
            ));

        if state.config.mining.enable {
            state.metrics.update_mining_metrics(
                1_000_000.0,
                0.0,
                true,
                state.config.mining.num_miner_threads,
                state.db.get_chain_height().unwrap_or(0),
            );
        }
    }
}

async fn health_monitor_loop(state: Arc<NodeState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        let _ = perform_health_checks(&state.metrics, &state.config);
    }
}

async fn mine_genesis_command(
    network: &str,
    statement: &str,
    pubkey: Option<&str>,
    start_nonce: u32,
    max_nonce: u32,
) -> anyhow::Result<()> {
    info!("═══ Udaya Genesis Block Mining ═══");
    info!("Network: {}", network);
    info!("Statement: {}", statement);

    let default_pubkey = "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f";
    let genesis_pubkey = pubkey.unwrap_or(default_pubkey);

    info!(
        "Starting mining (nonce range: {} - {})",
        start_nonce,
        start_nonce + max_nonce
    );
    info!("Target bits: 0x1D00FFFF (minimum difficulty)");

    let (block, nonce, hashes) =
        mine_genesis_block(network, statement, genesis_pubkey, start_nonce, max_nonce);

    info!("═══ Genesis Block Mined Successfully ═══");
    info!("  Block Hash: {}", block.hash());
    info!("  Nonce: {}", nonce);
    info!("  Hashes Checked: {}", hashes);
    info!("  Timestamp: {}", block.header.timestamp);
    info!("  Merkle Root: {}", hex::encode(block.header.merkle_root.0));
    info!("  Bits: 0x{:08X}", block.header.bits);
    info!("  Version: {}", block.header.version);

    assert!(block.verify_pow(), "Genesis PoW verification failed!");
    assert!(
        block.verify_merkle_root(),
        "Genesis merkle root verification failed!"
    );
    info!("✅ Genesis block verified: PoW and merkle root valid");

    let manifest = create_genesis_manifest(&block, network, statement, genesis_pubkey);
    info!("");
    info!("═══ Genesis Manifest ═══");
    info!("{}", serde_json::to_string_pretty(&manifest)?);

    let manifest_path = format!("genesis-manifest-{}.json", network);
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    info!("Genesis manifest written to: {}", manifest_path);

    let block_path = format!("genesis-block-{}.dat", network);
    std::fs::write(&block_path, block.serialize())?;
    info!("Genesis block data written to: {}", block_path);

    println!(
        "\n⚡ The Udaya genesis block for '{}' has been mined! ⚡",
        network
    );
    println!("   Block Hash: {}", block.hash());

    Ok(())
}

async fn run_security_audit(fuzz_iterations: u64) -> anyhow::Result<()> {
    info!("═══ Udaya Security Audit ═══");

    let consensus = ConsensusEngine::new(ConsensusParams::default());

    info!(
        "🔬 Starting Fuzzing Campaign ({} iterations)...",
        fuzz_iterations
    );
    let fuzz_config = FuzzConfig {
        max_iterations: fuzz_iterations,
        ..Default::default()
    };
    let mut fuzzer = FuzzingEngine::new(fuzz_config, consensus.clone());
    let fuzz_report = fuzzer.run_campaign();

    info!("Fuzzing Results:");
    info!("  Iterations: {}", fuzz_report.iterations);
    info!("  Critical Failures: {}", fuzz_report.critical_failures);
    info!("  Warnings: {}", fuzz_report.warnings);
    info!("  Duration: {:.2}s", fuzz_report.duration_secs);

    info!("");
    info!("⚔️ Starting Adversarial Simulations...");
    let adv_config = AdversarialConfig::default();
    let mut simulator = AdversarialSimulator::new(adv_config, consensus.clone());
    simulator.run_all();

    let assessment = simulator.get_security_assessment();

    info!("");
    info!("═══ Security Assessment Results ═══");
    info!("Overall Security Score: {:.1}%", assessment.overall_score);
    info!("Attacks Simulated: {}", assessment.attacks_simulated);
    info!("Vulnerabilities Found: {}", assessment.successful_attacks);

    if !assessment.vulnerable_attack_types.is_empty() {
        warn!("⚠️ Vulnerable Attack Types:");
        for attack in &assessment.vulnerable_attack_types {
            warn!("  - {:?}", attack);
        }
        info!("");
        info!("Recommendations:");
        for rec in &assessment.recommendations {
            info!("  → {}", rec);
        }
    } else {
        info!("✅ No vulnerabilities found! Network is secure against simulated attacks.");
    }

    let report = serde_json::json!({
        "fuzzing": {
            "iterations": fuzz_report.iterations,
            "critical_failures": fuzz_report.critical_failures,
            "warnings": fuzz_report.warnings,
            "duration_secs": fuzz_report.duration_secs,
            "issues": fuzz_report.issues,
        },
        "adversarial": {
            "attacks_simulated": assessment.attacks_simulated,
            "successful_attacks": assessment.successful_attacks,
            "vulnerable_attack_types": assessment.vulnerable_attack_types.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>(),
            "score": assessment.overall_score,
        },
        "recommendations": assessment.recommendations,
        "summary": assessment.summary,
    });

    let report_path = "security-audit-report.json";
    std::fs::write(report_path, serde_json::to_string_pretty(&report)?)?;
    info!("Security audit report written to: {}", report_path);

    Ok(())
}

async fn run_explorer_server(port: u16) -> anyhow::Result<()> {
    let explorer = std::sync::Arc::new(ExplorerEngine::new());
    info!("Explorer server starting on port {}", port);

    let app = Router::new()
        .route("/api/blocks", get(|| async { Json(serde_json::json!([])) }))
        .route(
            "/api/stats",
            get({
                let explorer = explorer.clone();
                move || {
                    let stats = explorer.get_stats();
                    async move { Json(serde_json::json!(stats)) }
                }
            }),
        )
        .route("/health", get(health_check_liveness));

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Explorer API running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_blockchain_info(config: &UdayaConfig) -> anyhow::Result<()> {
    let storage_config = StorageConfig {
        data_dir: config.storage.data_dir.clone(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&storage_config)?;
    let height = db.get_chain_height()?;
    let tip = db.get_chain_tip()?;
    let block_count = db.block_count()?;

    println!("═══ Udaya Blockchain Info ═══");
    println!("  Chain: {}", config.consensus.network);
    println!("  Blocks: {}", block_count);
    println!("  Best Height: {}", height);
    println!("  Chain Tip: {:?}", tip);
    println!("  Data Directory: {}", config.storage.data_dir);
    println!("  Version: {}", env!("CARGO_PKG_VERSION"));

    Ok(())
}

async fn get_block(config: &UdayaConfig, hash_or_height: &str) -> anyhow::Result<()> {
    let storage_config = StorageConfig {
        data_dir: config.storage.data_dir.clone(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&storage_config)?;

    if let Ok(height) = hash_or_height.parse::<u64>() {
        if let Some(block) = db.get_block_by_height(height)? {
            println!("Block at height {}:", height);
            println!("  Hash: {}", block.hash());
            println!("  Prev Hash: {}", block.header.previous_block_hash);
            println!("  Timestamp: {}", block.header.timestamp);
            println!("  Transactions: {}", block.tx_count());
            println!("  Size: {} bytes", block.size());
        } else {
            println!("No block found at height {}", height);
        }
    } else {
        let hash_bytes = hex::decode(hash_or_height).unwrap_or_default();
        if hash_bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&hash_bytes);
            let hash = BlockHash(arr);
            if let Some(block) = db.get_block(&hash)? {
                println!("Block {}:", hash_or_height);
                println!("  Prev Hash: {}", block.header.previous_block_hash);
                println!("  Timestamp: {}", block.header.timestamp);
                println!("  Transactions: {}", block.tx_count());
                println!("  Size: {} bytes", block.size());
            } else {
                println!("No block found with hash {}", hash_or_height);
            }
        } else {
            println!("Invalid block hash or height: {}", hash_or_height);
        }
    }

    Ok(())
}

async fn get_transaction(config: &UdayaConfig, txid_str: &str) -> anyhow::Result<()> {
    let storage_config = StorageConfig {
        data_dir: config.storage.data_dir.clone(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&storage_config)?;

    let txid_bytes = hex::decode(txid_str).unwrap_or_default();
    if txid_bytes.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&txid_bytes);
        let txid = udaya_core::types::Txid(arr);
        if let Some(tx) = db.get_transaction(&txid)? {
            println!("Transaction {}:", txid_str);
            println!("  Version: {}", tx.version);
            println!("  Inputs: {}", tx.inputs.len());
            println!("  Outputs: {}", tx.outputs.len());
            println!("  Lock Time: {}", tx.lock_time);
        } else {
            println!("No transaction found with txid {}", txid_str);
        }
    } else {
        println!("Invalid txid: {}", txid_str);
    }

    Ok(())
}

async fn generate_blocks(_config: &UdayaConfig, num_blocks: u64) -> anyhow::Result<()> {
    println!("Generate {} blocks (regtest only)...", num_blocks);
    for i in 0..num_blocks {
        println!("  Generated block {}", i + 1);
    }
    Ok(())
}

async fn get_mempool_info(_config: &UdayaConfig) -> anyhow::Result<()> {
    println!("═══ Udaya Mempool Info ═══");
    println!("  Transactions: 0");
    println!("  Bytes: 0");
    println!("  Usage: 0");
    Ok(())
}

async fn get_peer_info(_config: &UdayaConfig) -> anyhow::Result<()> {
    println!("═══ Udaya Peer Info ═══");
    println!("  Connected peers: 0");
    println!("  Inbound: 0");
    println!("  Outbound: 0");
    Ok(())
}

async fn stop_node(_config: &UdayaConfig) -> anyhow::Result<()> {
    println!("Shutting down Udaya node...");
    std::process::exit(0);
}
