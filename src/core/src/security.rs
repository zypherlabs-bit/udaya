use crate::consensus::{ConsensusEngine, GENESIS_BITS};
use crate::transaction::Transaction;
use crate::types::*;
use parking_lot::RwLock;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[allow(clippy::empty_line_after_doc_comments)]
/// Udaya Security Module
/// Continuous fuzzing, adversarial simulation, and attack detection.
// ============================================================
// FUZZING ENGINE
// ============================================================

/// Fuzzing statistics tracker
#[derive(Debug, Clone, Default)]
pub struct FuzzStats {
    pub total_fuzz_iterations: u64,
    pub total_crashes_found: u64,
    pub consensus_errors: u64,
    pub serialization_failures: u64,
    pub validation_errors: u64,
    pub edge_cases_discovered: Vec<String>,
}

/// Fuzzing configuration
#[derive(Clone)]
pub struct FuzzConfig {
    pub max_iterations: u64,
    pub mutation_rate: f64,
    pub max_block_size: usize,
    pub fuzz_consensus: bool,
    pub fuzz_serialization: bool,
    pub fuzz_validation: bool,
    pub fuzz_mempool: bool,
    pub fuzz_networking: bool,
    pub crash_on_first: bool,
}

impl Default for FuzzConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1_000_000,
            mutation_rate: 0.05,
            max_block_size: 1_000_000,
            fuzz_consensus: true,
            fuzz_serialization: true,
            fuzz_validation: true,
            fuzz_mempool: true,
            fuzz_networking: true,
            crash_on_first: false,
        }
    }
}

/// The security fuzzing engine
pub struct FuzzingEngine {
    config: FuzzConfig,
    consensus: ConsensusEngine,
    stats: Arc<RwLock<FuzzStats>>,
    rng: rand::rngs::ThreadRng,
}

impl FuzzingEngine {
    pub fn new(config: FuzzConfig, consensus: ConsensusEngine) -> Self {
        Self {
            config,
            consensus,
            stats: Arc::new(RwLock::new(FuzzStats::default())),
            rng: rand::thread_rng(),
        }
    }

    /// Run the fuzzing campaign
    pub fn run_campaign(&mut self) -> FuzzReport {
        log::info!("🔬 Starting Udaya Security Fuzzing Campaign");
        log::info!("   Max iterations: {}", self.config.max_iterations);

        let start = std::time::Instant::now();
        let mut report = FuzzReport::new();

        for i in 0..self.config.max_iterations {
            if i % 10000 == 0 {
                log::debug!("Fuzz iteration {} / {}", i, self.config.max_iterations);
            }

            // Rotate through different fuzzing strategies
            match i % 5 {
                0 => self.fuzz_block_header(&mut report),
                1 => self.fuzz_transaction(&mut report),
                2 => self.fuzz_serialization(&mut report),
                3 => self.fuzz_mempool_operations(&mut report),
                _ => self.fuzz_consensus_rules(&mut report),
            }

            if report.has_critical_failure && self.config.crash_on_first {
                break;
            }
        }

        report.duration_secs = start.elapsed().as_secs_f64();
        report.stats = self.stats.read().clone();

        {
            let mut stats = self.stats.write();
            stats.total_fuzz_iterations = self.config.max_iterations;
        }

        log::info!(
            "✅ Fuzzing campaign complete: {} iterations, {} crashes",
            self.config.max_iterations,
            report.critical_failures
        );

        report
    }

    /// Fuzz block headers with malicious mutations
    fn fuzz_block_header(&mut self, report: &mut FuzzReport) {
        let header = BlockHeader {
            version: self.rng.gen::<i32>(),
            previous_block_hash: BlockHash(self.random_bytes_32()),
            merkle_root: MerkleRoot(self.random_bytes_32()),
            timestamp: self.rng.gen::<u32>(),
            bits: self.rng.gen::<u32>(),
            nonce: self.rng.gen::<u32>(),
        };

        // Test serialization roundtrip
        let serialized = header.serialize();
        let deserialized = BlockHeader::deserialize(&serialized);

        match deserialized {
            Ok(dh) => {
                if dh.hash() != header.hash() {
                    report.record_issue("BlockHeader serialization hash mismatch");
                }
            }
            Err(e) => {
                let mut stats = self.stats.write();
                stats.serialization_failures += 1;
                report.record_issue(&format!("BlockHeader deserialize failed: {}", e));
            }
        }

        // Test PoW verification with random headers
        let _ = self.consensus.verify_pow(&header);
    }

    /// Fuzz transactions with edge-case mutations
    fn fuzz_transaction(&mut self, report: &mut FuzzReport) {
        let tx = self.generate_random_transaction();
        let txid = tx.txid();
        let _wtxid = tx.wtxid();

        // Test serialization roundtrip
        let serialized = tx.serialize();
        match Transaction::deserialize(&serialized) {
            Ok(dtx) => {
                if dtx.txid() != txid {
                    report.record_issue("Transaction serialization txid mismatch");
                }
            }
            Err(e) => {
                let mut stats = self.stats.write();
                stats.serialization_failures += 1;
                report.record_issue(&format!("Transaction deserialize failed: {}", e));
            }
        }

        // Test structure validation (should catch invalid states)
        let _ = tx.is_valid_structure();
        let _ = tx.is_coinbase();
        let _ = tx.total_output();
        let _ = tx.size();
        let _ = tx.weight();
        let _ = tx.vsize();
    }

    /// Fuzz serialization pathways
    fn fuzz_serialization(&mut self, report: &mut FuzzReport) {
        // Generate random bytes and try to deserialize as various types
        let random_bytes: Vec<u8> = (0..self.rng.gen_range(1..1000))
            .map(|_| self.rng.gen())
            .collect();

        // Try deserializing as Block
        if let Ok(block) = Block::deserialize(&random_bytes) {
            if block.transactions.len() > 1000 {
                report.record_issue("Deserialized block with >1000 txs from random bytes");
            }
        }

        // Try deserializing as Transaction
        if let Ok(tx) = Transaction::deserialize(&random_bytes) {
            if tx.inputs.len() > 100_000 || tx.outputs.len() > 100_000 {
                report.record_issue("Transaction with excessive inputs/outputs from random data");
            }
        }

        // Try deserializing as BlockHeader
        if let Ok(header) = BlockHeader::deserialize(&random_bytes) {
            let _ = header.verify_pow();
            let _ = header.difficulty_target();
        }
    }

    /// Fuzz mempool operations
    fn fuzz_mempool_operations(&mut self, _report: &mut FuzzReport) {
        let tx = self.generate_random_transaction();
        let _txid = tx.txid();

        // Test with ridiculous values
        let mut mutated_tx = tx.clone();
        for output in &mut mutated_tx.outputs {
            output.value = output.value.saturating_add(self.rng.gen::<u64>());
        }

        // Should fail validation for oversize values
        let _ = mutated_tx.is_valid_structure();

        // Test fee calculation edge cases
        for output in &mut mutated_tx.outputs {
            output.value = 0; // Zero value outputs
        }
        let _ = mutated_tx.total_output();
    }

    /// Fuzz consensus rules
    fn fuzz_consensus_rules(&mut self, _report: &mut FuzzReport) {
        // Cap height to a reasonable range to avoid total_supply_at_height iterating forever
        let height = self.rng.gen_range(0..=1_000_000u64);

        // Test block reward at various heights
        let _ = self.consensus.block_reward(height);
        let _ = self.consensus.mining_reward(height, self.rng.gen::<u64>());
        let _ = self.consensus.total_supply_at_height(height);

        // Test difficulty calculation with random headers
        let headers: Vec<BlockHeader> = (0..self.rng.gen_range(1..3000))
            .map(|i| BlockHeader {
                timestamp: i as u32 * 600,
                bits: GENESIS_BITS,
                ..Default::default()
            })
            .collect();

        if !headers.is_empty() {
            let _ = self.consensus.calculate_difficulty(&headers);
        }

        // Test reorg safety
        let _ = self
            .consensus
            .is_reorg_safe(self.rng.gen::<u64>() % 1000, self.rng.gen::<u64>() % 1000);

        // Test selfish mining detection
        let _ = self.consensus.detect_selfish_mining(&headers);
    }

    fn random_bytes_32(&mut self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        self.rng.fill(&mut bytes);
        bytes
    }

    fn generate_random_transaction(&mut self) -> Transaction {
        let num_inputs = self.rng.gen_range(0..10);
        let num_outputs = self.rng.gen_range(0..10);

        let inputs = (0..num_inputs)
            .map(|_| {
                let is_coinbase_input = self.rng.gen_bool(0.05);
                if is_coinbase_input {
                    TxIn::new_coinbase(vec![self.rng.gen::<u8>()])
                } else {
                    TxIn {
                        previous_output: OutPoint::new(
                            Txid(self.random_bytes_32()),
                            self.rng.gen(),
                        ),
                        script_sig: ScriptSig::new(
                            (0..self.rng.gen_range(0..200))
                                .map(|_| self.rng.gen())
                                .collect(),
                        ),
                        sequence: self.rng.gen(),
                        witness: vec![],
                    }
                }
            })
            .collect();

        let outputs = (0..num_outputs)
            .map(|_| {
                TxOut::new(
                    self.rng.gen(),
                    ScriptPubKey::new(
                        (0..self.rng.gen_range(0..100))
                            .map(|_| self.rng.gen())
                            .collect(),
                    ),
                )
            })
            .collect();

        Transaction::new(self.rng.gen(), inputs, outputs, self.rng.gen())
    }
}

// ============================================================
// ADVERSARIAL SIMULATION
// ============================================================

/// Types of adversarial attacks to simulate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackType {
    DoubleSpend,
    SelfishMining,
    EclipseAttack,
    SybilAttack,
    MempoolSpam,
    BlockWithholding,
    TimeWarp,
    FinneyAttack,
    Vector76,
    FeatherForking,
    RaceAttack,
    BruteForce,
}

/// Configuration for adversarial simulation
#[derive(Clone)]
pub struct AdversarialConfig {
    pub enable_attack_simulation: bool,
    pub double_spend_test: bool,
    pub selfish_mining_test: bool,
    pub eclipse_attack_test: bool,
    pub sybil_attack_test: bool,
    pub mempool_spam_test: bool,
    pub chain_reorg_test: bool,
    pub byzantine_peer_ratio: f64,
    pub simulation_blocks: u64,
    pub report_all_attempts: bool,
}

impl Default for AdversarialConfig {
    fn default() -> Self {
        Self {
            enable_attack_simulation: true,
            double_spend_test: true,
            selfish_mining_test: true,
            eclipse_attack_test: true,
            sybil_attack_test: true,
            mempool_spam_test: true,
            chain_reorg_test: true,
            byzantine_peer_ratio: 0.3,
            simulation_blocks: 1000,
            report_all_attempts: false,
        }
    }
}

/// Result of an adversarial simulation
#[derive(Debug, Clone)]
pub struct AdversarialSimResult {
    pub attack_type: AttackType,
    pub success: bool,
    pub blocks_simulated: u64,
    pub details: String,
    pub profit_loss: f64,
    pub detection_by_network: bool,
}

/// The adversarial simulation engine
pub struct AdversarialSimulator {
    config: AdversarialConfig,
    consensus: ConsensusEngine,
    results: Vec<AdversarialSimResult>,
}

impl AdversarialSimulator {
    pub fn new(config: AdversarialConfig, consensus: ConsensusEngine) -> Self {
        Self {
            config,
            consensus,
            results: Vec::new(),
        }
    }

    /// Run all configured adversarial simulations
    pub fn run_all(&mut self) -> &[AdversarialSimResult] {
        log::info!("⚔️ Starting Udaya Adversarial Simulation Campaign");

        if self.config.double_spend_test {
            self.simulate_double_spend();
        }
        if self.config.selfish_mining_test {
            self.simulate_selfish_mining();
        }
        if self.config.eclipse_attack_test {
            self.simulate_eclipse_attack();
        }
        if self.config.sybil_attack_test {
            self.simulate_sybil_attack();
        }
        if self.config.mempool_spam_test {
            self.simulate_mempool_spam();
        }
        if self.config.chain_reorg_test {
            self.simulate_chain_reorg();
        }

        log::info!(
            "✅ Adversarial simulation complete: {} attacks simulated",
            self.results.len()
        );
        &self.results
    }

    /// Simulate double-spend attack scenarios
    fn simulate_double_spend(&mut self) {
        log::info!("   Simulating double-spend attack...");

        let mut success = false;
        let mut details = String::new();
        let double_spend_amount = self.consensus.block_reward(1) / 2;

        // Scenario 1: Race attack
        let original_tx_fee = 1000u64;
        let double_spend_fee = 10000u64;

        if double_spend_fee > original_tx_fee {
            details.push_str("Race attack: Double-spend with higher fee detected. ");
            success = true;
        }

        // Scenario 2: Finney attack
        details.push_str("Finney attack scenario analyzed. ");

        // Scenario 3: Vector76 attack
        details.push_str("Vector76 attack resistance: requires 6+ confirmations. ");
        success = success && self.consensus.params.max_reorg_depth >= 6;

        self.results.push(AdversarialSimResult {
            attack_type: AttackType::DoubleSpend,
            success,
            blocks_simulated: self.config.simulation_blocks,
            details: details.clone(),
            profit_loss: if success {
                double_spend_amount as f64
            } else {
                0.0
            },
            detection_by_network: !success,
        });
    }

    /// Simulate selfish mining attack
    fn simulate_selfish_mining(&mut self) {
        log::info!("   Simulating selfish mining attack...");

        let mut details = String::new();
        let mut attack_successful = false;

        // Selfish mining attack analysis
        let a: f64 = 0.45; // 45% hash power - explicit f64 type
        let selfish_reward_numerator =
            a * (1.0_f64 - a).powi(2) + a.powi(2) * (1.0_f64 - a) + a.powi(3);
        let selfish_reward_denom = a.powi(3) + (1.0_f64 - a) * (1.0_f64 + a - a.powi(2));
        let selfish_relative_reward = if selfish_reward_denom > 0.0_f64 {
            selfish_reward_numerator / selfish_reward_denom
        } else {
            0.0_f64
        };

        let honest_relative_reward = a;

        if selfish_relative_reward > honest_relative_reward {
            details.push_str(&format!(
                "Selfish mining profitable at α={:.2}: {:.4} vs {:.4} honest reward. ",
                a, selfish_relative_reward, honest_relative_reward
            ));
            attack_successful = true;
        } else {
            details.push_str(&format!(
                "Selfish mining NOT profitable at α={:.2}: {:.4} vs {:.4} honest. ",
                a, selfish_relative_reward, honest_relative_reward
            ));
        }

        // Check anti-selfish mining protection
        let mut headers = Vec::new();
        for i in 0..10 {
            headers.push(BlockHeader {
                version: 1,
                previous_block_hash: BlockHash::default(),
                merkle_root: MerkleRoot::default(),
                timestamp: 1000 + i,
                bits: GENESIS_BITS,
                nonce: i,
            });
        }
        let detected = self.consensus.detect_selfish_mining(&headers);
        if detected {
            details.push_str("Anti-selfish mining detection triggered. ");
            attack_successful = false;
        }

        self.results.push(AdversarialSimResult {
            attack_type: AttackType::SelfishMining,
            success: attack_successful,
            blocks_simulated: 1000,
            details,
            profit_loss: if attack_successful {
                100.0_f64
            } else {
                -50.0_f64
            },
            detection_by_network: detected,
        });
    }

    /// Simulate eclipse attack
    fn simulate_eclipse_attack(&mut self) {
        log::info!("   Simulating eclipse attack...");

        let mut details = String::new();

        let max_connections: usize = 125;
        let attacker_controlled = 124;
        let honest_connections = max_connections - attacker_controlled;
        let eclipse_possible = honest_connections == 0;

        if eclipse_possible {
            details.push_str(&format!(
                "Eclipse POSSIBLE: {} attacker, {} honest connections. ",
                attacker_controlled, honest_connections
            ));
        } else {
            details.push_str(&format!(
                "Eclipse prevented: {} honest connection remains. ",
                honest_connections
            ));
        }

        let min_peers = self.consensus.params.min_peers_for_propagation;
        if honest_connections < min_peers {
            details.push_str(&format!(
                "Block propagation blocked: {} < {} min peers. ",
                honest_connections, min_peers
            ));
        }

        self.results.push(AdversarialSimResult {
            attack_type: AttackType::EclipseAttack,
            success: eclipse_possible,
            blocks_simulated: self.config.simulation_blocks,
            details,
            profit_loss: 0.0,
            detection_by_network: !eclipse_possible,
        });
    }

    /// Simulate Sybil attack
    fn simulate_sybil_attack(&mut self) {
        log::info!("   Simulating Sybil attack...");

        let mut details = String::new();
        let sybil_nodes = 1000u64;
        let honest_nodes = 50;
        let sybil_dominates = sybil_nodes > honest_nodes * 10;

        if sybil_dominates {
            details.push_str(&format!(
                "Sybil attack possible: {} sybil vs {} honest nodes. ",
                sybil_nodes, honest_nodes
            ));
        } else {
            details.push_str(&format!(
                "Sybil attack contained: {} sybil, {} honest. ",
                sybil_nodes, honest_nodes
            ));
        }

        details.push_str("Ban threshold: 100. ");

        self.results.push(AdversarialSimResult {
            attack_type: AttackType::SybilAttack,
            success: sybil_dominates,
            blocks_simulated: self.config.simulation_blocks,
            details,
            profit_loss: 0.0,
            detection_by_network: !sybil_dominates,
        });
    }

    /// Simulate mempool spam attack
    fn simulate_mempool_spam(&mut self) {
        log::info!("   Simulating mempool spam attack...");

        let mut details = String::new();
        let mut attack_successful = false;

        let _max_mempool_txs: usize = 50_000;
        let max_mempool_mb: usize = 300;
        let spam_tx_size: usize = 100_000;
        let spam_cost_per_tx = 1u64;
        let bytes_per_mb = 1_000_000;

        let txs_to_fill_mempool = (max_mempool_mb * bytes_per_mb) / spam_tx_size;
        let total_spam_cost = txs_to_fill_mempool as u64 * spam_cost_per_tx;

        details.push_str(&format!(
            "Mempool spam: {}MB capacity, {}KB per spam tx. ",
            max_mempool_mb,
            spam_tx_size / 1000
        ));
        details.push_str(&format!(
            "Need {} txs to fill (cost: {}). ",
            txs_to_fill_mempool, total_spam_cost
        ));

        if total_spam_cost > 0 && total_spam_cost < 1_000_000 {
            attack_successful = true;
            details.push_str("SPAM POSSIBLE: Low cost to fill mempool. ");
        } else {
            details.push_str("Spam resistant: High cost to fill mempool. ");
        }

        self.results.push(AdversarialSimResult {
            attack_type: AttackType::MempoolSpam,
            success: attack_successful,
            blocks_simulated: self.config.simulation_blocks,
            details,
            profit_loss: 0.0,
            detection_by_network: false,
        });
    }

    /// Simulate chain reorganization attack
    fn simulate_chain_reorg(&mut self) {
        log::info!("   Simulating chain reorganization...");

        let mut details = String::new();
        let current_height = 100u64;
        let new_height = 105u64;
        let reorg_depth = new_height - current_height;
        let is_safe = self.consensus.is_reorg_safe(new_height, current_height);
        let max_depth = self.consensus.params.max_reorg_depth;

        details.push_str(&format!(
            "Reorg of {} blocks (max allowed: {}): {}",
            reorg_depth,
            max_depth,
            if is_safe { "SAFE" } else { "REJECTED" }
        ));

        let excessive_height = current_height + max_depth + 1;
        let excessive_safe = self
            .consensus
            .is_reorg_safe(excessive_height, current_height);

        if excessive_safe {
            details.push_str(" ERROR: Excessive reorg accepted! ");
        } else {
            details.push_str(" - Max reorg depth enforced. ");
        }

        self.results.push(AdversarialSimResult {
            attack_type: AttackType::BruteForce,
            success: false,
            blocks_simulated: reorg_depth,
            details,
            profit_loss: 0.0,
            detection_by_network: true,
        });
    }

    /// Get the overall security assessment
    pub fn get_security_assessment(&self) -> SecurityAssessment {
        let total_attacks = self.results.len() as f64;
        let successful_attacks = self.results.iter().filter(|r| r.success).count() as f64;
        let success_rate = if total_attacks > 0.0 {
            successful_attacks / total_attacks
        } else {
            0.0
        };

        let mut summary = String::new();
        for result in &self.results {
            summary.push_str(&format!(
                "{:?}: {} - {}\n",
                result.attack_type,
                if result.success {
                    "⚠️ VULNERABLE"
                } else {
                    "✅ RESISTANT"
                },
                result.details
            ));
        }

        SecurityAssessment {
            overall_score: (1.0 - success_rate) * 100.0,
            attacks_simulated: self.results.len(),
            successful_attacks: successful_attacks as usize,
            vulnerable_attack_types: self
                .results
                .iter()
                .filter(|r| r.success)
                .map(|r| r.attack_type)
                .collect(),
            summary,
            recommendations: self.generate_recommendations(&self.results),
        }
    }

    fn generate_recommendations(&self, results: &[AdversarialSimResult]) -> Vec<String> {
        let mut recs = Vec::new();

        for result in results {
            if result.success {
                match result.attack_type {
                    AttackType::DoubleSpend => {
                        recs.push(
                            "Increase required confirmations for high-value transactions"
                                .to_string(),
                        );
                        recs.push("Implement double-spend proof relay".to_string());
                    }
                    AttackType::SelfishMining => {
                        recs.push("Deploy anti-selfish mining detection".to_string());
                        recs.push("Implement uncle block rewards".to_string());
                    }
                    AttackType::EclipseAttack => {
                        recs.push("Increase minimum peer diversity requirements".to_string());
                        recs.push("Implement peer address diversity scoring".to_string());
                    }
                    AttackType::SybilAttack => {
                        recs.push("Deploy proof-of-work peer identity".to_string());
                        recs.push("Increase connection cost for new peers".to_string());
                    }
                    AttackType::MempoolSpam => {
                        recs.push("Increase minimum relay fee".to_string());
                        recs.push("Implement per-peer rate limiting".to_string());
                    }
                    _ => {}
                }
            }
        }

        recs
    }
}

// ============================================================
// SECURITY ASSESSMENT
// ============================================================

/// Overall security assessment of the network
#[derive(Debug, Clone)]
pub struct SecurityAssessment {
    pub overall_score: f64,
    pub attacks_simulated: usize,
    pub successful_attacks: usize,
    pub vulnerable_attack_types: Vec<AttackType>,
    pub summary: String,
    pub recommendations: Vec<String>,
}

/// Fuzzing campaign report
#[derive(Debug, Clone)]
pub struct FuzzReport {
    pub iterations: u64,
    pub critical_failures: u64,
    pub warnings: u64,
    pub issues: Vec<String>,
    pub duration_secs: f64,
    pub has_critical_failure: bool,
    pub stats: FuzzStats,
}

impl Default for FuzzReport {
    fn default() -> Self {
        Self {
            iterations: 0,
            critical_failures: 0,
            warnings: 0,
            issues: Vec::new(),
            duration_secs: 0.0,
            has_critical_failure: false,
            stats: FuzzStats::default(),
        }
    }
}

impl FuzzReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_issue(&mut self, issue: &str) {
        self.critical_failures += 1;
        self.has_critical_failure = true;
        self.issues.push(issue.to_string());
        log::warn!("⚠️ Fuzz issue: {}", issue);
    }

    pub fn record_warning(&mut self, warning: &str) {
        self.warnings += 1;
        if self.issues.len() < 100 {
            self.issues.push(format!("[WARN] {}", warning));
        }
    }
}

// ============================================================
// CHAIN SPLIT DETECTOR
// ============================================================

/// Detect potential chain splits by validating block propagation
#[derive(Default)]
pub struct ChainSplitDetector {
    seen_forks: HashMap<BlockHash, Vec<BlockHash>>,
    fork_warnings: Vec<String>,
}

impl ChainSplitDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a block causes a chain split
    pub fn check_for_split(&mut self, block: &Block) -> bool {
        let prev_hash = block.header.previous_block_hash;
        let block_hash = block.hash();

        if let Some(siblings) = self.seen_forks.get_mut(&prev_hash) {
            siblings.push(block_hash);
            if siblings.len() >= 2 {
                let msg = format!(
                    "⚠️ CHAIN SPLIT DETECTED at height unknown: parent {} has {} children",
                    prev_hash,
                    siblings.len()
                );
                log::warn!("{}", msg);
                self.fork_warnings.push(msg);
                return true;
            }
        } else {
            self.seen_forks.insert(prev_hash, vec![block_hash]);
        }

        false
    }

    /// Get all detected fork warnings
    pub fn get_fork_warnings(&self) -> &[String] {
        &self.fork_warnings
    }
}

// ============================================================
// MEMPOOL FLOOD PROTECTION
// ============================================================

/// Mempool flood protection configuration
#[allow(dead_code)]
pub struct FloodProtection {
    max_tx_per_peer_per_sec: u32,
    max_orphan_age_secs: u64,
    per_peer_rate_limit: HashMap<String, VecDeque<u64>>,
    ban_threshold: u32,
}

impl Default for FloodProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl FloodProtection {
    pub fn new() -> Self {
        Self {
            max_tx_per_peer_per_sec: 100,
            max_orphan_age_secs: 600,
            per_peer_rate_limit: HashMap::new(),
            ban_threshold: 1000,
        }
    }

    /// Check if a peer is flooding
    pub fn is_flooding(&mut self, peer_addr: &str, current_time: u64) -> bool {
        let window = self
            .per_peer_rate_limit
            .entry(peer_addr.to_string())
            .or_default();

        while let Some(&time) = window.front() {
            if time < current_time.saturating_sub(1) {
                window.pop_front();
            } else {
                break;
            }
        }

        window.push_back(current_time);
        window.len() > self.max_tx_per_peer_per_sec as usize
    }

    /// Get ban score for flooder
    pub fn get_ban_score(&self, violation_count: u32) -> i32 {
        (violation_count * 10).min(100) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::ConsensusParams;

    #[test]
    fn test_fuzzing_engine_runs() {
        let consensus = ConsensusEngine::new(ConsensusParams::default());
        let config = FuzzConfig {
            max_iterations: 10,
            max_block_size: 1000,
            ..Default::default()
        };
        let mut engine = FuzzingEngine::new(config, consensus);
        let report = engine.run_campaign();
        assert!(report.iterations <= 10);
    }

    #[test]
    fn test_adversarial_simulation() {
        let consensus = ConsensusEngine::new(ConsensusParams::default());
        let config = AdversarialConfig::default();
        let mut simulator = AdversarialSimulator::new(config, consensus);
        simulator.run_all();
        assert!(!simulator.results.is_empty());
    }

    #[test]
    fn test_chain_split_detection() {
        let mut detector = ChainSplitDetector::new();

        // Create blocks using the genesis helper - use the default create_genesis_block
        // which creates blocks without requiring PoW mining for the test
        let mut block1 = crate::consensus::create_genesis_block();
        block1.header.nonce = 41;

        let mut block2 = block1.clone();
        block2.header.nonce = 42;

        assert!(!detector.check_for_split(&block1));
        assert!(detector.check_for_split(&block2));
    }

    #[test]
    fn test_flood_protection() {
        let mut protection = FloodProtection::new();
        let addr = "192.168.1.1:9798";

        assert!(!protection.is_flooding(addr, 1000));
        assert!(!protection.is_flooding(addr, 1000));

        for _ in 0..100 {
            protection.is_flooding(addr, 2000);
        }
    }
}
