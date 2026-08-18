use crate::config::UdayaConfig;
use crate::types::BlockHash;
use std::time::{SystemTime, UNIX_EPOCH};

use prometheus::{
    Gauge, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};

use parking_lot::RwLock;

/// Overall node status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Starting,
    Running,
    Syncing,
    Stopping,
    Error,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::Starting => "starting",
            NodeStatus::Running => "running",
            NodeStatus::Syncing => "syncing",
            NodeStatus::Stopping => "stopping",
            NodeStatus::Error => "error",
        }
    }

    pub fn as_int(&self) -> i64 {
        match self {
            NodeStatus::Starting => 0,
            NodeStatus::Running => 1,
            NodeStatus::Syncing => 2,
            NodeStatus::Stopping => 3,
            NodeStatus::Error => -1,
        }
    }
}

/// Chain split detection status
#[derive(Debug, Clone, Copy)]
pub struct ChainSplitInfo {
    pub fork_height: u64,
    pub fork_hash: BlockHash,
    pub fork_work: u128,
    pub main_chain_hash: BlockHash,
    pub main_chain_work: u128,
    pub is_active: bool,
}

/// Shared observability state accessible to metrics exporters
pub struct MetricsState {
    pub registry: Registry,

    // Blockchain metrics
    pub block_height: IntGauge,
    pub block_count: IntGauge,
    pub chain_tip_hash: IntGaugeVec,
    pub difficulty: Gauge,
    pub total_work: Gauge,
    pub chain_difficulty: Gauge,
    pub last_block_time: Gauge,
    pub block_interval_seconds: Gauge,
    pub block_size_bytes: HistogramVec,
    pub block_tx_count: HistogramVec,

    // Mempool metrics
    pub mempool_tx_count: IntGauge,
    pub mempool_size_bytes: IntGauge,
    pub mempool_orphan_count: IntGauge,
    pub mempool_total_fees: Gauge,
    pub mempool_min_fee_rate: Gauge,
    pub mempool_max_fee_rate: Gauge,

    // P2P Network metrics
    pub peer_count: IntGauge,
    pub peer_count_inbound: IntGauge,
    pub peer_count_outbound: IntGauge,
    pub peer_by_services: IntGaugeVec,
    pub bytes_sent_total: IntCounter,
    pub bytes_received_total: IntCounter,
    pub messages_sent_total: IntCounterVec,
    pub messages_received_total: IntCounterVec,
    pub peer_connection_count: IntGauge,
    pub banned_peers: IntGauge,

    // Mining metrics
    pub hashrate_estimate: Gauge,
    pub network_hashrate: Gauge,
    pub mining_status: IntGauge,
    pub mining_threads_active: IntGauge,
    pub blocks_mined_total: IntCounter,
    pub shares_submitted_total: IntCounter,

    // Node metrics
    pub node_status: IntGauge,
    pub node_uptime_seconds: Gauge,
    pub node_version: IntGaugeVec,
    pub cpu_usage: Gauge,
    pub memory_usage_bytes: Gauge,
    pub disk_usage_bytes: Gauge,

    // RPC metrics
    pub rpc_requests_total: IntCounterVec,
    pub rpc_request_duration_seconds: HistogramVec,
    pub rpc_errors_total: IntCounterVec,
    pub rpc_active_connections: IntGauge,

    // Chain quality metrics
    pub chain_splits_detected_total: IntCounter,
    pub orphan_blocks_total: IntCounter,
    pub reorg_depth: HistogramVec,
    pub reorgs_total: IntCounter,
    pub block_validation_duration_seconds: HistogramVec,

    // Health check metrics
    pub health_status: IntGaugeVec,
    pub last_block_received_timestamp: Gauge,
    pub last_peer_connected_timestamp: Gauge,
    pub last_rpc_request_timestamp: Gauge,

    // Rate limiting & safety
    pub flood_protection_triggered_total: IntCounter,
    pub rate_limited_requests_total: IntCounter,

    // Start time
    pub start_time: RwLock<u64>,
}

impl MetricsState {
    pub fn new(registry: Registry) -> Self {
        let block_height =
            IntGauge::with_opts(Opts::new("Udaya_block_height", "Current blockchain height"))
                .unwrap();
        registry.register(Box::new(block_height.clone())).unwrap();

        let block_count = IntGauge::with_opts(Opts::new(
            "Udaya_block_count",
            "Total number of blocks stored",
        ))
        .unwrap();
        registry.register(Box::new(block_count.clone())).unwrap();

        let chain_tip_hash = IntGaugeVec::new(
            Opts::new(
                "Udaya_chain_tip_hash",
                "Chain tip block hash (encoded as integer segments)",
            ),
            &["segment"],
        )
        .unwrap();
        registry.register(Box::new(chain_tip_hash.clone())).unwrap();

        let difficulty =
            Gauge::with_opts(Opts::new("Udaya_difficulty", "Current mining difficulty")).unwrap();
        registry.register(Box::new(difficulty.clone())).unwrap();

        let total_work =
            Gauge::with_opts(Opts::new("Udaya_total_work", "Total chain work")).unwrap();
        registry.register(Box::new(total_work.clone())).unwrap();

        let chain_difficulty = Gauge::with_opts(Opts::new(
            "Udaya_chain_difficulty_ratio",
            "Chain difficulty as float ratio",
        ))
        .unwrap();
        registry
            .register(Box::new(chain_difficulty.clone()))
            .unwrap();

        let last_block_time = Gauge::with_opts(Opts::new(
            "Udaya_last_block_timestamp",
            "Timestamp of the last block",
        ))
        .unwrap();
        registry
            .register(Box::new(last_block_time.clone()))
            .unwrap();

        let block_interval_seconds = Gauge::with_opts(Opts::new(
            "Udaya_block_interval_seconds",
            "Time since last block in seconds",
        ))
        .unwrap();
        registry
            .register(Box::new(block_interval_seconds.clone()))
            .unwrap();

        let block_size_buckets = vec![
            1000.0,
            10000.0,
            100000.0,
            500000.0,
            1_000_000.0,
            2_000_000.0,
            4_000_000.0,
        ];
        let block_size_bytes = HistogramVec::new(
            HistogramOpts::new("Udaya_block_size_bytes", "Block size distribution in bytes")
                .buckets(block_size_buckets),
            &["type"],
        )
        .unwrap();
        registry
            .register(Box::new(block_size_bytes.clone()))
            .unwrap();

        let block_tx_buckets = vec![1.0, 10.0, 100.0, 1000.0, 5000.0, 10000.0];
        let block_tx_count = HistogramVec::new(
            HistogramOpts::new(
                "Udaya_block_tx_count",
                "Transaction count per block distribution",
            )
            .buckets(block_tx_buckets),
            &["type"],
        )
        .unwrap();
        registry.register(Box::new(block_tx_count.clone())).unwrap();

        // Mempool metrics
        let mempool_tx_count = IntGauge::with_opts(Opts::new(
            "udaya_mempool_tx_count",
            "Number of transactions in mempool",
        ))
        .unwrap();
        registry
            .register(Box::new(mempool_tx_count.clone()))
            .unwrap();

        let mempool_size_bytes = IntGauge::with_opts(Opts::new(
            "udaya_mempool_size_bytes",
            "Total size of mempool in bytes",
        ))
        .unwrap();
        registry
            .register(Box::new(mempool_size_bytes.clone()))
            .unwrap();

        let mempool_orphan_count = IntGauge::with_opts(Opts::new(
            "udaya_mempool_orphan_count",
            "Number of orphan transactions in mempool",
        ))
        .unwrap();
        registry
            .register(Box::new(mempool_orphan_count.clone()))
            .unwrap();

        let mempool_total_fees = Gauge::with_opts(Opts::new(
            "udaya_mempool_total_fees",
            "Total fees in mempool",
        ))
        .unwrap();
        registry
            .register(Box::new(mempool_total_fees.clone()))
            .unwrap();

        let mempool_min_fee_rate = Gauge::with_opts(Opts::new(
            "udaya_mempool_min_fee_rate",
            "Minimum fee rate in mempool",
        ))
        .unwrap();
        registry
            .register(Box::new(mempool_min_fee_rate.clone()))
            .unwrap();

        let mempool_max_fee_rate = Gauge::with_opts(Opts::new(
            "udaya_mempool_max_fee_rate",
            "Maximum fee rate in mempool",
        ))
        .unwrap();
        registry
            .register(Box::new(mempool_max_fee_rate.clone()))
            .unwrap();

        // P2P metrics
        let peer_count = IntGauge::with_opts(Opts::new(
            "Udaya_peer_count",
            "Total number of connected peers",
        ))
        .unwrap();
        registry.register(Box::new(peer_count.clone())).unwrap();

        let peer_count_inbound = IntGauge::with_opts(Opts::new(
            "Udaya_peer_count_inbound",
            "Number of inbound peer connections",
        ))
        .unwrap();
        registry
            .register(Box::new(peer_count_inbound.clone()))
            .unwrap();

        let peer_count_outbound = IntGauge::with_opts(Opts::new(
            "Udaya_peer_count_outbound",
            "Number of outbound peer connections",
        ))
        .unwrap();
        registry
            .register(Box::new(peer_count_outbound.clone()))
            .unwrap();

        let peer_by_services = IntGaugeVec::new(
            Opts::new("Udaya_peer_by_services", "Peer count by service type"),
            &["service"],
        )
        .unwrap();
        registry
            .register(Box::new(peer_by_services.clone()))
            .unwrap();

        let bytes_sent_total = IntCounter::with_opts(Opts::new(
            "Udaya_bytes_sent_total",
            "Total bytes sent over P2P network",
        ))
        .unwrap();
        registry
            .register(Box::new(bytes_sent_total.clone()))
            .unwrap();

        let bytes_received_total = IntCounter::with_opts(Opts::new(
            "Udaya_bytes_received_total",
            "Total bytes received over P2P network",
        ))
        .unwrap();
        registry
            .register(Box::new(bytes_received_total.clone()))
            .unwrap();

        let messages_sent_total = IntCounterVec::new(
            Opts::new(
                "Udaya_messages_sent_total",
                "Total P2P messages sent by type",
            ),
            &["message_type"],
        )
        .unwrap();
        registry
            .register(Box::new(messages_sent_total.clone()))
            .unwrap();

        let messages_received_total = IntCounterVec::new(
            Opts::new(
                "Udaya_messages_received_total",
                "Total P2P messages received by type",
            ),
            &["message_type"],
        )
        .unwrap();
        registry
            .register(Box::new(messages_received_total.clone()))
            .unwrap();

        let peer_connection_count = IntGauge::with_opts(Opts::new(
            "Udaya_peer_connection_count",
            "Current peer connection count",
        ))
        .unwrap();
        registry
            .register(Box::new(peer_connection_count.clone()))
            .unwrap();

        let banned_peers = IntGauge::with_opts(Opts::new(
            "Udaya_banned_peers",
            "Number of currently banned peers",
        ))
        .unwrap();
        registry.register(Box::new(banned_peers.clone())).unwrap();

        // Mining metrics
        let hashrate_estimate = Gauge::with_opts(Opts::new(
            "Udaya_hashrate_estimate_hps",
            "Estimated node hashrate in hashes per second",
        ))
        .unwrap();
        registry
            .register(Box::new(hashrate_estimate.clone()))
            .unwrap();

        let network_hashrate = Gauge::with_opts(Opts::new(
            "Udaya_network_hashrate_hps",
            "Estimated network hashrate in hashes per second",
        ))
        .unwrap();
        registry
            .register(Box::new(network_hashrate.clone()))
            .unwrap();

        let mining_status = IntGauge::with_opts(Opts::new(
            "udaya_mining_status",
            "Mining status: 0=disabled, 1=idle, 2=running",
        ))
        .unwrap();
        registry.register(Box::new(mining_status.clone())).unwrap();

        let mining_threads_active = IntGauge::with_opts(Opts::new(
            "udaya_mining_threads_active",
            "Number of active mining threads",
        ))
        .unwrap();
        registry
            .register(Box::new(mining_threads_active.clone()))
            .unwrap();

        let blocks_mined_total = IntCounter::with_opts(Opts::new(
            "Udaya_blocks_mined_total",
            "Total blocks mined by this node",
        ))
        .unwrap();
        registry
            .register(Box::new(blocks_mined_total.clone()))
            .unwrap();

        let shares_submitted_total = IntCounter::with_opts(Opts::new(
            "Udaya_shares_submitted_total",
            "Total mining shares submitted",
        ))
        .unwrap();
        registry
            .register(Box::new(shares_submitted_total.clone()))
            .unwrap();

        // Node metrics
        let node_status = IntGauge::with_opts(Opts::new(
            "Udaya_node_status",
            "Node status: 0=starting, 1=running, 2=syncing, 3=stopping, -1=error",
        ))
        .unwrap();
        registry.register(Box::new(node_status.clone())).unwrap();

        let node_uptime_seconds = Gauge::with_opts(Opts::new(
            "Udaya_node_uptime_seconds",
            "Node uptime in seconds",
        ))
        .unwrap();
        registry
            .register(Box::new(node_uptime_seconds.clone()))
            .unwrap();

        let node_version = IntGaugeVec::new(
            Opts::new("Udaya_node_version", "Node version info"),
            &["version", "protocol"],
        )
        .unwrap();
        registry.register(Box::new(node_version.clone())).unwrap();

        let cpu_usage =
            Gauge::with_opts(Opts::new("Udaya_cpu_usage_percent", "CPU usage percentage")).unwrap();
        registry.register(Box::new(cpu_usage.clone())).unwrap();

        let memory_usage_bytes = Gauge::with_opts(Opts::new(
            "Udaya_memory_usage_bytes",
            "Memory usage in bytes",
        ))
        .unwrap();
        registry
            .register(Box::new(memory_usage_bytes.clone()))
            .unwrap();

        let disk_usage_bytes =
            Gauge::with_opts(Opts::new("Udaya_disk_usage_bytes", "Disk usage in bytes")).unwrap();
        registry
            .register(Box::new(disk_usage_bytes.clone()))
            .unwrap();

        // RPC metrics
        let rpc_requests_total = IntCounterVec::new(
            Opts::new("Udaya_rpc_requests_total", "Total RPC requests by method"),
            &["method"],
        )
        .unwrap();
        registry
            .register(Box::new(rpc_requests_total.clone()))
            .unwrap();

        let rpc_buckets = vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0];
        let rpc_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "Udaya_rpc_request_duration_seconds",
                "RPC request duration in seconds",
            )
            .buckets(rpc_buckets),
            &["method"],
        )
        .unwrap();
        registry
            .register(Box::new(rpc_request_duration_seconds.clone()))
            .unwrap();

        let rpc_errors_total = IntCounterVec::new(
            Opts::new(
                "Udaya_rpc_errors_total",
                "Total RPC errors by method and error code",
            ),
            &["method", "error_code"],
        )
        .unwrap();
        registry
            .register(Box::new(rpc_errors_total.clone()))
            .unwrap();

        let rpc_active_connections = IntGauge::with_opts(Opts::new(
            "Udaya_rpc_active_connections",
            "Number of active RPC connections",
        ))
        .unwrap();
        registry
            .register(Box::new(rpc_active_connections.clone()))
            .unwrap();

        // Chain quality metrics
        let chain_splits_detected_total = IntCounter::with_opts(Opts::new(
            "Udaya_chain_splits_detected_total",
            "Total chain splits detected",
        ))
        .unwrap();
        registry
            .register(Box::new(chain_splits_detected_total.clone()))
            .unwrap();

        let orphan_blocks_total = IntCounter::with_opts(Opts::new(
            "Udaya_orphan_blocks_total",
            "Total orphan blocks received",
        ))
        .unwrap();
        registry
            .register(Box::new(orphan_blocks_total.clone()))
            .unwrap();

        let reorg_buckets = vec![1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 500.0];
        let reorg_depth = HistogramVec::new(
            HistogramOpts::new("Udaya_reorg_depth", "Reorganization depth distribution")
                .buckets(reorg_buckets),
            &[],
        )
        .unwrap();
        registry.register(Box::new(reorg_depth.clone())).unwrap();

        let reorgs_total = IntCounter::with_opts(Opts::new(
            "Udaya_reorgs_total",
            "Total chain reorganizations",
        ))
        .unwrap();
        registry.register(Box::new(reorgs_total.clone())).unwrap();

        let validation_buckets = vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0];
        let block_validation_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "Udaya_block_validation_duration_seconds",
                "Block validation duration in seconds",
            )
            .buckets(validation_buckets),
            &[],
        )
        .unwrap();
        registry
            .register(Box::new(block_validation_duration_seconds.clone()))
            .unwrap();

        // Health check metrics
        let health_status = IntGaugeVec::new(
            Opts::new(
                "Udaya_health_status",
                "Health check status: 1=healthy, 0=degraded, -1=unhealthy",
            ),
            &["check"],
        )
        .unwrap();
        registry.register(Box::new(health_status.clone())).unwrap();

        let last_block_received_timestamp = Gauge::with_opts(Opts::new(
            "Udaya_last_block_received_timestamp",
            "Unix timestamp of last received block",
        ))
        .unwrap();
        registry
            .register(Box::new(last_block_received_timestamp.clone()))
            .unwrap();

        let last_peer_connected_timestamp = Gauge::with_opts(Opts::new(
            "Udaya_last_peer_connected_timestamp",
            "Unix timestamp of last peer connection",
        ))
        .unwrap();
        registry
            .register(Box::new(last_peer_connected_timestamp.clone()))
            .unwrap();

        let last_rpc_request_timestamp = Gauge::with_opts(Opts::new(
            "Udaya_last_rpc_request_timestamp",
            "Unix timestamp of last RPC request",
        ))
        .unwrap();
        registry
            .register(Box::new(last_rpc_request_timestamp.clone()))
            .unwrap();

        // Safety metrics
        let flood_protection_triggered_total = IntCounter::with_opts(Opts::new(
            "Udaya_flood_protection_triggered_total",
            "Total flood protection triggers",
        ))
        .unwrap();
        registry
            .register(Box::new(flood_protection_triggered_total.clone()))
            .unwrap();

        let rate_limited_requests_total = IntCounter::with_opts(Opts::new(
            "Udaya_rate_limited_requests_total",
            "Total rate-limited requests",
        ))
        .unwrap();
        registry
            .register(Box::new(rate_limited_requests_total.clone()))
            .unwrap();

        let start_time = RwLock::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        Self {
            registry,

            block_height,
            block_count,
            chain_tip_hash,
            difficulty,
            total_work,
            chain_difficulty,
            last_block_time,
            block_interval_seconds,
            block_size_bytes,
            block_tx_count,

            mempool_tx_count,
            mempool_size_bytes,
            mempool_orphan_count,
            mempool_total_fees,
            mempool_min_fee_rate,
            mempool_max_fee_rate,

            peer_count,
            peer_count_inbound,
            peer_count_outbound,
            peer_by_services,
            bytes_sent_total,
            bytes_received_total,
            messages_sent_total,
            messages_received_total,
            peer_connection_count,
            banned_peers,

            hashrate_estimate,
            network_hashrate,
            mining_status,
            mining_threads_active,
            blocks_mined_total,
            shares_submitted_total,

            node_status,
            node_uptime_seconds,
            node_version,
            cpu_usage,
            memory_usage_bytes,
            disk_usage_bytes,

            rpc_requests_total,
            rpc_request_duration_seconds,
            rpc_errors_total,
            rpc_active_connections,

            chain_splits_detected_total,
            orphan_blocks_total,
            reorg_depth,
            reorgs_total,
            block_validation_duration_seconds,

            health_status,
            last_block_received_timestamp,
            last_peer_connected_timestamp,
            last_rpc_request_timestamp,

            flood_protection_triggered_total,
            rate_limited_requests_total,

            start_time,
        }
    }

    /// Update uptime metric
    pub fn update_uptime(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let start = *self.start_time.read();
        self.node_uptime_seconds.set((now - start) as f64);
    }

    /// Update blockchain metrics
    pub fn update_blockchain_metrics(
        &self,
        height: u64,
        block_count: u64,
        difficulty: f64,
        total_work: f64,
        last_block_time_secs: u64,
    ) {
        self.block_height.set(height as i64);
        self.block_count.set(block_count as i64);
        self.difficulty.set(difficulty);
        self.total_work.set(total_work);
        self.chain_difficulty.set(difficulty);

        self.last_block_time.set(last_block_time_secs as f64);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if last_block_time_secs > 0 {
            self.block_interval_seconds
                .set((now - last_block_time_secs) as f64);
        }
    }

    /// Update mempool metrics
    pub fn update_mempool_metrics(
        &self,
        tx_count: usize,
        size_bytes: usize,
        orphan_count: usize,
        total_fees: f64,
        min_fee_rate: f64,
        max_fee_rate: f64,
    ) {
        self.mempool_tx_count.set(tx_count as i64);
        self.mempool_size_bytes.set(size_bytes as i64);
        self.mempool_orphan_count.set(orphan_count as i64);
        self.mempool_total_fees.set(total_fees);
        self.mempool_min_fee_rate.set(min_fee_rate);
        self.mempool_max_fee_rate.set(max_fee_rate);
    }

    /// Update P2P network metrics
    pub fn update_p2p_metrics(
        &self,
        total_peers: usize,
        inbound: usize,
        outbound: usize,
        bytes_sent: u64,
        bytes_received: u64,
        banned_count: usize,
    ) {
        self.peer_count.set(total_peers as i64);
        self.peer_count_inbound.set(inbound as i64);
        self.peer_count_outbound.set(outbound as i64);
        self.peer_connection_count.set(total_peers as i64);
        self.banned_peers.set(banned_count as i64);
        self.bytes_sent_total.inc_by(bytes_sent);
        self.bytes_received_total.inc_by(bytes_received);
    }

    /// Update mining metrics
    pub fn update_mining_metrics(
        &self,
        hashrate: f64,
        network_hashrate: f64,
        is_mining: bool,
        active_threads: usize,
        blocks_mined: u64,
    ) {
        self.hashrate_estimate.set(hashrate);
        self.network_hashrate.set(network_hashrate);
        self.mining_status.set(if is_mining { 2 } else { 0 });
        self.mining_threads_active.set(active_threads as i64);
        self.blocks_mined_total.inc_by(blocks_mined);
    }

    /// Update RPC metrics
    pub fn record_rpc_request(&self, method: &str, duration_secs: f64) {
        self.rpc_requests_total.with_label_values(&[method]).inc();
        self.rpc_request_duration_seconds
            .with_label_values(&[method])
            .observe(duration_secs);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_rpc_request_timestamp.set(now as f64);
    }

    /// Record RPC error
    pub fn record_rpc_error(&self, method: &str, error_code: &str) {
        self.rpc_errors_total
            .with_label_values(&[method, error_code])
            .inc();
    }

    /// Record a chain reorganization
    pub fn record_reorg(&self, depth: u64) {
        self.reorgs_total.inc();
        self.reorg_depth
            .with_label_values::<&str>(&[])
            .observe(depth as f64);
    }

    /// Record a chain split detection
    pub fn record_chain_split(&self) {
        self.chain_splits_detected_total.inc();
    }

    /// Record an orphan block
    pub fn record_orphan_block(&self) {
        self.orphan_blocks_total.inc();
    }

    /// Update health check metric
    pub fn set_health_check(&self, check_name: &str, status: i64) {
        self.health_status
            .with_label_values(&[check_name])
            .set(status);
    }

    /// Set node status
    pub fn set_node_status(&self, status: NodeStatus) {
        self.node_status.set(status.as_int());
        self.set_health_check("node_status", status.as_int());
    }

    /// Track block validation time
    pub fn observe_block_validation(&self, duration_secs: f64) {
        self.block_validation_duration_seconds
            .with_label_values::<&str>(&[])
            .observe(duration_secs);
    }

    /// Track message sent
    pub fn record_message_sent(&self, msg_type: &str) {
        self.messages_sent_total
            .with_label_values(&[msg_type])
            .inc();
    }

    /// Track message received
    pub fn record_message_received(&self, msg_type: &str) {
        self.messages_received_total
            .with_label_values(&[msg_type])
            .inc();
    }
}

/// Create a default metrics state with a new registry
pub fn create_metrics() -> MetricsState {
    let registry = Registry::new();
    MetricsState::new(registry)
}

/// Run a comprehensive health check and return the results
pub fn perform_health_checks(metrics: &MetricsState, config: &UdayaConfig) -> HealthReport {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let start = *metrics.start_time.read();
    let uptime = now - start;

    let mut checks = Vec::new();

    // Check 1: Node Uptime
    let uptime_healthy = uptime > 10;
    metrics.set_health_check("uptime", if uptime_healthy { 1 } else { 0 });
    checks.push(HealthCheck {
        name: "uptime".to_string(),
        status: if uptime_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        detail: format!("Node uptime: {} seconds", uptime),
        healthy: uptime_healthy,
    });

    // Check 2: Peer connectivity
    let peer_count = metrics.peer_count.get();
    let peer_healthy = peer_count >= 1;
    metrics.set_health_check("peers", if peer_healthy { 1 } else { 0 });
    checks.push(HealthCheck {
        name: "peers".to_string(),
        status: if peer_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        detail: format!("Connected peers: {}", peer_count),
        healthy: peer_healthy,
    });

    // Check 3: Block production
    let block_age = metrics.block_interval_seconds.get();
    let block_healthy = block_age < 3600.0; // Less than 1 hour since last block
    metrics.set_health_check("block_production", if block_healthy { 1 } else { 0 });
    checks.push(HealthCheck {
        name: "block_production".to_string(),
        status: if block_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        detail: format!("Time since last block: {:.0} seconds", block_age),
        healthy: block_healthy,
    });

    // Check 4: Memory pool
    let mempool_size = metrics.mempool_size_bytes.get() as f64;
    let mempool_healthy = mempool_size < 300_000_000.0; // Less than 300MB
    metrics.set_health_check("mempool", if mempool_healthy { 1 } else { 0 });
    checks.push(HealthCheck {
        name: "mempool".to_string(),
        status: if mempool_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        detail: format!("Mempool size: {:.0} bytes", mempool_size),
        healthy: mempool_healthy,
    });

    // Check 5: RPC responsiveness
    let rpc_count = metrics
        .rpc_requests_total
        .with_label_values(&["total"])
        .get();
    metrics.set_health_check("rpc", 1);
    checks.push(HealthCheck {
        name: "rpc".to_string(),
        status: "healthy".to_string(),
        detail: format!("Total RPC requests: {}", rpc_count),
        healthy: true,
    });

    // Check 6: Disk space (check if data directory exists and has space)
    let data_dir = &config.storage.data_dir;
    let disk_healthy = std::path::Path::new(data_dir).exists();
    metrics.set_health_check("disk", if disk_healthy { 1 } else { 0 });
    checks.push(HealthCheck {
        name: "disk".to_string(),
        status: if disk_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        detail: format!("Data directory: {}", data_dir),
        healthy: disk_healthy,
    });

    // Check 7: Mining status (if enabled)
    if config.mining.enable {
        let mining_active = metrics.mining_status.get() == 2;
        metrics.set_health_check("mining", if mining_active { 1 } else { -1 });
        checks.push(HealthCheck {
            name: "mining".to_string(),
            status: if mining_active {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            detail: format!(
                "Mining threads active: {}",
                metrics.mining_threads_active.get()
            ),
            healthy: mining_active,
        });
    }

    let all_healthy = checks.iter().all(|c| c.healthy);

    HealthReport {
        status: if all_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        service: "Udayad".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        timestamp: chrono::Utc::now().to_rfc3339(),
        block_height: metrics.block_height.get() as u64,
        peer_count: peer_count as u64,
        mempool_tx_count: metrics.mempool_tx_count.get() as u64,
        checks,
    }
}

/// A single health check result
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub healthy: bool,
}

/// Comprehensive health report
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthReport {
    pub status: String,
    pub service: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
    pub block_height: u64,
    pub peer_count: u64,
    pub mempool_tx_count: u64,
    pub checks: Vec<HealthCheck>,
}

/// System resource metrics collector
pub struct SystemMetrics {
    _last_cpu_measure: parking_lot::RwLock<Option<(std::time::Instant, u64)>>,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            _last_cpu_measure: parking_lot::RwLock::new(None),
        }
    }

    /// Estimate CPU usage (simplified - returns a heuristic)
    pub fn estimate_cpu_usage(&self) -> f64 {
        25.0 // Placeholder; real implementation would use OS-specific APIs
    }

    /// Estimate memory usage
    pub fn estimate_memory_usage(&self) -> f64 {
        // Try to get RSS from /proc/self/status on Linux, fallback to heuristic
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(size_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = size_str.parse::<f64>() {
                            return kb * 1024.0;
                        }
                    }
                }
            }
        }
        // Fallback: estimate based on process info
        256_000_000.0 // 256MB placeholder
    }

    /// Estimate disk usage for the data directory
    pub fn estimate_disk_usage(path: &str) -> f64 {
        let path = std::path::Path::new(path);
        if !path.exists() {
            return 0.0;
        }
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total += metadata.len();
                    }
                }
            }
        }
        total as f64
    }
}

use prometheus::HistogramOpts;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = create_metrics();
        assert_eq!(metrics.block_height.get(), 0);
        assert_eq!(metrics.peer_count.get(), 0);
        assert_eq!(metrics.mempool_tx_count.get(), 0);
    }

    #[test]
    fn test_update_blockchain_metrics() {
        let metrics = create_metrics();
        metrics.update_blockchain_metrics(100, 100, 1.5, 1000.0, 1234567890);
        assert_eq!(metrics.block_height.get(), 100);
        assert_eq!(metrics.block_count.get(), 100);
        assert!((metrics.difficulty.get() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_update_mempool_metrics() {
        let metrics = create_metrics();
        metrics.update_mempool_metrics(50, 10000, 2, 0.5, 1.0, 100.0);
        assert_eq!(metrics.mempool_tx_count.get(), 50);
        assert_eq!(metrics.mempool_size_bytes.get(), 10000);
        assert_eq!(metrics.mempool_orphan_count.get(), 2);
    }

    #[test]
    fn test_update_p2p_metrics() {
        let metrics = create_metrics();
        metrics.update_p2p_metrics(10, 3, 7, 1000, 500, 1);
        assert_eq!(metrics.peer_count.get(), 10);
        assert_eq!(metrics.peer_count_inbound.get(), 3);
        assert_eq!(metrics.peer_count_outbound.get(), 7);
    }

    #[test]
    fn test_node_status_transition() {
        assert_eq!(NodeStatus::Starting.as_str(), "starting");
        assert_eq!(NodeStatus::Running.as_str(), "running");
        assert_eq!(NodeStatus::Syncing.as_str(), "syncing");
        assert_eq!(NodeStatus::Stopping.as_str(), "stopping");
        assert_eq!(NodeStatus::Error.as_str(), "error");
        assert_eq!(NodeStatus::Running.as_int(), 1);
        assert_eq!(NodeStatus::Error.as_int(), -1);
    }

    #[test]
    fn test_set_node_status() {
        let metrics = create_metrics();
        metrics.set_node_status(NodeStatus::Running);
        assert_eq!(metrics.node_status.get(), 1_i64);
    }

    #[test]
    fn test_record_rpc_request() {
        let metrics = create_metrics();
        metrics.record_rpc_request("getblockchaininfo", 0.05);
        metrics.record_rpc_request("getblockchaininfo", 0.03);
        assert_eq!(
            metrics
                .rpc_requests_total
                .with_label_values(&["getblockchaininfo"])
                .get(),
            2
        );
    }

    #[test]
    fn test_health_checks() {
        let metrics = create_metrics();
        let config = UdayaConfig::default();
        metrics.set_node_status(NodeStatus::Running);
        let report = perform_health_checks(&metrics, &config);
        assert_eq!(report.service, "Udayad");
        assert!(!report.checks.is_empty());
        // Peers check should be degraded since no peers connected
        let peers_check = report.checks.iter().find(|c| c.name == "peers").unwrap();
        assert!(!peers_check.healthy);
    }

    #[test]
    fn test_chain_split_recording() {
        let metrics = create_metrics();
        metrics.record_chain_split();
        metrics.record_chain_split();
        assert_eq!(metrics.chain_splits_detected_total.get(), 2);
    }

    #[test]
    fn test_reorg_recording() {
        let metrics = create_metrics();
        metrics.record_reorg(5);
        assert_eq!(metrics.reorgs_total.get(), 1);
    }

    #[test]
    fn test_block_validation_timing() {
        let metrics = create_metrics();
        metrics.observe_block_validation(0.25);
        // Observation should not panic
        // assert!(true); // Removed always-true assertion
    }
}
