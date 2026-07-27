use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup};
use udaya_core::{consensus::ConsensusEngine, consensus::ConsensusParams, types::Transaction, validation::UTXOSet};
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;
use udaya_mempool::Mempool;
use udaya_wallet::Wallet;
use std::time::Instant;

/// Benchmark block validation throughput
fn bench_block_validation(c: &mut BenchmarkGroup) {
    let consensus = ConsensusEngine::new(ConsensusParams::default());
    let utxo_set = UTXOSet::new();
    let genesis = consensus.create_genesis_block();

    c.bench_function("block_validation", |b| {
        b.iter(|| {
            consensus.verify_block(&genesis, &utxo_set, 0).unwrap();
        })
    });
}

/// Benchmark transaction creation and validation
fn bench_transaction_validation(c: &mut BenchmarkGroup) {
    let consensus = ConsensusEngine::new(ConsensusParams::default());

    c.bench_function("transaction_creation", |b| {
        b.iter(|| {
            consensus.create_test_transaction();
        })
    });
}

/// Benchmark mempool operations
fn bench_mempool_operations(c: &mut BenchmarkGroup) {
    let consensus = ConsensusEngine::new(ConsensusParams::default());
    let mempool = Mempool::new(
        udaya_mempool::MempoolConfig::default(),
        consensus.clone(),
    );

    c.bench_function("mempool_submit", |b| {
        b.iter(|| {
            let tx = consensus.create_test_transaction();
            let _ = mempool.submit_transaction(tx, 0, 0);
        })
    });
}

/// Benchmark wallet address generation
fn bench_wallet_operations(c: &mut BenchmarkGroup) {
    let wallet = Wallet::new("Bench Wallet", "testnet");

    c.bench_function("wallet_generate_address", |b| {
        b.iter(|| {
            wallet.generate_address();
        })
    });
}

/// Benchmark UTXO set operations
fn bench_utxo_operations(c: &mut BenchmarkGroup) {
    let mut utxo_set = UTXOSet::new();
    let consensus = ConsensusEngine::new(ConsensusParams::default());
    let genesis = consensus.create_genesis_block();

    c.bench_function("utxo_apply_coinbase", |b| {
        b.iter(|| {
            let mut utxo = UTXOSet::new();
            if let Some(coinbase) = genesis.coinbase_tx() {
                utxo.apply_coinbase(coinbase, &coinbase.txid(), 0);
            }
        })
    });
}

/// Benchmark database operations
fn bench_db_operations(c: &mut BenchmarkGroup) {
    let temp_dir = std::env::temp_dir().join(format!("udaya_bench_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let config = StorageConfig {
        data_dir: temp_dir.to_str().unwrap().to_string(),
        db_cache_size_mb: 64,
        max_open_files: 100,
        enable_compression: false,
        prune_blocks: false,
        prune_target_gb: 1,
    };

    if let Ok(db) = BlockchainDB::open(&config) {
        let genesis = ConsensusEngine::new(ConsensusParams::default()).create_genesis_block();

        c.bench_function("db_store_block", |b| {
            b.iter(|| {
                let _ = db.store_block(&genesis, 0);
            })
        });

        c.bench_function("db_get_chain_height", |b| {
            b.iter(|| {
                let _ = db.get_chain_height();
            })
        });
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Benchmark hashing operations (SHA-256d)
fn bench_hash_operations(c: &mut BenchmarkGroup) {
    use udaya_core::types::BlockHash;

    c.bench_function("hash_computation", |b| {
        b.iter(|| {
            let data = [0u8; 32];
            let _ = BlockHash::hash(&data);
        })
    });
}

/// Benchmark merkle root computation
fn bench_merkle_operations(c: &mut BenchmarkGroup) {
    use udaya_core::types::{Block, Txid, BlockHeader};
    use std::sync::Arc;

    // Create dummy transactions
    let txs: Vec<Arc<Transaction>> = (0..100)
        .map(|i| {
            let mut tx = Transaction {
                version: 1,
                inputs: vec![],
                outputs: vec![],
                lock_time: 0,
            };
            tx
        })
        .collect();

    c.bench_function("merkle_root_100tx", |b| {
        b.iter(|| {
            let txids: Vec<Txid> = txs.iter().map(|_| Txid([0u8; 32])).collect();
            let _ = udaya_core::types::MerkleRoot::compute(&txids);
        })
    });
}

/// Benchmark signature verification
fn bench_signature_operations(c: &mut BenchmarkGroup) {
    let wallet = Wallet::new("Bench", "testnet");
    let address = wallet.generate_address();

    c.bench_function("signature_verification", |b| {
        b.iter(|| {
            let message = b"test message";
            let _ = wallet.sign_message(message);
        })
    });
}

/// Run a comprehensive throughput test
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.sample_size(10);
    
    bench_block_validation(&group);
    bench_transaction_validation(&group);
    bench_mempool_operations(&group);
    bench_wallet_operations(&group);
    bench_utxo_operations(&mut group.clone());
    bench_db_operations(&mut group.clone());
    bench_hash_operations(&mut group.clone());
    bench_merkle_operations(&mut group.clone());
    bench_signature_operations(&group);
    
    group.finish();
}

/// Run latency-focused benchmarks
fn bench_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");
    group.sample_size(1000);
    group.measurement_time(std::time::Duration::from_secs(10));
    
    bench_block_validation(&mut group);
    bench_transaction_validation(&mut group);
    
    group.finish();
}

criterion_group!(benches, bench_throughput, bench_latency);
criterion_main!(benches);