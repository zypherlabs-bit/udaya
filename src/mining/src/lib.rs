use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use udaya_core::consensus::ConsensusEngine;
use udaya_core::transaction::Transaction;
use udaya_core::types::*;

/// Udaya Mining Infrastructure
/// Stratum V2 mining pool protocol, miner telemetry, and dashboards.

/// Mining pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub pool_name: String,
    pub pool_fee_percent: f64,
    pub minimum_payout: u64,
    pub payout_interval_blocks: u64,
    pub listen_addr: String,
    pub listen_port: u16,
    pub max_miners: u32,
    pub difficulty_target: u32,
    pub share_target: u32,
    pub enable_vardiff: bool,
    pub vardiff_retarget_time: u64,
    pub vardiff_target_per_share: f64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            pool_name: "Udaya Pool".to_string(),
            pool_fee_percent: 1.0,
            minimum_payout: 100_000_000, // 1 UDYA
            payout_interval_blocks: 100,
            listen_addr: "0.0.0.0".to_string(),
            listen_port: 3333,
            max_miners: 10000,
            difficulty_target: 0x1D00FFFF,
            share_target: 0x1E00FFFF,
            enable_vardiff: true,
            vardiff_retarget_time: 120,
            vardiff_target_per_share: 30.0,
        }
    }
}

/// Stratum V2 protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StratumMessage {
    // Mining protocol messages
    MiningSubscribe {
        agent: String,
        host: String,
        port: u16,
    },
    MiningSubscribeResult {
        subscription_id: String,
        extranonce1: String,
        extranonce2_size: u32,
    },
    MiningAuthorize {
        worker_name: String,
        password: String,
    },
    MiningAuthorizeResult {
        worker_name: String,
        authorized: bool,
        error: Option<String>,
    },
    MiningSetDifficulty {
        difficulty: f64,
    },
    MiningNotify {
        job_id: String,
        prev_hash: String,
        coinbase1: String,
        coinbase2: String,
        merkle_branch: Vec<String>,
        version: String,
        nbits: String,
        ntime: String,
        clean_jobs: bool,
    },
    MiningSubmit {
        worker_name: String,
        job_id: String,
        extranonce2: String,
        ntime: String,
        nonce: String,
    },
    MiningSubmitResult {
        worker_name: String,
        job_id: String,
        accepted: bool,
        error: Option<String>,
    },
    // Pool-to-miner messages
    SetExtranonce {
        extranonce1: String,
        extranonce2_size: u32,
    },
    // Administration
    Configure {
        extensions: Vec<String>,
    },
    ConfigureResult {
        extensions: Vec<String>,
        error: Option<String>,
    },
}

/// Miner session
#[derive(Debug, Clone)]
pub struct MinerSession {
    pub session_id: String,
    pub worker_name: String,
    pub miner_version: String,
    pub ip_address: String,
    pub connected_at: u64,
    pub last_share_at: u64,
    pub difficulty: f64,
    pub shares_submitted: u64,
    pub shares_valid: u64,
    pub shares_invalid: u64,
    pub hashrate_estimate: f64,
    pub accepted_hashes: u64,
    pub rejected_hashes: u64,
    pub total_work: u64,
}

/// Mining job
#[derive(Debug, Clone)]
pub struct MiningJob {
    pub job_id: String,
    pub height: u64,
    pub block_template: Block,
    pub created_at: u64,
    pub expired: bool,
    pub target: u32,
    pub coinbase_tx: Option<Transaction>,
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub pool_name: String,
    pub connected_miners: u32,
    pub total_blocks_found: u64,
    pub total_shares: u64,
    pub valid_shares: u64,
    pub invalid_shares: u64,
    pub pool_hashrate_ths: f64,
    pub avg_block_time_secs: f64,
    pub estimated_profitability: f64,
    pub miner_count: u32,
    pub active_jobs: u32,
}

/// Miner telemetry data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerTelemetry {
    pub miner_id: String,
    pub timestamp: u64,
    pub hashrate_ghs: f64,
    pub temperature_celsius: f64,
    pub fan_speed_rpm: u32,
    pub power_watts: u32,
    pub efficiency_j_gh: f64,
    pub hw_errors: u32,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub uptime_seconds: u64,
    pub firmware_version: String,
    pub pool_url: String,
    pub worker: String,
}

/// Mining pool engine
pub struct MiningPool {
    config: PoolConfig,
    _consensus: ConsensusEngine,
    miners: Arc<RwLock<HashMap<String, MinerSession>>>,
    jobs: Arc<RwLock<HashMap<String, MiningJob>>>,
    stats: Arc<RwLock<PoolStats>>,
    telemetry: Arc<RwLock<Vec<MinerTelemetry>>>,
}

impl MiningPool {
    pub fn new(config: PoolConfig, consensus: ConsensusEngine) -> Self {
        let pool_name = config.pool_name.clone();
        Self {
            config,
            _consensus: consensus,
            miners: Arc::new(RwLock::new(HashMap::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PoolStats {
                pool_name,
                connected_miners: 0,
                total_blocks_found: 0,
                total_shares: 0,
                valid_shares: 0,
                invalid_shares: 0,
                pool_hashrate_ths: 0.0,
                avg_block_time_secs: 600.0,
                estimated_profitability: 0.0,
                miner_count: 0,
                active_jobs: 0,
            })),
            telemetry: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Authorize a miner
    pub fn authorize_miner(&self, worker_name: &str, _password: &str) -> anyhow::Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = MinerSession {
            session_id: session_id.clone(),
            worker_name: worker_name.to_string(),
            miner_version: String::new(),
            ip_address: String::new(),
            connected_at: now,
            last_share_at: now,
            difficulty: 1.0,
            shares_submitted: 0,
            shares_valid: 0,
            shares_invalid: 0,
            hashrate_estimate: 0.0,
            accepted_hashes: 0,
            rejected_hashes: 0,
            total_work: 0,
        };

        let mut miners = self.miners.write();
        miners.insert(session_id.clone(), session);

        let mut stats = self.stats.write();
        stats.connected_miners = miners.len() as u32;
        stats.miner_count = miners.len() as u32;

        log::info!(
            "Miner authorized: {} (session: {})",
            worker_name,
            session_id
        );
        Ok(session_id)
    }

    /// Submit a share from a miner
    pub fn submit_share(
        &self,
        session_id: &str,
        _job_id: &str,
        _share_nonce: u32,
        share_hash: BlockHash,
    ) -> anyhow::Result<bool> {
        let mut miners = self.miners.write();
        let session = miners
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown miner session"))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        session.last_share_at = now;
        session.shares_submitted += 1;

        // Verify share meets difficulty
        let hash_int = num_bigint::BigUint::from_bytes_be(&share_hash.0);
        let target = num_bigint::BigUint::from_bytes_be(&[
            0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ]);

        let accepted = hash_int <= target;
        if accepted {
            session.shares_valid += 1;
            session.accepted_hashes += 1;
            // Update hashrate estimate
            let time_diff = now.saturating_sub(session.connected_at).max(1);
            session.hashrate_estimate = session.accepted_hashes as f64 / time_diff as f64
                * self.config.difficulty_target as f64;
        } else {
            session.shares_invalid += 1;
            session.rejected_hashes += 1;
        }

        // Update pool stats
        let mut stats = self.stats.write();
        stats.total_shares += 1;
        if accepted {
            stats.valid_shares += 1;
        } else {
            stats.invalid_shares += 1;
        }
        stats.pool_hashrate_ths = self.calculate_pool_hashrate();

        Ok(accepted)
    }

    /// Create a new mining job
    pub fn create_job(&self, block_template: Block, height: u64) -> String {
        let job_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let job = MiningJob {
            job_id: job_id.clone(),
            height,
            block_template,
            created_at: now,
            expired: false,
            target: self.config.difficulty_target,
            coinbase_tx: None,
        };

        let mut jobs = self.jobs.write();
        // Expire old jobs
        jobs.retain(|_, j| !j.expired && now - j.created_at < 600);
        jobs.insert(job_id.clone(), job);

        let mut stats = self.stats.write();
        stats.active_jobs = jobs.len() as u32;

        job_id
    }

    /// Record miner telemetry
    pub fn record_telemetry(&self, telemetry: MinerTelemetry) {
        let mut data = self.telemetry.write();
        data.push(telemetry);

        // Keep last 10000 data points
        if data.len() > 10000 {
            data.remove(0);
        }
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        self.stats.read().clone()
    }

    /// Get connected miners
    pub fn get_miners(&self) -> Vec<MinerSession> {
        self.miners.read().values().cloned().collect()
    }

    /// Calculate pool hashrate
    fn calculate_pool_hashrate(&self) -> f64 {
        let miners = self.miners.read();
        if miners.is_empty() {
            return 0.0;
        }

        let total_hashrate: f64 = miners.values().map(|m| m.hashrate_estimate).sum();

        total_hashrate / 1_000_000_000_000.0 // Convert to TH/s
    }
}

/// ASIC optimization profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsicProfile {
    pub manufacturer: String,
    pub model: String,
    pub hashrate_ghs: f64,
    pub power_watts: u32,
    pub efficiency_j_gh: f64,
    pub recommended_frequency_mhz: u32,
    pub voltage_mv: u32,
    pub fan_profile: String,
    auto_tune_params: HashMap<String, f64>,
}

/// Pre-configured ASIC profiles
pub fn get_asic_profiles() -> Vec<AsicProfile> {
    vec![
        AsicProfile {
            manufacturer: "Udaya".to_string(),
            model: "BF-ASIC S1".to_string(),
            hashrate_ghs: 100.0,
            power_watts: 3250,
            efficiency_j_gh: 32.5,
            recommended_frequency_mhz: 600,
            voltage_mv: 1200,
            fan_profile: "balanced".to_string(),
            auto_tune_params: HashMap::new(),
        },
        AsicProfile {
            manufacturer: "Udaya".to_string(),
            model: "BF-ASIC Pro".to_string(),
            hashrate_ghs: 200.0,
            power_watts: 5000,
            efficiency_j_gh: 25.0,
            recommended_frequency_mhz: 750,
            voltage_mv: 1150,
            fan_profile: "performance".to_string(),
            auto_tune_params: HashMap::new(),
        },
        AsicProfile {
            manufacturer: "Udaya".to_string(),
            model: "BF-ASIC Ultra".to_string(),
            hashrate_ghs: 500.0,
            power_watts: 10000,
            efficiency_j_gh: 20.0,
            recommended_frequency_mhz: 900,
            voltage_mv: 1100,
            fan_profile: "liquid".to_string(),
            auto_tune_params: HashMap::new(),
        },
    ]
}

/// Anti-centralization monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecentralizationReport {
    pub timestamp: u64,
    pub total_miners: u32,
    pub total_pools: u32,
    pub top_pool_hashrate_share: f64,
    pub nakamoto_coefficient: u32,
    pub gini_coefficient: f64,
    pub herfindahl_hirschman_index: f64,
    pub miner_diversity_score: f64,
    pub is_healthy: bool,
    pub recommendations: Vec<String>,
}

/// Decentralization monitor
pub struct DecentralizationMonitor {
    _config: PoolConfig,
}

impl DecentralizationMonitor {
    pub fn new(config: PoolConfig) -> Self {
        Self { _config: config }
    }

    /// Analyze miner distribution for decentralization
    pub fn analyze(&self, miners: &[MinerSession]) -> DecentralizationReport {
        let total_miners = miners.len() as u32;
        let total_hashrate: f64 = miners.iter().map(|m| m.hashrate_estimate).sum();

        // Calculate concentration metrics
        let mut hashrates: Vec<f64> = miners
            .iter()
            .map(|m| m.hashrate_estimate)
            .filter(|&h| h > 0.0)
            .collect();
        hashrates.sort_by(|a, b| b.partial_cmp(a).unwrap());

        // Nakamoto coefficient = smallest number of entities that can collude to 51%
        let mut cumulative = 0.0_f64;
        let mut nakamoto = 0u32;
        for h in &hashrates {
            cumulative += h;
            nakamoto += 1;
            if cumulative / total_hashrate.max(1.0) >= 0.51 {
                break;
            }
        }

        // Gini coefficient
        let gini = calculate_gini_coefficient(&hashrates);

        // Herfindahl-Hirschman Index (HHI)
        let hhi: f64 = hashrates
            .iter()
            .map(|h| {
                let share = h / total_hashrate.max(1.0);
                share * share * 10000.0
            })
            .sum();

        // Top pool share
        let top_pool_share = hashrates.first().copied().unwrap_or(0.0) / total_hashrate.max(1.0);

        // Overall health assessment
        let is_healthy = nakamoto >= 3 && hhi < 2500.0 && top_pool_share < 0.5;

        let mut recommendations = Vec::new();
        if nakamoto < 3 {
            recommendations.push(
                "Nakamoto coefficient too low. Encourage more independent miners.".to_string(),
            );
        }
        if hhi >= 2500.0 {
            recommendations.push(
                "HHI indicates high concentration. Implement pool diversity incentives."
                    .to_string(),
            );
        }
        if top_pool_share >= 0.5 {
            recommendations.push(
                "Single pool has majority hashrate. Deploy anti-centralization measures."
                    .to_string(),
            );
        }

        DecentralizationReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_miners,
            total_pools: hashrates.len() as u32,
            top_pool_hashrate_share: top_pool_share,
            nakamoto_coefficient: nakamoto,
            gini_coefficient: gini,
            herfindahl_hirschman_index: hhi,
            miner_diversity_score: (1.0 - gini) * 100.0,
            is_healthy,
            recommendations,
        }
    }
}

/// Calculate Gini coefficient for hash rate distribution
fn calculate_gini_coefficient(values: &[f64]) -> f64 {
    if values.len() <= 1 || values.iter().all(|&v| v == 0.0) {
        return 0.0;
    }

    let n = values.len() as f64;
    let sorted: Vec<f64> = {
        let mut v = values.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v
    };

    let sum_ranks: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, &val)| (i as f64 + 1.0) * val)
        .sum();

    let _mean = sorted.iter().sum::<f64>() / n;

    let gini = (2.0 * sum_ranks) / (n * sorted.iter().sum::<f64>()) - (n + 1.0) / n;

    if gini.is_nan() || gini < 0.0 {
        0.0
    } else {
        gini
    }
}

/// Pool profitability calculator
pub fn estimate_pool_profitability(
    pool_hashrate_ths: f64,
    network_hashrate_ths: f64,
    block_reward: u64,
    pool_fee_percent: f64,
    _electricity_cost_kwh: f64,
    _miner_efficiency_j_gh: f64,
) -> f64 {
    if network_hashrate_ths <= 0.0 {
        return 0.0;
    }

    let share = pool_hashrate_ths / network_hashrate_ths;
    let blocks_per_day = 144.0; // ~10 min blocks
    let daily_revenue =
        share * blocks_per_day * block_reward as f64 * (1.0 - pool_fee_percent / 100.0);

    // Convert to UDYA per day per TH/s
    daily_revenue / pool_hashrate_ths.max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use udaya_core::consensus::{ConsensusEngine, ConsensusParams};

    #[test]
    fn test_pool_creation() {
        let config = PoolConfig::default();
        let consensus = ConsensusEngine::new(ConsensusParams::default());
        let pool = MiningPool::new(config, consensus);

        let stats = pool.get_stats();
        assert_eq!(stats.connected_miners, 0);
        assert_eq!(stats.pool_name, "Udaya Pool");
    }

    #[test]
    fn test_miner_authorization() {
        let config = PoolConfig::default();
        let consensus = ConsensusEngine::new(ConsensusParams::default());
        let pool = MiningPool::new(config, consensus);

        let session_id = pool.authorize_miner("miner1", "pass").unwrap();
        assert!(!session_id.is_empty());

        let miners = pool.get_miners();
        assert_eq!(miners.len(), 1);
        assert_eq!(miners[0].worker_name, "miner1");
    }

    #[test]
    fn test_gini_coefficient() {
        // Equal distribution should give 0
        let equal = vec![10.0, 10.0, 10.0];
        let gini = calculate_gini_coefficient(&equal);
        assert!((gini - 0.0).abs() < 0.01);

        // Max inequality should give near 1 (one entity has everything)
        let unequal = vec![100.0, 0.0, 0.0];
        let gini = calculate_gini_coefficient(&unequal);
        assert!(gini > 0.5);
    }

    #[test]
    fn test_decentralization_analysis() {
        let config = PoolConfig::default();
        let monitor = DecentralizationMonitor::new(config);

        let miners = vec![
            MinerSession {
                session_id: "1".to_string(),
                worker_name: "pool1".to_string(),
                miner_version: "1.0".to_string(),
                ip_address: "1.1.1.1".to_string(),
                connected_at: 1000,
                last_share_at: 1000,
                difficulty: 1.0,
                shares_submitted: 100,
                shares_valid: 90,
                shares_invalid: 10,
                hashrate_estimate: 100.0,
                accepted_hashes: 90,
                rejected_hashes: 10,
                total_work: 1000,
            },
            MinerSession {
                session_id: "2".to_string(),
                worker_name: "pool2".to_string(),
                miner_version: "1.0".to_string(),
                ip_address: "2.2.2.2".to_string(),
                connected_at: 1000,
                last_share_at: 1000,
                difficulty: 1.0,
                shares_submitted: 50,
                shares_valid: 45,
                shares_invalid: 5,
                hashrate_estimate: 50.0,
                accepted_hashes: 45,
                rejected_hashes: 5,
                total_work: 500,
            },
        ];

        let report = monitor.analyze(&miners);
        assert!(report.nakamoto_coefficient >= 1);
        assert!(report.total_miners >= 2);
    }
}
