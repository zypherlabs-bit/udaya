use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Clone)]
#[command(name = "Udaya-pool")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "3333")]
    stratum_port: u16,
    #[arg(long, default_value = "9090")]
    api_port: u16,
    #[arg(long, default_value = "http://127.0.0.1:8332")]
    rpc_url: String,
    #[arg(long, default_value = "1.0")]
    fee_percent: f64,
    #[arg(long, default_value = "1.0")]
    min_payout: f64,
    #[arg(long)]
    pool_wallet: Option<String>,
}

type SharedMap<V> = Arc<tokio::sync::Mutex<HashMap<String, V>>>;

struct PoolState {
    config: Cli,
    miners: SharedMap<MinerInfo>,
    shares: SharedMap<ShareStats>,
    blocks: Arc<tokio::sync::Mutex<Vec<BlockFound>>>,
    start_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinerInfo {
    name: String,
    address: String,
    ip: String,
    connected_at: u64,
    last_share: u64,
    difficulty: f64,
    hashrate_ghs: f64,
}

#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
struct MinerStats {
    name: String,
    hashrate_ghs: f64,
    valid_shares: u64,
    invalid_shares: u64,
    last_share_ago: u64,
    connected_since: u64,
    estimated_daily_UDYA: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShareStats {
    valid: u64,
    invalid: u64,
    total: u64,
    last_hashrate: f64,
}

#[derive(Debug, Clone, Serialize)]
struct BlockFound {
    height: u64,
    hash: String,
    miner: String,
    reward: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
#[allow(non_snake_case)]
struct PoolInfo {
    name: String,
    hashrate_ghs: f64,
    miners_connected: u32,
    total_blocks: u64,
    total_shares: u64,
    valid_shares: u64,
    pool_fee_percent: f64,
    min_payout_UDYA: f64,
    uptime_hours: f64,
    estimated_daily_revenue_UDYA: f64,
    network: String,
    status: String,
}

fn default_miner(name: &str, ip: &str) -> MinerInfo {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    MinerInfo {
        name: name.to_string(),
        address: name.split('.').next().unwrap_or(name).to_string(),
        ip: ip.to_string(),
        connected_at: now,
        last_share: 0,
        difficulty: 1.0,
        hashrate_ghs: 0.0,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let config = Cli::parse();
    let state = Arc::new(PoolState {
        miners: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        shares: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        blocks: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        start_time: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        config: config.clone(),
    });

    println!("╔══════════════════════════════════════════════════╗");
    println!("║        Udaya Mining Pool v1.0.0                ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("API port: {}", config.api_port);
    println!("Stratum port: {}", config.stratum_port);
    println!("Pool fee: {}%", config.fee_percent);
    println!("RPC: {}", config.rpc_url);

    let stratum_state = state.clone();
    let stratum_port = config.stratum_port;
    tokio::spawn(async move {
        run_stratum_server(stratum_state, stratum_port).await;
    });

    let api_app = Router::new()
        .route("/api/pool/info", get(pool_info))
        .route("/api/pool/miners", get(pool_miners))
        .route("/api/pool/miner/{name}", get(miner_details))
        .route("/api/pool/blocks", get(pool_blocks))
        .route("/api/pool/submit", post(submit_share))
        .route("/health", get(health_check))
        .with_state(state.clone());

    let api_addr = format!("{}:{}", config.host, config.api_port);
    println!("Pool API listening on http://{}", api_addr);
    let listener = tokio::net::TcpListener::bind(&api_addr).await?;
    axum::serve(listener, api_app).await?;
    Ok(())
}

async fn run_stratum_server(state: Arc<PoolState>, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            log::info!("Stratum server listening on {}", addr);
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        log::info!("Stratum connection from {}", addr);
                        let s = state.clone();
                        tokio::spawn(async move {
                            handle_stratum_connection(stream, s).await;
                        });
                    }
                    Err(e) => log::error!("Stratum accept error: {}", e),
                }
            }
        }
        Err(e) => log::error!("Failed to bind stratum port {}: {}", port, e),
    }
}

async fn handle_stratum_connection(mut stream: tokio::net::TcpStream, state: Arc<PoolState>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut buf = vec![0u8; 4096];
    let mut buffer = String::new();
    let peer_addr = stream
        .peer_addr()
        .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
    let mut worker_name = String::from("anonymous");
    let mut subscribed = false;
    let mut authorized = false;

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                buffer.push_str(&String::from_utf8_lossy(&buf[..n]));
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                        let method = msg
                            .get("method")
                            .and_then(|m| m.as_str())
                            .unwrap_or("")
                            .to_string();
                        let id = msg.get("id").cloned();

                        if method == "mining.subscribe" {
                            subscribed = true;
                            let response = serde_json::json!({
                                "id": id, "result": [[["mining.notify", "sid1"], ["mining.set_difficulty", "1"]], "08000002", 4], "error": null
                            });
                            let resp_str = serde_json::to_string(&response).unwrap();
                            let _ = stream.write_all(format!("{}\n", resp_str).as_bytes()).await;
                            log::info!("Stratum: subscribed client from {}", peer_addr);
                        } else if method == "mining.authorize" {
                            if !subscribed {
                                let err = serde_json::json!({"id": id, "result": null, "error": [24, "Not subscribed", null]});
                                let _ = stream
                                    .write_all(
                                        format!("{}\n", serde_json::to_string(&err).unwrap())
                                            .as_bytes(),
                                    )
                                    .await;
                                continue;
                            }
                            if let Some(params) = msg.get("params").and_then(|p| p.as_array()) {
                                if let Some(name) = params.get(0).and_then(|p| p.as_str()) {
                                    worker_name = name.to_string();
                                    authorized = true;
                                    let mut miners = state.miners.lock().await;
                                    miners.insert(
                                        worker_name.clone(),
                                        default_miner(&worker_name, &peer_addr.ip().to_string()),
                                    );
                                    drop(miners);
                                    let response = serde_json::json!({"id": id, "result": true, "error": null});
                                    let _ = stream
                                        .write_all(
                                            format!(
                                                "{}\n",
                                                serde_json::to_string(&response).unwrap()
                                            )
                                            .as_bytes(),
                                        )
                                        .await;
                                    let diff = serde_json::json!({"id": null, "method": "mining.set_difficulty", "params": [1.0]});
                                    let _ = stream
                                        .write_all(
                                            format!("{}\n", serde_json::to_string(&diff).unwrap())
                                                .as_bytes(),
                                        )
                                        .await;
                                    log::info!(
                                        "Stratum: authorized miner '{}' from {}",
                                        worker_name,
                                        peer_addr
                                    );
                                }
                            }
                        } else if method == "mining.submit" {
                            if !authorized {
                                let err = serde_json::json!({"id": id, "result": null, "error": [24, "Unauthorized", null]});
                                let _ = stream
                                    .write_all(
                                        format!("{}\n", serde_json::to_string(&err).unwrap())
                                            .as_bytes(),
                                    )
                                    .await;
                                continue;
                            }
                            let hashrate = 100.0 + rand::thread_rng().gen::<f64>() * 50.0;
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            {
                                let mut shares = state.shares.lock().await;
                                let stats =
                                    shares.entry(worker_name.clone()).or_insert(ShareStats {
                                        valid: 0,
                                        invalid: 0,
                                        total: 0,
                                        last_hashrate: 0.0,
                                    });
                                stats.valid += 1;
                                stats.total += 1;
                                stats.last_hashrate = hashrate;
                            }
                            {
                                let mut miners = state.miners.lock().await;
                                if let Some(miner) = miners.get_mut(&worker_name) {
                                    miner.last_share = now;
                                    miner.hashrate_ghs = hashrate;
                                }
                            }
                            let response =
                                serde_json::json!({"id": id, "result": true, "error": null});
                            let _ = stream
                                .write_all(
                                    format!("{}\n", serde_json::to_string(&response).unwrap())
                                        .as_bytes(),
                                )
                                .await;
                        } else {
                            log::debug!("Stratum: unknown method '{}' from {}", method, peer_addr);
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!("Stratum read error from {}: {}", peer_addr, e);
                break;
            }
        }
    }
    log::info!("Stratum: {} disconnected", worker_name);
    state.miners.lock().await.remove(&worker_name);
}

async fn pool_info(State(state): State<Arc<PoolState>>) -> Json<PoolInfo> {
    let miners = state.miners.lock().await;
    let shares = state.shares.lock().await;
    let blocks = state.blocks.lock().await;
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - state.start_time;
    let total_hashrate: f64 = miners.values().map(|m| m.hashrate_ghs).sum();
    let total_valid: u64 = shares.values().map(|s| s.valid).sum();
    let total_shares: u64 = shares.values().map(|s| s.total).sum();
    let estimated_daily = if total_hashrate > 0.0 {
        let network_hashrate = 100000.0;
        let share = total_hashrate / network_hashrate;
        share * 144.0 * 50.0 * (1.0 - state.config.fee_percent / 100.0)
    } else {
        0.0
    };
    Json(PoolInfo {
        name: "Udaya Pool".to_string(),
        hashrate_ghs: total_hashrate,
        miners_connected: miners.len() as u32,
        total_blocks: blocks.len() as u64,
        total_shares,
        valid_shares: total_valid,
        pool_fee_percent: state.config.fee_percent,
        min_payout_UDYA: state.config.min_payout,
        uptime_hours: uptime as f64 / 3600.0,
        estimated_daily_revenue_UDYA: estimated_daily,
        network: "testnet".to_string(),
        status: "active".to_string(),
    })
}

async fn pool_miners(State(state): State<Arc<PoolState>>) -> Json<Vec<MinerStats>> {
    let miners = state.miners.lock().await;
    let shares = state.shares.lock().await;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let stats: Vec<MinerStats> = miners
        .iter()
        .map(|(name, info)| {
            let share_stats = shares.get(name);
            MinerStats {
                name: name.clone(),
                hashrate_ghs: info.hashrate_ghs,
                valid_shares: share_stats.map(|s| s.valid).unwrap_or(0),
                invalid_shares: share_stats.map(|s| s.invalid).unwrap_or(0),
                last_share_ago: if info.last_share > 0 {
                    now - info.last_share
                } else {
                    0
                },
                connected_since: now - info.connected_at,
                estimated_daily_UDYA: info.hashrate_ghs * 0.00001,
            }
        })
        .collect();
    drop(miners);
    drop(shares);
    Json(stats)
}

async fn miner_details(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(state): State<Arc<PoolState>>,
) -> Json<serde_json::Value> {
    let miners = state.miners.lock().await;
    let shares = state.shares.lock().await;
    let result = if let Some(miner) = miners.get(&name) {
        let share_stats = shares.get(&name);
        serde_json::json!({
            "name": name, "address": miner.address, "ip": miner.ip,
            "hashrate_ghs": miner.hashrate_ghs, "difficulty": miner.difficulty,
            "connected_at": miner.connected_at, "last_share": miner.last_share,
            "shares": { "valid": share_stats.map(|s| s.valid).unwrap_or(0), "invalid": share_stats.map(|s| s.invalid).unwrap_or(0), "total": share_stats.map(|s| s.total).unwrap_or(0) }
        })
    } else {
        serde_json::json!({"error": "Miner not found"})
    };
    drop(miners);
    drop(shares);
    Json(result)
}

async fn pool_blocks(State(state): State<Arc<PoolState>>) -> Json<Vec<BlockFound>> {
    let blocks = state.blocks.lock().await;
    let result = blocks.clone();
    drop(blocks);
    Json(result)
}

async fn submit_share(
    State(state): State<Arc<PoolState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let worker = body
        .get("worker")
        .and_then(|w| w.as_str())
        .unwrap_or("unknown")
        .to_string();
    let mut shares = state.shares.lock().await;
    let stats = shares.entry(worker.clone()).or_insert(ShareStats {
        valid: 0,
        invalid: 0,
        total: 0,
        last_hashrate: 0.0,
    });
    stats.valid += 1;
    stats.total += 1;
    Json(serde_json::json!({"status": "accepted", "worker": worker}))
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "Udaya-pool", "version": "1.0.0"}))
}
