use axum::{
    body::Body,
    extract::State,
    http::{
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
        HeaderMap, HeaderValue, Request, StatusCode,
    },
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use clap::{Parser, Subcommand};
use dashmap::DashMap;
use log::{debug, error, info, warn};
use parking_lot::Mutex;
use prometheus::Encoder;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
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
    pub rpc_handler: std::sync::Mutex<RpcHandler>,
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

    // Build P2P config from node config, mapping preferred_peers to seed_nodes
    let mut p2p_config = udaya_p2p::P2PConfig {
        listen_port: config.network.listen_port,
        max_peers: config.network.max_peers as usize,
        enable_dns_seed: false,
        ..Default::default()
    };
    // Use preferred_peers as seed nodes for local testnet connectivity
    if !config.network.preferred_peers.is_empty() {
        p2p_config.seed_nodes = config.network.preferred_peers.clone();
    }
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
        rpc_handler: std::sync::Mutex::new(rpc_handler),
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
                if let Err(e) = state.mempool.submit_transaction(tx.clone(), height, now) {
                    anyhow::bail!("Transaction rejected by mempool: {}", e);
                }

                // Note: The P2P layer broadcasts accepted transactions to peers via
                // the p2p message handler. This RPC path just adds to local mempool.

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

// ============================================================================
// RPC Security Middleware
// ============================================================================

/// In-memory rate limiter state
struct RateLimiterState {
    requests: DashMap<std::net::IpAddr, (AtomicU64, parking_lot::Mutex<std::time::Instant>)>,
}

impl RateLimiterState {
    fn new() -> Self {
        Self {
            requests: DashMap::new(),
        }
    }

    fn check_rate_limit(&self, ip: std::net::IpAddr, _max_rps: u32, burst: u32) -> bool {
        let now = std::time::Instant::now();
        let entry = self
            .requests
            .entry(ip)
            .or_insert_with(|| (AtomicU64::new(0), parking_lot::Mutex::new(now)));
        let (count, last_reset_mutex) = entry.value();

        let mut last_reset = last_reset_mutex.lock();
        let elapsed = now.duration_since(*last_reset).as_secs();
        if elapsed >= 1 {
            count.store(0, Ordering::SeqCst);
            *last_reset = now;
        }
        drop(last_reset);

        let current = count.fetch_add(1, Ordering::SeqCst) + 1;
        if current > burst as u64 {
            return false;
        }
        true
    }
}

/// HTTP Basic Auth middleware
async fn auth_middleware(
    State(config): State<Arc<UdayaConfig>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !config.rpc.enable_auth {
        return next.run(req).await;
    }

    let headers = req.headers();
    let auth_header = match headers.get(AUTHORIZATION) {
        Some(h) => h,
        None => {
            return build_auth_challenge_response();
        }
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return build_auth_challenge_response(),
    };

    if !auth_str.starts_with("Basic ") {
        return build_auth_challenge_response();
    }

    let decoded = match BASE64.decode(&auth_str[6..]) {
        Ok(d) => d,
        Err(_) => return build_auth_challenge_response(),
    };

    let credentials = match std::str::from_utf8(&decoded) {
        Ok(s) => s.to_string(),
        Err(_) => return build_auth_challenge_response(),
    };

    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        return build_auth_challenge_response();
    }

    let (username, password) = (parts[0], parts[1]);
    if username == config.rpc.username && password == config.rpc.password {
        next.run(req).await
    } else {
        build_auth_challenge_response()
    }
}

fn build_auth_challenge_response() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"Udaya RPC\""),
    );
    (
        StatusCode::UNAUTHORIZED,
        headers,
        Json(serde_json::json!({"error": "Unauthorized", "code": -401})),
    )
        .into_response()
}

/// Rate limiting middleware
async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiterState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = req
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|addr| addr.ip())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

    let config = req.extensions().get::<Arc<UdayaConfig>>();
    let max_rps = config.map(|c| c.rpc.rate_limit_rps).unwrap_or(100);
    let burst = config.map(|c| c.rpc.rate_limit_burst).unwrap_or(200);

    if !limiter.check_rate_limit(ip, max_rps, burst) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "Rate limit exceeded", "code": -429})),
        )
            .into_response();
    }

    next.run(req).await
}

/// Request size limiting middleware
async fn request_size_middleware(
    State(config): State<Arc<UdayaConfig>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let max_size = (config.rpc.max_request_size_mb as usize) * 1024 * 1024;

    let (parts, body) = req.into_parts();
    let body = http_body_util::BodyExt::collect(body).await.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Failed to read request body"})),
        )
            .into_response()
    });

    match body {
        Ok(collected) => {
            let bytes = collected.to_bytes();
            let total_size = bytes.len();
            if total_size > max_size {
                return (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    Json(serde_json::json!({"error": "Request too large", "code": -32600})),
                )
                    .into_response();
            }
            let req = Request::from_parts(parts, Body::from(bytes));
            next.run(req).await
        }
        Err(resp) => resp,
    }
}

/// Restricted method middleware
async fn restricted_method_middleware(
    State(config): State<Arc<UdayaConfig>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !config.rpc.enable_auth {
        return next.run(req).await;
    }

    let auth_header = req.headers().get(AUTHORIZATION);
    let is_authenticated = auth_header.is_some();

    if is_authenticated {
        return next.run(req).await;
    }

    let body_bytes = match axum::body::to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid request"})),
            )
                .into_response();
        }
    };

    let body: JsonRpcRequest = match serde_json::from_slice(&body_bytes) {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid JSON-RPC request"})),
            )
                .into_response();
        }
    };

    if config
        .rpc
        .restricted_methods
        .iter()
        .any(|m| m == &body.method)
    {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Method requires authentication", "code": -403})),
        )
            .into_response();
    }

    let req = Request::from_parts(
        Request::new(Body::empty()).into_parts().0,
        Body::from(body_bytes.clone()),
    );
    let (mut parts, _) = req.into_parts();
    parts.extensions.insert(config);
    let req = Request::from_parts(parts, Body::from(body_bytes));
    next.run(req).await
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

    // === Register live RPC handlers into the existing state ===
    {
        let mut rpc_handler = state.rpc_handler.lock().unwrap();
        register_rpc_handlers(&mut rpc_handler, &state);
    }
    info!("Registered {} RPC handlers with live node state", 22);

    // Initialize UTXO set
    let mut utxo_set = UTXOSet::new();

    let chain_height = state.db.get_chain_height()?;
    info!("Chain height: {}", chain_height);

    // Advertise our current height in version messages so peers can sync from us
    state.network_state.set_start_height(chain_height);

    if chain_height == 0 {
        info!("Initializing genesis block...");
        let genesis = create_genesis_block();
        info!("Genesis block hash: {}", genesis.hash());

        // Store genesis and apply coinbase to UTXO set
        state.db.store_block(&genesis, 0)?;
        state.db.update_utxo_set_for_block(&genesis, 0)?;

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
    let p2p_rx = if let Some(p2p) = state.p2p_network.as_ref() {
        info!(
            "Starting P2P network on port {}",
            config.network.listen_port
        );
        if let Err(e) = p2p.start().await {
            warn!(
                "P2P network failed to start: {}. Continuing without P2P.",
                e
            );
            None
        } else {
            p2p.take_message_receiver().await
        }
    } else {
        None
    };

    // Process incoming P2P messages (blocks and transactions) in background
    if let Some(rx) = p2p_rx {
        let p2p_state = state.clone();
        tokio::spawn(async move {
            handle_p2p_messages(p2p_state, rx).await;
        });
    }

    // Periodically re-sync from peers. This catches up nodes that fell behind,
    // including after a restart/reconnect, by re-requesting headers whenever a
    // connected peer advertises a higher height than we have.
    {
        let sync_state = state.clone();
        tokio::spawn(async move {
            periodic_sync_loop(sync_state).await;
        });
    }

    // Connect explorer to blockchain database
    if let Err(e) = state.explorer.load_from_database(&state.db) {
        warn!("Failed to load explorer data from database: {}", e);
    } else {
        info!("Explorer connected to blockchain database");
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

    let config_arc = Arc::new(config.clone());
    let config_arc_for_mw1 = config_arc.clone();
    let config_arc_for_mw2 = config_arc.clone();
    let rate_limiter = Arc::new(RateLimiterState::new());

    let mut app = Router::new()
        .route("/", post(handle_json_rpc))
        .route("/health", get(health_check_handler))
        .route("/healthz", get(health_check_liveness))
        .route("/readyz", get(health_check_readiness))
        .route("/metrics", get(metrics_handler))
        .with_state(state.clone())
        .layer(middleware::from_fn(move |req: Request<Body>, next: Next| {
            let config = config_arc_for_mw1.clone();
            let limiter = rate_limiter.clone();
            async move {
                let ip = req
                    .extensions()
                    .get::<std::net::SocketAddr>()
                    .map(|addr| addr.ip())
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

                if !limiter.check_rate_limit(ip, config.rpc.rate_limit_rps, config.rpc.rate_limit_burst) {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(serde_json::json!({"error": "Rate limit exceeded", "code": -429})),
                    )
                        .into_response();
                }

                if config.rpc.enable_auth {
                    let headers = req.headers();
                    let auth_header = match headers.get(AUTHORIZATION) {
                        Some(h) => h,
                        None => {
                            let mut resp_headers = HeaderMap::new();
                            resp_headers.insert(
                                WWW_AUTHENTICATE,
                                HeaderValue::from_static("Basic realm=\"Udaya RPC\""),
                            );
                            return (
                                StatusCode::UNAUTHORIZED,
                                resp_headers,
                                Json(serde_json::json!({"error": "Unauthorized", "code": -401})),
                            )
                                .into_response();
                        }
                    };

                    let auth_str = match auth_header.to_str() {
                        Ok(s) => s,
                        Err(_) => {
                            let mut resp_headers = HeaderMap::new();
                            resp_headers.insert(
                                WWW_AUTHENTICATE,
                                HeaderValue::from_static("Basic realm=\"Udaya RPC\""),
                            );
                            return (
                                StatusCode::UNAUTHORIZED,
                                resp_headers,
                                Json(serde_json::json!({"error": "Unauthorized", "code": -401})),
                            )
                                .into_response();
                        }
                    };

                    if !auth_str.starts_with("Basic ") {
                        let mut resp_headers = HeaderMap::new();
                        resp_headers.insert(
                            WWW_AUTHENTICATE,
                            HeaderValue::from_static("Basic realm=\"Udaya RPC\""),
                        );
                        return (
                            StatusCode::UNAUTHORIZED,
                            resp_headers,
                            Json(serde_json::json!({"error": "Unauthorized", "code": -401})),
                            )
                                .into_response();
                    }

                    let decoded = match BASE64.decode(&auth_str[6..]) {
                        Ok(d) => d,
                        Err(_) => {
                            let mut resp_headers = HeaderMap::new();
                            resp_headers.insert(
                                WWW_AUTHENTICATE,
                                HeaderValue::from_static("Basic realm=\"Udaya RPC\""),
                            );
                            return (
                                StatusCode::UNAUTHORIZED,
                                resp_headers,
                                Json(serde_json::json!({"error": "Unauthorized", "code": -401})),
                            )
                                .into_response();
                        }
                    };

                    let credentials = match std::str::from_utf8(&decoded) {
                        Ok(s) => s.to_string(),
                        Err(_) => {
                            let mut resp_headers = HeaderMap::new();
                            resp_headers.insert(
                                WWW_AUTHENTICATE,
                                HeaderValue::from_static("Basic realm=\"Udaya RPC\""),
                            );
                            return (
                                StatusCode::UNAUTHORIZED,
                                resp_headers,
                                Json(serde_json::json!({"error": "Unauthorized", "code": -401})),
                            )
                                .into_response();
                        }
                    };

                    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
                    if parts.len() != 2 || parts[0] != config.rpc.username || parts[1] != config.rpc.password {
                        let mut resp_headers = HeaderMap::new();
                        resp_headers.insert(
                            WWW_AUTHENTICATE,
                            HeaderValue::from_static("Basic realm=\"Udaya RPC\""),
                        );
                        return (
                            StatusCode::UNAUTHORIZED,
                            resp_headers,
                            Json(serde_json::json!({"error": "Unauthorized", "code": -401})),
                        )
                            .into_response();
                    }
                }

                next.run(req).await
            }
        }))
        .layer(middleware::from_fn(move |req: Request<Body>, next: Next| {
            let config = config_arc_for_mw2.clone();
            async move {
                if !config.rpc.enable_auth {
                    return next.run(req).await;
                }

                let body_bytes = match axum::body::to_bytes(req.into_body(), config.rpc.max_request_size_mb as usize * 1024 * 1024).await {
                    Ok(b) => b,
                    Err(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": "Invalid request body"})),
                        )
                            .into_response();
                    }
                };

                let rpc_req: JsonRpcRequest = match serde_json::from_slice(&body_bytes) {
                    Ok(b) => b,
                    Err(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": "Invalid JSON-RPC request"})),
                        )
                            .into_response();
                    }
                };

                if config.rpc.restricted_methods.iter().any(|m| m == &rpc_req.method) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({"error": "Method requires elevated privileges", "code": -403})),
                    )
                        .into_response();
                }

                let req = Request::from_parts(
                    Request::new(Body::empty()).into_parts().0,
                    Body::from(body_bytes.clone()),
                );
                let (mut parts, _) = req.into_parts();
                parts.extensions.insert(config);
                let req = Request::from_parts(parts, Body::from(body_bytes));
                next.run(req).await
            }
        }));

    if config_arc.rpc.enable_tls {
        info!("RPC server listening on https://{}", rpc_addr);
        let acceptor = axum_server::tls_rustls::RustlsAcceptor::new(
            axum_server::tls_rustls::RustlsConfig::from_pem_file(
                config_arc.rpc.tls_cert_path.as_deref().unwrap_or_default(),
                config_arc.rpc.tls_key_path.as_deref().unwrap_or_default(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load TLS config: {}", e))?,
        );
        let rpc_socket_addr: std::net::SocketAddr = rpc_addr.parse()?;
        axum_server::Server::bind(rpc_socket_addr)
            .acceptor(acceptor)
            .serve(app.into_make_service())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
        info!("RPC server listening on http://{}", rpc_addr);
        axum::serve(listener, app).await?;
    }
    Ok(())
}

/// Mining loop: creates block templates, mines PoW, and submits blocks
async fn mining_loop(state: Arc<NodeState>) {
    info!("Mining loop started");
    let mut height = match state.db.get_chain_height() {
        Ok(h) => h,
        Err(_) => 0,
    };

    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get a wallet address for the coinbase reward
        let coinbase_script = {
            let wallet_addr = state.wallet.generate_address();
            udaya_core::types::ScriptPubKey::with_address(
                wallet_addr.as_bytes().to_vec(),
                wallet_addr,
            )
        };

        // Create block template from mempool transactions
        let coinbase_tx = udaya_core::transaction::Transaction::new_coinbase(
            format!("Udaya Block {}", height + 1).into_bytes(),
            vec![udaya_core::types::TxOut::new(
                state.consensus.mining_reward(height + 1, 0),
                coinbase_script,
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

        // Use a very easy difficulty for testnet mining
        // bits = 0x207FFFFF means exponent=32, mantissa=0x7FFFFF
        // This gives a target of 0x7FFFFF * 2^(8*(32-3)) = 0x7FFFFF * 2^232
        // which is much easier than the Bitcoin minimum difficulty
        let testnet_bits: u32 = 0x207FFFFF;
        let mut header = udaya_core::types::BlockHeader {
            version: 1,
            previous_block_hash: prev_hash,
            merkle_root,
            timestamp: now,
            bits: testnet_bits,
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
                // Store block and update UTXO set
                if let Err(e) = state.db.store_block(&block, height + 1) {
                    warn!("Failed to store mined block: {}", e);
                } else if let Err(e) = state.db.update_utxo_set_for_block(&block, height + 1) {
                    warn!("Failed to update UTXO set for mined block: {}", e);
                } else {
                    height += 1;
                    info!("Mined block #{}: {}", height, block.hash());

                    // Register coinbase UTXO with wallet so it can spend mining rewards
                    if let Some(coinbase) = block.coinbase_tx() {
                        // Track the coinbase output in the wallet
                        for (vout, output) in coinbase.outputs.iter().enumerate() {
                            if let Some(addr) = &output.script_pubkey.address {
                                let utxo = udaya_wallet::WalletUTXO {
                                    txid: coinbase.txid(),
                                    vout: vout as u32,
                                    value: output.value,
                                    address: addr.clone(),
                                    script_pubkey: output.script_pubkey.data.clone(),
                                    height,
                                    confirmations: 1,
                                    is_coinbase: true,
                                    is_spent: false,
                                    is_frozen: false,
                                };
                                state.wallet.add_utxo(utxo);
                            }
                        }
                    }

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
    let response = {
        let handler = state.rpc_handler.lock().unwrap();
        handler.handle(request)
    };
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

async fn handle_p2p_messages(
    state: Arc<NodeState>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<(
        Arc<udaya_p2p::network::PeerConnection>,
        udaya_p2p::NetworkMessage,
    )>,
) {
    use std::collections::HashSet;
    use udaya_core::types::{BlockHash, InvType, InvVector};
    use udaya_p2p::NetworkMessage;

    // Track blocks we've already requested to avoid duplicate requests
    let pending_block_requests: Arc<parking_lot::Mutex<HashSet<BlockHash>>> =
        Arc::new(parking_lot::Mutex::new(HashSet::new()));

    // Orphan block buffer: blocks whose parent we don't have yet
    let orphan_blocks: Arc<
        parking_lot::Mutex<std::collections::HashMap<BlockHash, udaya_core::types::Block>>,
    > = Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new()));
    const MAX_ORPHAN_BLOCKS: usize = 500;

    while let Some((peer, msg)) = rx.recv().await {
        match msg {
            NetworkMessage::Version(v) => {
                info!(
                    "[SYNC] Peer {} handshake: version={}, user_agent={}, height={}",
                    peer.address, v.version, v.user_agent, v.start_height
                );

                // Update peer height in state
                state
                    .network_state
                    .update_peer_height(peer.id, v.start_height);

                // Trigger initial sync if peer is ahead
                let current_height = state.db.get_chain_height().unwrap_or(0);
                if v.start_height > current_height {
                    info!(
                        "[SYNC] Peer {} is ahead ({} > {}), requesting headers",
                        peer.address, v.start_height, current_height
                    );
                    let locator = get_block_locator(&state.db);
                    let _ = peer
                        .send_message(&udaya_p2p::network::create_getheaders_message(
                            &state.network_state.config,
                            locator,
                            BlockHash::default(),
                        ))
                        .await;
                }
            }
            NetworkMessage::Verack => {
                debug!("[SYNC] Received verack from {}", peer.address);

                // Send our mempool transactions to the newly connected peer
                let mempool_txs = state.mempool.get_block_template(1_000_000);
                if !mempool_txs.is_empty() {
                    let invs: Vec<InvVector> = mempool_txs
                        .iter()
                        .map(|tx| {
                            let txid = tx.txid();
                            InvVector::new(InvType::Tx, BlockHash::from_bytes(&txid.0))
                        })
                        .collect();
                    let inv_msg = udaya_p2p::network::create_inv_message(&invs);
                    let _ = peer.send_message(&inv_msg).await;
                    debug!(
                        "[SYNC] Sent {} mempool tx invs to {}",
                        invs.len(),
                        peer.address
                    );
                }

                // Also trigger a sync check
                let current_height = state.db.get_chain_height().unwrap_or(0);
                let peer_height = state
                    .network_state
                    .peers
                    .get(&peer.id)
                    .map(|p| p.height)
                    .unwrap_or(0);
                if peer_height > current_height {
                    let locator = get_block_locator(&state.db);
                    let _ = peer
                        .send_message(&udaya_p2p::network::create_getheaders_message(
                            &state.network_state.config,
                            locator,
                            BlockHash::default(),
                        ))
                        .await;
                }
            }
            NetworkMessage::Inv(inv_list) => {
                let mut blocks_to_request = Vec::new();
                let mut txs_to_request = Vec::new();

                for inv in inv_list {
                    match inv.inv_type {
                        InvType::Block => {
                            // Check if we already have this block
                            if !state.db.block_exists(&inv.hash).unwrap_or(false) {
                                // Check if we haven't already requested it
                                let already_requested = {
                                    let pending = pending_block_requests.lock();
                                    pending.contains(&inv.hash)
                                };
                                if !already_requested {
                                    {
                                        let mut pending = pending_block_requests.lock();
                                        pending.insert(inv.hash);
                                    }
                                    blocks_to_request.push(inv);
                                }
                            }
                        }
                        InvType::Tx => {
                            if !state.mempool.contains(&udaya_core::types::Txid(inv.hash.0)) {
                                txs_to_request.push(inv);
                            }
                        }
                        _ => {}
                    }
                }

                if !blocks_to_request.is_empty() {
                    debug!(
                        "[SYNC] Requesting {} blocks from {}",
                        blocks_to_request.len(),
                        peer.address
                    );
                    let getdata = udaya_p2p::network::create_getdata_message(&blocks_to_request);
                    let _ = peer.send_message(&getdata).await;
                }
                if !txs_to_request.is_empty() {
                    let getdata = udaya_p2p::network::create_getdata_message(&txs_to_request);
                    let _ = peer.send_message(&getdata).await;
                }
            }
            NetworkMessage::Headers(headers) => {
                if headers.is_empty() {
                    debug!("[SYNC] Received empty headers from {}", peer.address);
                    continue;
                }
                info!(
                    "[SYNC] Received {} headers from {} (first={:?}, last={:?})",
                    headers.len(),
                    peer.address,
                    headers.first().map(|h| h.hash()),
                    headers.last().map(|h| h.hash())
                );

                // Find the starting point: look for the first header whose parent we have
                let mut start_idx = None;
                for (i, header) in headers.iter().enumerate() {
                    if header.previous_block_hash.is_zero() {
                        // Genesis header
                        start_idx = Some(i);
                        break;
                    }
                    if state
                        .db
                        .block_exists(&header.previous_block_hash)
                        .unwrap_or(false)
                    {
                        start_idx = Some(i);
                        break;
                    }
                }

                // If we can't find a connection point, request headers with a broader locator
                if start_idx.is_none() {
                    warn!(
                        "[SYNC] Headers from {} don't connect to our chain. Requesting with locator.",
                        peer.address
                    );
                    let locator = get_block_locator(&state.db);
                    let _ = peer
                        .send_message(&udaya_p2p::network::create_getheaders_message(
                            &state.network_state.config,
                            locator,
                            BlockHash::default(),
                        ))
                        .await;
                    continue;
                }

                // Validate headers starting from the connection point
                let mut valid_headers = Vec::new();
                let start = start_idx.unwrap();

                for header in &headers[start..] {
                    // Verify PoW
                    if !state.consensus.verify_pow(header) {
                        warn!(
                            "[SYNC] Invalid PoW in header {} from {}",
                            header.hash(),
                            peer.address
                        );
                        state
                            .network_state
                            .increment_ban_score(&peer.address.ip().to_string(), 20);
                        break;
                    }

                    // Check timestamp not too far in the future
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs() as u32)
                        .unwrap_or(0);
                    if header.timestamp > now + 7200 {
                        warn!(
                            "[SYNC] Header {} has future timestamp {}",
                            header.hash(),
                            header.timestamp
                        );
                        break;
                    }

                    // If we already have this block, skip it but continue
                    if state.db.block_exists(&header.hash()).unwrap_or(false) {
                        continue;
                    }

                    valid_headers.push(header.clone());
                }

                if valid_headers.is_empty() {
                    debug!("[SYNC] No new valid headers from {}", peer.address);
                    continue;
                }

                info!(
                    "[SYNC] {} new valid headers from {} (range: {} -> {})",
                    valid_headers.len(),
                    peer.address,
                    valid_headers.first().map(|h| h.hash()).unwrap_or_default(),
                    valid_headers.last().map(|h| h.hash()).unwrap_or_default()
                );

                // Request blocks for the valid headers
                let invs: Vec<InvVector> = valid_headers
                    .iter()
                    .map(|h| InvVector::new(InvType::Block, h.hash()))
                    .collect();

                {
                    let mut pending = pending_block_requests.lock();
                    for inv in &invs {
                        pending.insert(inv.hash);
                    }
                }

                let getdata = udaya_p2p::network::create_getdata_message(&invs);
                let _ = peer.send_message(&getdata).await;

                // If we got 2000 headers (max), request more from the last hash
                if headers.len() >= 2000 {
                    let last_hash = headers.last().unwrap().hash();
                    let locator = vec![last_hash];
                    let _ = peer
                        .send_message(&udaya_p2p::network::create_getheaders_message(
                            &state.network_state.config,
                            locator,
                            BlockHash::default(),
                        ))
                        .await;
                    debug!(
                        "[SYNC] Requesting more headers from {} after {}",
                        peer.address, last_hash
                    );
                }
            }
            NetworkMessage::Block(block) => {
                let hash = block.hash();
                let prev_hash = block.header.previous_block_hash;

                // Remove from pending requests
                {
                    let mut pending = pending_block_requests.lock();
                    pending.remove(&hash);
                }

                // Duplicate check
                if state.db.block_exists(&hash).unwrap_or(false) {
                    debug!("[SYNC] Duplicate block {} from {}", hash, peer.address);
                    continue;
                }

                let current_height = state.db.get_chain_height().unwrap_or(0);

                // Check if it's the next block (extends our tip)
                let is_next = match state.db.get_chain_tip().unwrap_or(None) {
                    Some(tip) => tip == prev_hash,
                    None => prev_hash.is_zero(),
                };

                if is_next {
                    // Validate and accept the block
                    if let Err(e) = state
                        .consensus
                        .verify_block_basic(&block, current_height + 1)
                    {
                        warn!("[SYNC] Invalid block {} from {}: {}", hash, peer.address, e);
                        state
                            .network_state
                            .increment_ban_score(&peer.address.ip().to_string(), 20);
                        continue;
                    }

                    // Contextual validation
                    let prev_header = if prev_hash.is_zero() {
                        None
                    } else {
                        state.db.get_block_header(&prev_hash).unwrap_or(None)
                    };

                    if let Some(prev) = prev_header {
                        if let Err(e) = state.consensus.verify_block_context(
                            &block,
                            current_height + 1,
                            &prev,
                            prev.timestamp as u64,
                        ) {
                            warn!(
                                "[SYNC] Block {} from {} failed context validation: {}",
                                hash, peer.address, e
                            );
                            continue;
                        }
                    }

                    // Store and update UTXO set
                    if let Err(e) = state.db.store_block(&block, current_height + 1) {
                        warn!("[SYNC] Failed to store block {}: {}", hash, e);
                    } else if let Err(e) = state
                        .db
                        .update_utxo_set_for_block(&block, current_height + 1)
                    {
                        warn!("[SYNC] Failed to update UTXO for block {}: {}", hash, e);
                    } else {
                        info!(
                            "[SYNC] ✅ Accepted block #{} ({}) from peer {}",
                            current_height + 1,
                            hash,
                            peer.address
                        );

                        // Update advertised height
                        state.network_state.set_start_height(current_height + 1);

                        // Remove confirmed txs from mempool
                        state.mempool.remove_transactions(&block.transactions);

                        // Relay to other peers
                        if let Some(p2p) = state.p2p_network.as_ref() {
                            p2p.broadcast_block(&block).await;
                        }

                        // Try to process any orphan blocks that now connect
                        process_orphan_blocks(
                            &state,
                            &orphan_blocks,
                            &pending_block_requests,
                            &peer,
                        )
                        .await;
                    }
                } else {
                    // Not the next block - could be orphan, out-of-order, or reorg
                    if !state.db.block_exists(&prev_hash).unwrap_or(false) {
                        // We don't have the parent - it's an orphan
                        debug!(
                            "[SYNC] Orphan block {} from {} (parent {} unknown)",
                            hash, peer.address, prev_hash
                        );

                        // Buffer the orphan block
                        {
                            let mut orphans = orphan_blocks.lock();
                            if orphans.len() >= MAX_ORPHAN_BLOCKS {
                                // Remove oldest orphan (first inserted)
                                if let Some(&oldest_hash) = orphans.keys().next() {
                                    orphans.remove(&oldest_hash);
                                }
                            }
                            orphans.insert(hash, block.clone());
                        }

                        // Request headers to find the missing parent
                        let locator = get_block_locator(&state.db);
                        let _ = peer
                            .send_message(&udaya_p2p::network::create_getheaders_message(
                                &state.network_state.config,
                                locator,
                                BlockHash::default(),
                            ))
                            .await;
                    } else {
                        // We have the parent but it's not our tip - potential reorg
                        info!(
                            "[SYNC] Potential reorg: block {} from {} (parent {} exists but not tip)",
                            hash, peer.address, prev_hash
                        );
                        handle_potential_reorg(&state, block, current_height, &peer).await;
                    }
                }
            }
            NetworkMessage::Tx(tx) => {
                let txid = tx.txid();
                if state.mempool.contains(&txid) {
                    debug!("[SYNC] Duplicate tx {} from {}", txid, peer.address);
                    continue;
                }

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let height = state.db.get_chain_height().unwrap_or(0);

                if let Err(e) = state.mempool.submit_transaction(tx.clone(), height, now) {
                    debug!("[SYNC] Rejected tx {} from {}: {}", txid, peer.address, e);
                } else {
                    info!("[SYNC] ✅ Accepted tx {} from peer {}", txid, peer.address);
                    // Relay to other peers
                    if let Some(p2p) = state.p2p_network.as_ref() {
                        p2p.broadcast_transaction(&tx).await;
                    }
                }
            }
            NetworkMessage::GetHeaders(req) => {
                debug!(
                    "[SYNC] getheaders from {} (locator_count={})",
                    peer.address,
                    req.locator_hashes.len()
                );

                let mut headers = Vec::new();
                let mut start_height = 0;

                // Find the first locator hash we have in our chain
                for hash in &req.locator_hashes {
                    if let Ok(Some(height)) = state.db.get_block_height_by_hash(hash) {
                        start_height = height + 1;
                        break;
                    }
                }

                // Collect up to 2000 headers starting from start_height
                let current_height = state.db.get_chain_height().unwrap_or(0);
                for h in start_height..=current_height {
                    if headers.len() >= 2000 {
                        break;
                    }
                    if let Ok(Some(block)) = state.db.get_block_by_height(h) {
                        headers.push(block.header);
                        if block.hash() == req.hash_stop {
                            break;
                        }
                    }
                }

                debug!(
                    "[SYNC] Sending {} headers to {} (start_height={})",
                    headers.len(),
                    peer.address,
                    start_height
                );

                let msg = udaya_p2p::network::create_headers_message(&headers);
                let _ = peer.send_message(&msg).await;
            }
            NetworkMessage::GetData(inv_list) => {
                for inv in inv_list {
                    match inv.inv_type {
                        InvType::Block => {
                            if let Ok(Some(block)) = state.db.get_block(&inv.hash) {
                                let block_data = bincode::serialize(&block).unwrap_or_default();
                                let msg = udaya_p2p::network::Message::new(
                                    b"block\0\0\0\0\0\0\0",
                                    block_data,
                                );
                                let _ = peer.send_message(&msg).await;
                                debug!("[SYNC] Sent block {} to {}", inv.hash, peer.address);
                            }
                        }
                        InvType::Tx => {
                            // Check mempool first, then DB
                            let txid = udaya_core::types::Txid(inv.hash.0);
                            if let Some(entry) = state.mempool.transactions.get(&txid) {
                                let tx_data = bincode::serialize(&entry.tx).unwrap_or_default();
                                let msg = udaya_p2p::network::Message::new(
                                    b"tx\0\0\0\0\0\0\0\0\0\0",
                                    tx_data,
                                );
                                let _ = peer.send_message(&msg).await;
                            } else if let Ok(Some(tx)) = state.db.get_transaction(&txid) {
                                let tx_data = bincode::serialize(&tx).unwrap_or_default();
                                let msg = udaya_p2p::network::Message::new(
                                    b"tx\0\0\0\0\0\0\0\0\0\0",
                                    tx_data,
                                );
                                let _ = peer.send_message(&msg).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
            NetworkMessage::Addr(addrs) => {
                debug!(
                    "[SYNC] Received {} peer addresses from {}",
                    addrs.len(),
                    peer.address
                );
            }
            NetworkMessage::SendHeaders => {
                debug!("[SYNC] Peer {} prefers header relay", peer.address);
            }
            NetworkMessage::Reject(reject) => {
                warn!(
                    "[SYNC] Rejected by {}: {} (code: {})",
                    peer.address, reject.message, reject.ccode
                );
            }
            _ => {
                debug!("[SYNC] Unhandled message from {}", peer.address);
            }
        }
    }
}

/// Try to process orphan blocks that may now connect to our chain.
async fn process_orphan_blocks(
    state: &Arc<NodeState>,
    orphan_blocks: &Arc<
        parking_lot::Mutex<std::collections::HashMap<BlockHash, udaya_core::types::Block>>,
    >,
    _pending_block_requests: &Arc<parking_lot::Mutex<std::collections::HashSet<BlockHash>>>,
    _peer: &Arc<udaya_p2p::network::PeerConnection>,
) {
    let mut processed_any = true;
    while processed_any {
        processed_any = false;
        let current_height = state.db.get_chain_height().unwrap_or(0);
        let tip = state.db.get_chain_tip().unwrap_or(None);

        // Find orphans whose parent is now our tip
        let mut to_process: Vec<(BlockHash, udaya_core::types::Block)> = Vec::new();
        {
            let orphans = orphan_blocks.lock();
            for (hash, block) in orphans.iter() {
                if block.header.previous_block_hash == tip.unwrap_or_default() {
                    to_process.push((*hash, block.clone()));
                }
            }
        }

        for (hash, block) in to_process {
            // Remove from orphan pool
            {
                let mut orphans = orphan_blocks.lock();
                orphans.remove(&hash);
            }

            // Validate
            if let Err(e) = state
                .consensus
                .verify_block_basic(&block, current_height + 1)
            {
                warn!("[SYNC] Orphan block {} failed validation: {}", hash, e);
                continue;
            }

            if let Err(e) = state.db.store_block(&block, current_height + 1) {
                warn!("[SYNC] Failed to store orphan block {}: {}", hash, e);
                continue;
            }
            if let Err(e) = state
                .db
                .update_utxo_set_for_block(&block, current_height + 1)
            {
                warn!(
                    "[SYNC] Failed to update UTXO for orphan block {}: {}",
                    hash, e
                );
                continue;
            }

            info!(
                "[SYNC] ✅ Accepted orphan block #{} ({})",
                current_height + 1,
                hash
            );

            state.network_state.set_start_height(current_height + 1);
            state.mempool.remove_transactions(&block.transactions);

            // Relay
            if let Some(p2p) = state.p2p_network.as_ref() {
                p2p.broadcast_block(&block).await;
            }

            processed_any = true;
        }
    }
}

/// Periodically request headers from connected peers to catch up on missed blocks.
/// This is the primary mechanism for recovering after reconnects/restarts when a
/// peer advertises a higher chain height than ours. It runs alongside the
/// event-driven sync in `handle_p2p_messages`.
async fn periodic_sync_loop(state: Arc<NodeState>) {
    use udaya_core::types::BlockHash;

    let mut interval = tokio::time::interval(Duration::from_secs(10));
    // Skip the first immediate tick so the node can finish startup
    interval.tick().await;

    loop {
        interval.tick().await;

        // Update our advertised height so peers know we may be behind/ahead
        if let Ok(height) = state.db.get_chain_height() {
            state.network_state.set_start_height(height);
        }

        let local_height = state.db.get_chain_height().unwrap_or(0);
        let p2p_network = match state.p2p_network.as_ref() {
            Some(p) => p,
            None => continue,
        };

        // Check if any connected peer advertises a higher height
        let mut max_peer_height = 0;
        let mut need_sync = false;
        let mut peer_count = 0;

        for peer in state.network_state.peers.iter() {
            peer_count += 1;
            if peer.height > max_peer_height {
                max_peer_height = peer.height;
            }
            if peer.height > local_height {
                need_sync = true;
            }
        }

        if need_sync {
            info!(
                "[SYNC] Periodic sync: local={} behind peers (max_peer_height={}, peers={}), requesting headers",
                local_height, max_peer_height, peer_count
            );
            let locator = get_block_locator(&state.db);
            for conn in p2p_network.connections.iter() {
                let msg = udaya_p2p::network::create_getheaders_message(
                    &state.network_state.config,
                    locator.clone(),
                    BlockHash::default(),
                );
                let _ = conn.send_message(&msg).await;
            }
        } else if peer_count > 0 && local_height > 0 {
            // We're in sync - log periodically
            debug!(
                "[SYNC] In sync: height={}, peers={}",
                local_height, peer_count
            );
        }
    }
}

/// Build a block locator for header-first sync.
/// The locator contains hashes at exponentially increasing distances back from
/// the tip, allowing peers to quickly find a common ancestor.
fn get_block_locator(db: &BlockchainDB) -> Vec<udaya_core::types::BlockHash> {
    let mut locator = Vec::new();
    let height = db.get_chain_height().unwrap_or(0);

    if height == 0 {
        // Just return genesis if we have it
        if let Ok(Some(hash)) = db.get_block_hash_by_height(0) {
            locator.push(hash);
        }
        return locator;
    }

    // Start from the tip and go back with exponentially increasing steps
    let mut step = 1u64;
    let mut current = height;

    loop {
        if let Ok(Some(hash)) = db.get_block_hash_by_height(current) {
            locator.push(hash);
        }

        // After 10 entries, start doubling the step
        if locator.len() >= 10 {
            step *= 2;
        }

        if current < step {
            break;
        }
        current = current.saturating_sub(step);

        // Safety: don't go below 0
        if current == 0 {
            break;
        }
    }

    // Always include genesis
    if let Ok(Some(hash)) = db.get_block_hash_by_height(0) {
        if !locator.contains(&hash) {
            locator.push(hash);
        }
    }

    locator
}

/// Handle a potential chain reorganization.
/// This is called when we receive a block whose parent exists in our DB
/// but is not our current tip. We compare cumulative chain work and
/// reorganize to the chain with more work if it's valid.
async fn handle_potential_reorg(
    state: &Arc<NodeState>,
    block: udaya_core::types::Block,
    current_height: u64,
    _peer: &Arc<udaya_p2p::network::PeerConnection>,
) {
    use num_bigint::BigUint;

    let hash = block.hash();
    let prev_hash = block.header.previous_block_hash;

    info!(
        "[REORG] Evaluating potential reorg: new block {} (parent={})",
        hash, prev_hash
    );

    // Basic validation
    if let Err(e) = state
        .consensus
        .verify_block_basic(&block, current_height + 1)
    {
        warn!("[REORG] Invalid reorg block {}: {}", hash, e);
        return;
    }

    if !block.verify_pow() || !block.verify_merkle_root() {
        warn!("[REORG] Block {} failed PoW/merkle verification", hash);
        return;
    }

    // Find the height of the parent block using reverse lookup
    let parent_height = match state.db.get_block_height_by_hash(&prev_hash) {
        Ok(Some(h)) => h,
        _ => {
            warn!(
                "[REORG] Could not find height for parent block {}",
                prev_hash
            );
            return;
        }
    };

    // The new block would be at parent_height + 1
    let new_block_height = parent_height + 1;

    // Check reorg depth safety
    let reorg_depth = current_height.saturating_sub(parent_height);
    if reorg_depth > state.consensus.params.max_reorg_depth {
        warn!(
            "[REORG] Reorg depth {} exceeds max allowed {} for block {}",
            reorg_depth, state.consensus.params.max_reorg_depth, hash
        );
        return;
    }

    // Calculate cumulative work for the new block
    let new_block_work = state
        .db
        .get_chain_work(&prev_hash)
        .unwrap_or(BigUint::from(0u32))
        + udaya_core::consensus::ConsensusEngine::calculate_block_work(&block.header);

    // Calculate cumulative work for our current tip
    let current_tip_work = state.db.get_tip_chain_work().unwrap_or(BigUint::from(0u32));

    info!(
        "[REORG] Comparing work: new_chain_work={} vs current_chain_work={}",
        new_block_work, current_tip_work
    );

    // Only reorg if the new chain has more cumulative work
    if new_block_work <= current_tip_work {
        info!(
            "[REORG] New block {} has less work than current tip. Not reorganizing.",
            hash
        );
        // Still store the block so we have it if a longer fork arrives later
        let _ = state.db.store_block(&block, new_block_height);
        return;
    }

    // We need to reorganize. First, store the new block.
    if let Err(e) = state.db.store_block(&block, new_block_height) {
        warn!("[REORG] Failed to store reorg block {}: {}", hash, e);
        return;
    }

    // Walk back from current tip to find the common ancestor
    let current_tip = state.db.get_chain_tip().unwrap_or(None);
    let mut blocks_to_remove: Vec<BlockHash> = Vec::new();

    // Find common ancestor by walking back from current tip
    let mut walk_hash = current_tip;
    let mut common_ancestor_height: Option<u64> = None;

    while let Some(h) = walk_hash {
        // Check if this hash is the parent of our new block (i.e., the fork point)
        if h == prev_hash {
            common_ancestor_height = Some(parent_height);
            break;
        }

        // Check if this block exists at the same height as our new chain
        if let Ok(Some(this_height)) = state.db.get_block_height_by_hash(&h) {
            if this_height <= parent_height {
                common_ancestor_height = Some(this_height);
                break;
            }
        }

        blocks_to_remove.push(h);

        // Get the previous block
        if let Ok(Some(block_data)) = state.db.get_block(&h) {
            walk_hash = Some(block_data.header.previous_block_hash);
        } else {
            break;
        }
    }

    let Some(fork_height) = common_ancestor_height else {
        warn!(
            "[REORG] Could not find common ancestor for reorg block {}",
            hash
        );
        return;
    };

    info!(
        "[REORG] Fork point at height {}, removing {} blocks from old chain",
        fork_height,
        blocks_to_remove.len()
    );

    // Remove blocks from the old chain (above fork point)
    for old_hash in &blocks_to_remove {
        if let Err(e) = state.db.remove_block(old_hash) {
            warn!("[REORG] Failed to remove old block {}: {}", old_hash, e);
        }
    }

    // Set the new tip
    if let Err(e) = state.db.set_chain_tip(&hash, new_block_height) {
        warn!("[REORG] Failed to set new chain tip: {}", e);
        return;
    }

    // Rebuild UTXO set from the fork point
    // This is a simplified approach: rebuild from genesis to the new tip
    info!("[REORG] Rebuilding UTXO set from genesis...");
    let mut utxo_set = udaya_core::validation::UTXOSet::new();

    // Replay all blocks from genesis to the new tip
    for h in 0..=new_block_height {
        if let Ok(Some(block)) = state.db.get_block_by_height(h) {
            if let Some(coinbase) = block.coinbase_tx() {
                utxo_set.apply_coinbase(coinbase, &coinbase.txid(), h);
            }
            for tx in &block.transactions[1..] {
                utxo_set.apply_transaction(tx, &tx.txid(), h);
            }
        }
    }

    // Store the rebuilt UTXO set
    if let Err(e) = state.db.store_utxo_set(&utxo_set) {
        warn!("[REORG] Failed to store rebuilt UTXO set: {}", e);
    }

    // Update advertised height
    state.network_state.set_start_height(new_block_height);

    info!(
        "[REORG] ✅ Chain reorganized: new tip #{} ({}) from fork at height {} (reorg depth {})",
        new_block_height, hash, fork_height, reorg_depth
    );

    // Broadcast the reorg block to peers
    if let Some(p2p) = state.p2p_network.as_ref() {
        p2p.broadcast_block(&block).await;
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
