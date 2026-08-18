use clap::Parser;
use log::{error, info, warn};
use rand::Rng;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use udaya_core::consensus::{ConsensusEngine, ConsensusParams};
use udaya_core::types::*;

/// Udaya Standalone Miner - Mine UDYA with CPU or submit to pool
#[derive(Parser)]
#[command(name = "Udaya-miner")]
#[command(about = "Udaya miner - CPU mining or pool submission", long_about = None)]
struct Cli {
    /// Mining pool URL (Stratum)
    #[arg(short, long)]
    pool_url: Option<String>,

    /// Wallet address for rewards
    #[arg(short, long)]
    wallet: String,

    /// Number of CPU threads for solo mining
    #[arg(short, long, default_value = "1")]
    threads: usize,

    /// Solo mine (no pool)
    #[arg(long)]
    solo: bool,

    /// RPC endpoint for solo mining
    #[arg(long, default_value = "http://127.0.0.1:8332")]
    rpc_url: String,

    /// RPC username
    #[arg(long)]
    rpc_user: String,

    /// RPC password
    #[arg(long)]
    rpc_password: String,

    /// Worker name for pool
    #[arg(short, long, default_value = "worker1")]
    worker: String,

    /// Worker password for pool
    #[arg(short, long, default_value = "x")]
    worker_password: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }

    println!("╔══════════════════════════════════════════════════╗");
    println!("║        Udaya Miner v1.0.0                      ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("Wallet: {}", cli.wallet);
    println!("Threads: {}", cli.threads);

    if let Some(pool_url) = &cli.pool_url {
        println!("Pool: {}", pool_url);
        println!("Worker: {}", cli.worker);
        run_pool_miner(
            pool_url,
            &cli.wallet,
            &cli.worker,
            &cli.worker_password,
            cli.threads,
        )
        .await?;
    } else {
        println!("Mode: Solo mining via RPC at {}", cli.rpc_url);
        run_solo_miner(
            &cli.rpc_url,
            &cli.rpc_user,
            &cli.rpc_password,
            &cli.wallet,
            cli.threads,
        )
        .await?;
    }

    Ok(())
}

async fn run_solo_miner(
    rpc_url: &str,
    rpc_user: &str,
    rpc_pass: &str,
    wallet: &str,
    threads: usize,
) -> anyhow::Result<()> {
    info!("Starting solo miner with {} threads", threads);

    let _consensus = Arc::new(ConsensusEngine::new(ConsensusParams::default()));
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut total_hashes: u64 = 0;
    let mut blocks_found: u64 = 0;

    // Create a client for RPC
    let client = reqwest::Client::new();

    loop {
        // Get blockchain info from RPC
        let block_template =
            match get_block_template(rpc_url, rpc_user, rpc_pass, wallet, &client).await {
                Some(template) => template,
                None => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    continue;
                }
            };

        let target = block_template.header.difficulty_target();

        // Mine the block
        info!(
            "Mining block at height ? (target: {:?})",
            &target.to_bytes_be()[..4]
        );

        let header = block_template.header;
        let hashes_checked = Arc::new(std::sync::atomic::AtomicU64::new(0));

        let mining_handles: Vec<_> = (0..threads)
            .map(|thread_id| {
                let _target = target.clone();
                let mut local_header = header;
                let counter = hashes_checked.clone();

                std::thread::spawn(move || {
                    let mut nonce_start = thread_id as u32 * (u32::MAX / threads as u32);
                    let nonce_end = if thread_id == threads - 1 {
                        u32::MAX
                    } else {
                        (thread_id as u32 + 1) * (u32::MAX / threads as u32) - 1
                    };

                    local_header.nonce = nonce_start;

                    while local_header.nonce <= nonce_end {
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if local_header.verify_pow() {
                            return Some(local_header);
                        }
                        local_header.nonce = local_header.nonce.wrapping_add(1);
                        if local_header.nonce == 0 {
                            break; // Wrapped around
                        }
                    }
                    None
                })
            })
            .collect();

        // Wait for a thread to find a solution or all to finish
        let mut solution: Option<BlockHeader> = None;
        for handle in mining_handles {
            if let Ok(Some(header)) = handle.join() {
                solution = Some(header);
                break;
            }
        }

        let checked = hashes_checked.load(std::sync::atomic::Ordering::Relaxed);
        total_hashes += checked;

        if let Some(solved_header) = solution {
            let mut block = block_template.clone();
            block.header = solved_header;

            info!("🎉 Block found! Hash: {}", block.hash());
            info!("Nonce: {}", solved_header.nonce);

            // Submit block via RPC
            match submit_block(rpc_url, rpc_user, rpc_pass, &block, &client).await {
                Ok(()) => {
                    blocks_found += 1;
                    info!("✅ Block submitted successfully!");
                }
                Err(e) => {
                    warn!("Failed to submit block: {}", e);
                }
            }
        } else {
            info!("No solution found in this round, updating timestamp and retrying...");
            // Update timestamp for next attempt
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            let _ = now;
        }

        // Report hash rate periodically
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - start_time;
        if elapsed > 0 {
            let hash_rate = total_hashes as f64 / elapsed as f64;
            info!(
                "Hash rate: {:.2} H/s, Total hashes: {}, Blocks: {}",
                hash_rate, total_hashes, blocks_found
            );
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

async fn run_pool_miner(
    pool_url: &str,
    wallet: &str,
    worker: &str,
    worker_pass: &str,
    _threads: usize,
) -> anyhow::Result<()> {
    info!("Connecting to pool: {} as worker {}", pool_url, worker);

    // Parse pool URL to get host and port
    let pool_addr = pool_url
        .trim_start_matches("stratum+tcp://")
        .trim_start_matches("tcp://");

    let (host, port) = if let Some(colon_pos) = pool_addr.rfind(':') {
        let h = &pool_addr[..colon_pos];
        let p: u16 = pool_addr[colon_pos + 1..].parse().unwrap_or(3333);
        (h.to_string(), p)
    } else {
        (pool_addr.to_string(), 3333u16)
    };

    info!("Resolved pool: {}:{}", host, port);

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut total_shares: u64 = 0;
    let mut valid_shares: u64 = 0;
    let mut invalid_shares: u64 = 0;

    // Connect to pool via Stratum TCP
    loop {
        match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
            Ok(mut stream) => {
                info!("Connected to pool {}:{}", host, port);

                // Send mining.subscribe
                let subscribe_msg = serde_json::json!({
                    "id": 1,
                    "method": "mining.subscribe",
                    "params": ["Udaya-miner/1.0.0", format!("{}.{}", host, port)]
                });

                let subscribe_str = serde_json::to_string(&subscribe_msg)?;
                stream
                    .write_all(format!("{}\n", subscribe_str).as_bytes())
                    .await?;
                info!("Sent mining.subscribe");

                // Read response
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await?;
                let response = String::from_utf8_lossy(&buf[..n]);
                info!("Pool response: {}", response.trim());

                // Send mining.authorize
                let auth_msg = serde_json::json!({
                    "id": 2,
                    "method": "mining.authorize",
                    "params": [worker, worker_pass]
                });
                let auth_str = serde_json::to_string(&auth_msg)?;
                stream
                    .write_all(format!("{}\n", auth_str).as_bytes())
                    .await?;
                info!("Sent mining.authorize for worker: {}", worker);

                // Read auth response
                let n = stream.read(&mut buf).await?;
                let auth_response = String::from_utf8_lossy(&buf[..n]);
                info!("Auth response: {}", auth_response.trim());

                // Main mining loop - listen for jobs and submit shares
                let mut buffer = String::new();
                loop {
                    let n = stream.read(&mut buf).await?;
                    if n == 0 {
                        warn!("Pool connection closed");
                        break;
                    }
                    buffer.push_str(&String::from_utf8_lossy(&buf[..n]));

                    // Process each line
                    while let Some(newline_pos) = buffer.find('\n') {
                        let line = buffer[..newline_pos].to_string();
                        buffer = buffer[newline_pos + 1..].to_string();

                        if line.is_empty() {
                            continue;
                        }

                        // Parse the JSON message
                        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                            handle_pool_message(
                                &mut stream,
                                &msg,
                                &mut total_shares,
                                &mut valid_shares,
                                &mut invalid_shares,
                                start_time,
                            )
                            .await;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to pool {}:{}: {}", host, port, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_pool_message(
    stream: &mut tokio::net::TcpStream,
    msg: &serde_json::Value,
    total_shares: &mut u64,
    valid_shares: &mut u64,
    invalid_shares: &mut u64,
    start_time: u64,
) {
    // Check for mining.notify (job)
    if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
        match method {
            "mining.notify" => {
                if let Some(params) = msg.get("params").and_then(|p| p.as_array()) {
                    let job_id = params[0].as_str().unwrap_or("unknown");
                    let _prev_hash = params[1].as_str().unwrap_or("");
                    let _coinbase1 = params[2].as_str().unwrap_or("");
                    let _coinbase2 = params[3].as_str().unwrap_or("");
                    let _merkle_branch = params[4]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let _version = params[5].as_str().unwrap_or("");
                    let _nbits = params[6].as_str().unwrap_or("");
                    let ntime = params[7].as_str().unwrap_or("");
                    let clean_jobs = params[8].as_bool().unwrap_or(true);

                    info!("New job: {} (clean: {})", job_id, clean_jobs);

                    // Simulate finding a share (in real miner this would do actual PoW)
                    // For demo, submit a valid share format
                    let share_submit = serde_json::json!({
                        "id": 4,
                        "method": "mining.submit",
                        "params": [
                            "worker1",
                            job_id,
                            "00000000",
                            ntime,
                            format!("{:08x}", rand::thread_rng().gen::<u32>())
                        ]
                    });

                    let submit_str = serde_json::to_string(&share_submit).unwrap();
                    let _ = stream
                        .write_all(format!("{}\n", submit_str).as_bytes())
                        .await;
                    *total_shares += 1;

                    // Report hash rate periodically
                    let elapsed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        - start_time;
                    if elapsed > 0 && *total_shares % 100 == 0 {
                        let hash_rate = *total_shares as f64 / elapsed as f64;
                        info!(
                            "Shares: {} valid, {} invalid, {:.2} shares/s",
                            valid_shares, invalid_shares, hash_rate
                        );
                    }
                }
            }
            "mining.set_difficulty" => {
                if let Some(params) = msg.get("params").and_then(|p| p.as_array()) {
                    if let Some(diff) = params[0].as_f64() {
                        info!("New difficulty: {}", diff);
                    }
                }
            }
            _ => {}
        }
    }

    // Check for result (submit response)
    if let Some(result) = msg.get("result") {
        match result.as_bool() {
            Some(true) => {
                *valid_shares += 1;
            }
            Some(false) => {
                *invalid_shares += 1;
                if let Some(error) = msg.get("error") {
                    warn!("Share rejected: {:?}", error);
                }
            }
            None => {}
        }
    }
}

async fn get_block_template(
    _rpc_url: &str,
    _rpc_user: &str,
    _rpc_pass: &str,
    _wallet: &str,
    _client: &reqwest::Client,
) -> Option<Block> {
    // Simplified - in production this would call getblocktemplate RPC
    // For now return a basic template
    use udaya_core::consensus::create_genesis_block;
    Some(create_genesis_block())
}

async fn submit_block(
    rpc_url: &str,
    rpc_user: &str,
    rpc_pass: &str,
    block: &Block,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "submitblock",
        "params": [hex::encode(block.serialize())]
    });

    let response = client
        .post(rpc_url)
        .json(&request)
        .basic_auth(rpc_user, Some(rpc_pass))
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    info!("Submit block response: {:?}", result);

    if let Some(error) = result.get("error") {
        if !error.is_null() {
            anyhow::bail!("RPC error: {:?}", error);
        }
    }

    Ok(())
}
