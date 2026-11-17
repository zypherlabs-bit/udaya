//! End-to-end cluster tests over real TCP sockets, Noise handshakes and the
//! production consensus/storage stack.
//!
//! These tests deliberately use four independent data directories and real
//! validator key files. Nothing is mocked: blocks are signed, votes are
//! journaled, certificates are verified and state is recomputed on every node.

// Tests mutate default configs field-by-field for clarity; struct-literal updates
// would obscure the single setting under test.
#![allow(clippy::field_reassign_with_default)]

use ed25519_dalek::SigningKey;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ug_core::crypto::{self, Hash32};
use ug_core::genesis::{create_config, Allocation, Genesis};
use ug_core::tx::{OutPoint, Transaction, TxIn, TxOut};
use ug_core::ChainParams;
use ug_node::config::{ConsensusSettings, NodeConfig};
use ug_node::store::StorePaths;
use ug_node::{read_disk_status, save_seed, verify_data_dir, Node, NodeHandle, NodeStatus};

const VALIDATORS: [u8; 4] = [11, 12, 13, 14];
const FUNDING: u8 = 42;
const RECIPIENT: u8 = 77;
const GENESIS_VALUE: u64 = 1_000_000_000;
const WAIT_TIMEOUT: Duration = Duration::from_secs(120);

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

struct Cluster {
    _dir: tempfile::TempDir,
    genesis: Genesis,
    chain: ChainParams,
    dirs: Vec<PathBuf>,
    handles: Vec<Option<NodeHandle>>,
}

impl Cluster {
    async fn start() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let validator_keys: Vec<Hash32> = VALIDATORS
            .iter()
            .map(|seed| crypto::public_key(&key(*seed)))
            .collect();
        let genesis = Genesis::build(create_config(
            unix_now(),
            vec![Allocation {
                recipient: crypto::public_key(&key(FUNDING)),
                value: GENESIS_VALUE,
            }],
            validator_keys,
        ))
        .expect("genesis");
        let chain = genesis.chain_params();

        let mut dirs = Vec::new();
        for (index, validator) in VALIDATORS.iter().enumerate() {
            let data_dir = dir.path().join(format!("node-{index}"));
            std::fs::create_dir_all(&data_dir).expect("data dir");
            genesis
                .save(&StorePaths::new(&data_dir).genesis)
                .expect("save genesis");
            save_seed(&data_dir.join("validator.key"), &[*validator; 32]).expect("save key");
            dirs.push(data_dir);
        }

        let mut handles: Vec<Option<NodeHandle>> = Vec::new();
        let mut addresses = Vec::new();
        for (index, data_dir) in dirs.iter().enumerate() {
            let mut config = NodeConfig::default();
            config.listen = "127.0.0.1:0".parse().unwrap();
            config.ping_interval_secs = 1;
            config.peer_timeout_secs = 10;
            config.reconnect_interval_secs = 1;
            config.genesis_hash = Some(hex::encode(genesis.hash()));
            config.consensus = ConsensusSettings {
                propose_timeout_ms: 1_000,
                prevote_timeout_ms: 1_000,
                precommit_timeout_ms: 1_000,
            };
            if index > 0 {
                config.bootstrap.push(addresses[0]);
            }
            config
                .save(&StorePaths::new(data_dir).node_config)
                .expect("save node config");
            let node = Node::open(data_dir, &data_dir.join("validator.key"), config)
                .await
                .expect("open node");
            addresses.push(node.listen_addr());
            handles.push(Some(node.spawn()));
        }

        Self {
            _dir: dir,
            genesis,
            chain,
            dirs,
            handles,
        }
    }

    fn handle(&self, index: usize) -> &NodeHandle {
        self.handles[index].as_ref().expect("running node")
    }

    async fn heights(&self) -> Vec<u64> {
        let mut heights = Vec::new();
        for handle in &self.handles {
            let Some(handle) = handle else {
                heights.push(0);
                continue;
            };
            heights.push(handle.status().await.expect("status").height);
        }
        heights
    }

    async fn wait_equal_height_at_least(&self, target: u64) {
        let deadline = std::time::Instant::now() + WAIT_TIMEOUT;
        loop {
            let heights = self.heights().await;
            if heights.iter().all(|height| *height >= target)
                && heights.iter().all(|height| *height == heights[0])
            {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "nodes did not converge to height >= {target}: {heights:?}"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_supply(&self, expected: u64) {
        let deadline = std::time::Instant::now() + WAIT_TIMEOUT;
        loop {
            let mut all = true;
            for handle in self.handles.iter().flatten() {
                let status = handle.status().await.expect("status");
                if status.supply != Some(expected) {
                    all = false;
                    break;
                }
            }
            if all {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "supply {expected} was not reached on all nodes"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Snapshot all nodes at the same height. Because the chain keeps
    /// committing empty blocks, comparing statuses captured a second apart
    /// would compare different heights; retry until one height is common.
    async fn consistent_statuses(&self) -> Vec<NodeStatus> {
        let deadline = std::time::Instant::now() + WAIT_TIMEOUT;
        loop {
            let mut statuses = Vec::new();
            for handle in self.handles.iter().flatten() {
                statuses.push(handle.status().await.expect("status"));
            }
            let height = statuses[0].height;
            if statuses.iter().all(|status| status.height == height) {
                return statuses;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "nodes did not share a height: {:?}",
                statuses
                    .iter()
                    .map(|status| status.height)
                    .collect::<Vec<_>>()
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn stop(&mut self, index: usize) {
        if let Some(handle) = self.handles[index].take() {
            handle.shutdown().await.expect("clean shutdown");
        }
    }

    async fn restart(&mut self, index: usize, bootstrap: std::net::SocketAddr) {
        let data_dir = self.dirs[index].clone();
        let paths = StorePaths::new(&data_dir);
        let mut config = NodeConfig::load(&paths.node_config).expect("load config");
        config.genesis_hash = Some(hex::encode(self.genesis.hash()));
        if !config.bootstrap.contains(&bootstrap) {
            config.bootstrap.push(bootstrap);
        }
        let node = Node::open(&data_dir, &data_dir.join("validator.key"), config)
            .await
            .expect("reopen node");
        self.handles[index] = Some(node.spawn());
    }

    async fn shutdown_all(&mut self) {
        for index in 0..self.handles.len() {
            self.stop(index).await;
        }
    }
}

fn funded_outpoint(genesis: &Genesis, chain: &ChainParams) -> (OutPoint, u64) {
    let txid = genesis.block().transactions[0].txid(chain);
    let outpoint = OutPoint::new(txid, 0);
    let utxo = genesis.state().get(&outpoint).expect("funding utxo");
    (outpoint, utxo.value)
}

fn build_spend(
    chain: &ChainParams,
    outpoint: OutPoint,
    input_value: u64,
    recipient: Hash32,
    signing_seed: u8,
) -> Transaction {
    let mut tx = Transaction::new(
        vec![TxIn::new(outpoint)],
        vec![TxOut::new(input_value - 1, recipient)],
        0,
    );
    // Fee is part of the signed body, so fix it before signing.
    let fee = tx.encoded_size() as u64 * chain.min_fee_per_byte;
    tx.fee = fee;
    tx.outputs[0].value = input_value - fee;
    tx.sign_input_with_chain(0, &key(signing_seed), chain)
        .expect("sign");
    tx
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn four_validator_cluster_reaches_consensus_and_catches_up() {
    let mut cluster = Cluster::start().await;

    // 1. The cluster must commit at least one block with a full 4-node quorum.
    cluster.wait_equal_height_at_least(1).await;
    let anchor = cluster.handle(0).listen_addr;

    let genesis_status = cluster.handle(0).status().await.expect("status");
    assert_eq!(genesis_status.validators, 4);
    assert_eq!(genesis_status.supply, Some(GENESIS_VALUE));
    assert_eq!(
        genesis_status.genesis_hash,
        hex::encode(cluster.genesis.hash())
    );

    // 2. Invalid signatures and below-minimum fees are rejected before relay.
    let (outpoint, value) = funded_outpoint(&cluster.genesis, &cluster.chain);
    let mut unsigned = Transaction::new(
        vec![TxIn::new(outpoint)],
        vec![TxOut::new(value - 1, crypto::public_key(&key(RECIPIENT)))],
        0,
    );
    unsigned.fee = 1;
    let bad_fee = unsigned.clone();
    assert!(cluster.handle(0).submit_transaction(bad_fee).await.is_err());

    let mut bad_signature = build_spend(
        &cluster.chain,
        outpoint,
        value,
        crypto::public_key(&key(RECIPIENT)),
        FUNDING,
    );
    bad_signature.inputs[0].signature[0] ^= 0xFF;
    assert!(cluster
        .handle(0)
        .submit_transaction(bad_signature)
        .await
        .is_err());

    // 3. A valid transaction is included and burned-fee supply is identical
    //    on every node.
    let tx = build_spend(
        &cluster.chain,
        outpoint,
        value,
        crypto::public_key(&key(RECIPIENT)),
        FUNDING,
    );
    let fee = tx.fee;
    let height_before = cluster.heights().await[0];
    cluster
        .handle(0)
        .submit_transaction(tx)
        .await
        .expect("valid transaction accepted");
    cluster.wait_for_supply(GENESIS_VALUE - fee).await;
    cluster
        .wait_equal_height_at_least(height_before.saturating_add(1))
        .await;

    // Every node agrees on the tip, the state root and the total supply.
    let statuses = cluster.consistent_statuses().await;
    let block_hash = statuses[0].block_hash.clone();
    let state_root = statuses[0].state_root.clone();
    for status in &statuses {
        assert_eq!(status.block_hash, block_hash);
        assert_eq!(status.state_root, state_root);
        assert_eq!(status.supply, Some(GENESIS_VALUE - fee));
        assert_eq!(status.mempool_transactions, Some(0));
    }

    // 4. Re-spending the same outpoint is impossible after finality.
    let double_spend = build_spend(
        &cluster.chain,
        outpoint,
        value,
        crypto::public_key(&key(99)),
        FUNDING,
    );
    let error = cluster
        .handle(0)
        .submit_transaction(double_spend)
        .await
        .expect_err("double spend must be rejected");
    assert!(
        error.to_string().to_lowercase().contains("missing input")
            || error.to_string().to_lowercase().contains("spent"),
        "unexpected rejection: {error}"
    );

    // 5. A validator stops; the remaining three (2f+1) continue finalising.
    cluster.stop(3).await;
    let height_when_stopped = cluster.heights().await[..3].iter().copied().max().unwrap();
    let deadline = std::time::Instant::now() + WAIT_TIMEOUT;
    loop {
        let heights = cluster.heights().await;
        if heights[..3]
            .iter()
            .all(|height| *height >= height_when_stopped + 2)
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "remaining three validators did not make progress: {heights:?}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 6. The restarted validator catches up through verified certificates,
    //    without ever signing a conflicting vote.
    cluster.restart(3, anchor).await;
    let target = cluster.heights().await[..3].iter().copied().max().unwrap();
    cluster.wait_equal_height_at_least(target).await;

    let mut roots = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    for status in cluster.consistent_statuses().await {
        roots.insert(status.state_root);
        hashes.insert(status.block_hash);
        assert_eq!(status.supply, Some(GENESIS_VALUE - fee));
    }
    assert_eq!(roots.len(), 1, "state roots diverged after catch-up");
    assert_eq!(hashes.len(), 1, "block hashes diverged after catch-up");

    cluster.shutdown_all().await;

    // 7. Offline verification replays every block and checks the state root.
    let verified = verify_data_dir(&cluster.dirs[1]).expect("verify node 1");
    assert!(verified.height >= target);
    assert_eq!(verified.supply, Some(GENESIS_VALUE - fee));
    let disk = read_disk_status(&cluster.dirs[1]).expect("disk status");
    assert_eq!(disk.height, verified.height);
    assert_eq!(disk.state_root, verified.state_root);

    // 8. A corrupted state snapshot fails closed; the node refuses to start.
    let paths = StorePaths::new(&cluster.dirs[0]);
    let mut bytes = std::fs::read(&paths.state).expect("snapshot");
    bytes[20] ^= 0xFF;
    std::fs::write(&paths.state, bytes).expect("corrupt snapshot");
    let config = NodeConfig::load(&paths.node_config).expect("config");
    let error = match Node::open(
        &cluster.dirs[0],
        &cluster.dirs[0].join("validator.key"),
        config,
    )
    .await
    {
        Ok(_) => panic!("corrupt snapshot must fail closed"),
        Err(error) => error,
    };
    let text = error.to_string().to_lowercase();
    assert!(
        text.contains("snapshot") || text.contains("corrupt") || text.contains("mismatch"),
        "unexpected corruption error: {error}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn conflicting_spends_cannot_both_finalize() {
    let mut cluster = Cluster::start().await;
    cluster.wait_equal_height_at_least(1).await;

    let (outpoint, value) = funded_outpoint(&cluster.genesis, &cluster.chain);
    let recipient_a = crypto::public_key(&key(RECIPIENT));
    let recipient_b = crypto::public_key(&key(99));

    let tx_a = build_spend(&cluster.chain, outpoint, value, recipient_a, FUNDING);
    let tx_b = build_spend(&cluster.chain, outpoint, value, recipient_b, FUNDING);
    let fee = tx_a.fee;

    // Submit both conflicting spends to two independent nodes simultaneously.
    let (result_a, result_b) = tokio::join!(
        cluster.handle(0).submit_transaction(tx_a),
        cluster.handle(1).submit_transaction(tx_b),
    );
    // A node may accept either spend into its mempool; neither may be
    // finalized until a block commits exactly one of them.
    let _ = (result_a, result_b);

    // Wait until exactly one of the conflicting spends is finalized: supply
    // must drop by one fee, never by two.
    cluster.wait_for_supply(GENESIS_VALUE - fee).await;

    // All nodes must converge to the same tip, state root and supply.
    let statuses = cluster.consistent_statuses().await;
    let block_hash = statuses[0].block_hash.clone();
    let state_root = statuses[0].state_root.clone();
    for status in &statuses {
        assert_eq!(status.block_hash, block_hash, "block hashes diverged");
        assert_eq!(status.state_root, state_root, "state roots diverged");
        assert_eq!(
            status.supply,
            Some(GENESIS_VALUE - fee),
            "supply must reflect exactly one finalized spend"
        );
        assert_eq!(status.mempool_transactions, Some(0));
    }

    cluster.shutdown_all().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn node_refuses_a_foreign_genesis_pin() {
    let dir = tempfile::tempdir().unwrap();

    let validator_keys: Vec<Hash32> = VALIDATORS
        .iter()
        .map(|seed| crypto::public_key(&key(*seed)))
        .collect();
    let genesis = Genesis::build(create_config(
        unix_now(),
        vec![Allocation {
            recipient: crypto::public_key(&key(FUNDING)),
            value: GENESIS_VALUE,
        }],
        validator_keys,
    ))
    .unwrap();
    let data_dir = dir.path().join("node");
    std::fs::create_dir_all(&data_dir).unwrap();
    genesis.save(&StorePaths::new(&data_dir).genesis).unwrap();
    save_seed(&data_dir.join("validator.key"), &[VALIDATORS[0]; 32]).unwrap();
    let mut config = NodeConfig::default();
    config.listen = "127.0.0.1:0".parse().unwrap();
    config.genesis_hash = Some("00".repeat(32));
    let error = match Node::open(&data_dir, &data_dir.join("validator.key"), config).await {
        Ok(_) => panic!("pinned genesis mismatch must fail"),
        Err(error) => error,
    };
    assert!(error.to_string().to_lowercase().contains("genesis"));
}
