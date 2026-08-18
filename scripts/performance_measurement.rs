use std::time::Instant;
use udaya_core::{consensus::ConsensusEngine, consensus::ConsensusParams, validation::UTXOSet};
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;
use udaya_mempool::Mempool;
use udaya_wallet::Wallet;
use tempfile::tempdir;

fn main() {
    println!("=== Udaya Blockchain Performance Measurement ===");
    println!("Establishing baseline metrics for optimization...");

    // Measure node startup time
    measure_node_startup();

    // Measure block validation
    measure_block_validation();

    // Measure transaction processing
    measure_transaction_processing();

    // Measure database operations
    measure_database_operations();

    // Measure signature verification
    measure_signature_verification();

    println!("\n=== Performance Measurement Complete ===");
    println!("Use these results to guide optimization efforts.");
}

fn measure_node_startup() {
    println!("\n📊 Measuring Node Startup Time...");

    let start = Instant::now();

    // Simulate node initialization
    let consensus_params = ConsensusParams::default();
    let consensus = ConsensusEngine::new(consensus_params);

    let temp_dir = tempdir().unwrap();
    let storage_config = StorageConfig {
        data_dir: temp_dir.path().to_str().unwrap().to_string(),
        db_cache_size_mb: 64,
        max_open_files: 100,
        enable_compression: false,
        prune_blocks: false,
        prune_target_gb: 1,
    };

    let _db = BlockchainDB::open(&storage_config).unwrap();
    let _mempool = Mempool::new(udaya_mempool::MempoolConfig::default(), consensus.clone());
    let _wallet = Wallet::new("Performance Test Wallet", "testnet");

    let duration = start.elapsed();
    println!("  ✅ Node startup: {:?}", duration);
    println!("  📈 Startup rate: {:.2} ms", duration.as_millis());
}

fn measure_block_validation() {
    println!("\n📊 Measuring Block Validation...");

    let consensus = ConsensusEngine::new(ConsensusParams::default());
    let utxo_set = UTXOSet::new();
    let genesis = consensus.create_genesis_block();

    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        consensus.verify_block(&genesis, &utxo_set, 0).unwrap();
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    let ops_per_sec = 1_000_000_000.0 / avg_time;

    println!("  ✅ {} block validations: {:?}", iterations, duration);
    println!("  📈 Average time per validation: {:.2} µs", avg_time / 1000.0);
    println!("  🚀 Throughput: {:.0} validations/sec", ops_per_sec);
}

fn measure_transaction_processing() {
    println!("\n📊 Measuring Transaction Processing...");

    let consensus = ConsensusEngine::new(ConsensusParams::default());
    let mempool = Mempool::new(
        udaya_mempool::MempoolConfig::default(),
        consensus.clone(),
    );

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        let tx = consensus.create_test_transaction();
        let _ = mempool.submit_transaction(tx, 0, 0);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    let ops_per_sec = 1_000_000_000.0 / avg_time;

    println!("  ✅ {} transaction submissions: {:?}", iterations, duration);
    println!("  📈 Average time per transaction: {:.2} µs", avg_time / 1000.0);
    println!("  🚀 Throughput: {:.0} tx/sec", ops_per_sec);
}

fn measure_database_operations() {
    println!("\n📊 Measuring Database Operations...");

    let temp_dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: temp_dir.path().to_str().unwrap().to_string(),
        db_cache_size_mb: 64,
        max_open_files: 100,
        enable_compression: false,
        prune_blocks: false,
        prune_target_gb: 1,
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = ConsensusEngine::new(ConsensusParams::default()).create_genesis_block();

    // Measure write operations
    let start_write = Instant::now();
    let write_iterations = 10;

    for i in 0..write_iterations {
        let _ = db.store_block(&genesis, i);
    }

    let write_duration = start_write.elapsed();
    let avg_write_time = write_duration.as_millis() as f64 / write_iterations as f64;

    // Measure read operations
    let start_read = Instant::now();
    let read_iterations = 100;

    for _ in 0..read_iterations {
        let _ = db.get_chain_height();
    }

    let read_duration = start_read.elapsed();
    let avg_read_time = read_duration.as_nanos() as f64 / read_iterations as f64;

    println!("  ✅ {} block writes: {:?}", write_iterations, write_duration);
    println!("  📈 Average write time: {:.2} ms", avg_write_time);
    println!("  ✅ {} block reads: {:?}", read_iterations, read_duration);
    println!("  📈 Average read time: {:.2} µs", avg_read_time / 1000.0);
}

fn measure_signature_verification() {
    println!("\n📊 Measuring Signature Verification...");

    let wallet = Wallet::new("Performance Test", "testnet");
    let message = b"test message for performance measurement";

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        let _ = wallet.sign_message(message);
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    let ops_per_sec = 1_000_000_000.0 / avg_time;

    println!("  ✅ {} signature operations: {:?}", iterations, duration);
    println!("  📈 Average time per signature: {:.2} µs", avg_time / 1000.0);
    println!("  🚀 Throughput: {:.0} sigs/sec", ops_per_sec);
}