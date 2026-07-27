use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Udaya Faucet - Distribute testnet UDYA to users
#[derive(clap::Parser, Clone)]
#[command(name = "Udaya-faucet")]
struct Cli {
    /// Listen address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Listen port
    #[arg(long, default_value = "8081")]
    port: u16,

    /// RPC endpoint for the node
    #[arg(long, default_value = "http://127.0.0.1:8332")]
    rpc_url: String,

    /// RPC username
    #[arg(long, default_value = "Udaya")]
    rpc_user: String,

    /// RPC password
    #[arg(long, default_value = "Udaya_rpc")]
    rpc_password: String,

    /// Amount to dispense per request (in satoshis)
    #[arg(long, default_value = "1000000000")]
    dispense_amount: u64, // 10 UDYA

    /// Rate limit (seconds between requests per IP)
    #[arg(long, default_value = "86400")]
    rate_limit_secs: u64, // 24 hours

    /// Captcha secret key (Google reCAPTCHA)
    #[arg(long)]
    captcha_secret: Option<String>,

    /// Maximum daily dispense limit per IP (in UDYA)
    #[arg(long, default_value = "100")]
    max_daily_UDYA: f64,

    /// Network (testnet or mainnet)
    #[arg(long, default_value = "testnet")]
    network: String,
}

#[derive(Clone)]
struct FaucetState {
    config: Cli,
    rate_limiter: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    total_dispensed: Arc<RwLock<u64>>,
    client: reqwest::Client,
    start_time: u64,
}

#[derive(Deserialize)]
struct FaucetRequest {
    address: String,
    captcha_token: Option<String>,
}

#[derive(Serialize)]
struct FaucetResponse {
    success: bool,
    message: String,
    txid: Option<String>,
    amount_UDYA: f64,
    address: String,
}

#[derive(Serialize)]
struct FaucetStats {
    total_dispensed_UDYA: f64,
    total_requests: u64,
    requests_today: u64,
    uptime_hours: f64,
    rate_limit_secs: u64,
    dispense_amount_UDYA: f64,
    network: String,
    status: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    use clap::Parser;
    let cli = Cli::parse();

    let state = FaucetState {
        rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        total_dispensed: Arc::new(RwLock::new(0)),
        client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?,
        start_time: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        config: cli.clone(),
    };

    println!("╔══════════════════════════════════════════════════╗");
    println!("║        Udaya Faucet v1.0.0                     ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("Network: {}", cli.network);
    println!(
        "Dispense: {} UDYA per request",
        cli.dispense_amount as f64 / 100_000_000.0
    );
    println!("Rate limit: {} seconds", cli.rate_limit_secs);
    println!("RPC: {}", cli.rpc_url);

    let app = Router::new()
        .route("/", get(faucet_page))
        .route("/api/claim", get(handle_claim))
        .route("/api/stats", get(handle_stats))
        .route("/health", get(health_check))
        .with_state(Arc::new(state));

    let addr = format!("{}:{}", cli.host, cli.port);
    println!("Faucet server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn faucet_page() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        include_str!("../index.html"),
    )
}

async fn handle_claim(
    Query(params): Query<FaucetRequest>,
    state: axum::extract::State<Arc<FaucetState>>,
) -> Json<FaucetResponse> {
    let ip = "unknown"; // In production, get from headers

    // Validate address format
    if params.address.len() < 26 || params.address.len() > 62 {
        return Json(FaucetResponse {
            success: false,
            message: "Invalid address format".to_string(),
            txid: None,
            amount_UDYA: 0.0,
            address: params.address,
        });
    }

    // Rate limiting
    {
        let mut limiter = state.rate_limiter.write();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let requests = limiter.entry(ip.to_string()).or_insert_with(Vec::new);

        // Clean old entries
        requests.retain(|&t| now - t < state.config.rate_limit_secs);

        if requests.len() >= 1 {
            let wait_time =
                state.config.rate_limit_secs - (now - requests.last().copied().unwrap_or(0));
            return Json(FaucetResponse {
                success: false,
                message: format!("Rate limited. Please wait {} seconds.", wait_time),
                txid: None,
                amount_UDYA: 0.0,
                address: params.address,
            });
        }

        requests.push(now);
    }

    // Send coins via RPC
    let amount_UDYA = state.config.dispense_amount as f64 / 100_000_000.0;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendtoaddress",
        "params": [params.address, amount_UDYA]
    });

    match state
        .client
        .post(&state.config.rpc_url)
        .json(&request)
        .basic_auth(&state.config.rpc_user, Some(&state.config.rpc_password))
        .send()
        .await
    {
        Ok(response) => {
            let result: serde_json::Value = response.json().await.unwrap_or_default();
            if let Some(txid) = result.get("result").and_then(|r| r.as_str()) {
                let mut total = state.total_dispensed.write();
                *total += state.config.dispense_amount;

                log::info!(
                    "Dispensed {} UDYA to {} (txid: {})",
                    amount_UDYA,
                    params.address,
                    txid
                );

                Json(FaucetResponse {
                    success: true,
                    message: format!(
                        "Successfully sent {} UDYA to {}",
                        amount_UDYA, params.address
                    ),
                    txid: Some(txid.to_string()),
                    amount_UDYA,
                    address: params.address,
                })
            } else {
                let error = result
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                Json(FaucetResponse {
                    success: false,
                    message: format!("RPC error: {}", error),
                    txid: None,
                    amount_UDYA: 0.0,
                    address: params.address,
                })
            }
        }
        Err(e) => {
            log::error!("RPC request failed: {}", e);
            Json(FaucetResponse {
                success: false,
                message: format!("Failed to connect to node: {}", e),
                txid: None,
                amount_UDYA: 0.0,
                address: params.address,
            })
        }
    }
}

async fn handle_stats(state: axum::extract::State<Arc<FaucetState>>) -> Json<FaucetStats> {
    let total = *state.total_dispensed.read();
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - state.start_time;

    let requests_today = {
        let limiter = state.rate_limiter.read();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        limiter
            .iter()
            .map(|(_, times)| times.iter().filter(|&&t| now - t < 86400).count())
            .sum::<usize>() as u64
    };

    Json(FaucetStats {
        total_dispensed_UDYA: total as f64 / 100_000_000.0,
        total_requests: total as u64,
        requests_today,
        uptime_hours: uptime as f64 / 3600.0,
        rate_limit_secs: state.config.rate_limit_secs,
        dispense_amount_UDYA: state.config.dispense_amount as f64 / 100_000_000.0,
        network: state.config.network.clone(),
        status: "running".to_string(),
    })
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "Udaya-faucet",
        "version": "1.0.0"
    }))
}
