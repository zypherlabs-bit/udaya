//! Node-level benchmarks: full proposal verification and durable snapshot
//! writes. The snapshot benchmark includes the fsync performed before a state
//! snapshot becomes visible.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use ed25519_dalek::SigningKey;
use ug_consensus::ProposalVerifier;
use ug_core::block::{Block, BlockHeader, BLOCK_VERSION};
use ug_core::crypto::{self, Hash32};
use ug_core::genesis::{create_config, Allocation, Genesis};
use ug_core::state::{Utxo, UtxoSet};
use ug_core::tx::{OutPoint, Transaction, TxIn, TxOut};
use ug_core::ChainParams;
use ug_node::store::{StateSnapshot, StorePaths};
use ug_node::verifier::NodeVerifier;

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn pubkey(seed: u8) -> Hash32 {
    crypto::public_key(&key(seed))
}

fn genesis_with_utxos(count: usize) -> (ChainParams, Genesis) {
    let allocations = (0..count)
        .map(|_| Allocation {
            recipient: pubkey(1),
            value: 1_000_000,
        })
        .collect();
    let genesis = Genesis::build(create_config(
        1_700_000_000,
        allocations,
        (200..204).map(pubkey).collect(),
    ))
    .unwrap();
    let chain = genesis.chain_params();
    (chain, genesis)
}

fn child_block(chain: &ChainParams, genesis: &Genesis, tx_count: usize) -> Block {
    let funding = genesis.block().transactions[0].txid(chain);
    let mut transactions = Vec::new();
    let mut state = genesis.state().clone();
    for index in 0..tx_count {
        let mut tx = Transaction::new(
            vec![TxIn::new(OutPoint::new(funding, index as u32))],
            vec![TxOut::new(999_000, pubkey(2))],
            1_000,
        );
        tx.sign_input_with_chain(0, &key(1), chain).unwrap();
        state.apply_transaction(chain, &tx, 1).unwrap();
        transactions.push(tx);
    }
    let txids: Vec<Hash32> = transactions.iter().map(|tx| tx.txid(chain)).collect();
    Block {
        header: BlockHeader {
            version: BLOCK_VERSION,
            height: 1,
            previous_hash: genesis.hash(),
            timestamp: genesis.block().header.timestamp + 1,
            tx_root: crypto::merkle_root(&txids),
            state_root: state.state_root(chain),
            proposer: pubkey(200),
            round: 0,
        },
        transactions,
    }
}

fn benchmark_full_verification(c: &mut Criterion) {
    let tx_count = 100;
    let (chain, genesis) = genesis_with_utxos(tx_count);
    let block = child_block(&chain, &genesis, tx_count);
    let parent = genesis.block().header;
    let verifier = NodeVerifier {
        chain: &chain,
        parent: &parent,
        parent_state: genesis.state(),
        now_unix: block.header.timestamp,
    };

    let mut group = c.benchmark_group("node_proposal_verification");
    group.throughput(Throughput::Elements(tx_count as u64));
    group.bench_function("verify_100_transactions", |b| {
        b.iter(|| {
            let _: () = verifier.verify(&chain, &block).unwrap();
            black_box(())
        })
    });
    group.finish();
}

fn benchmark_snapshot_save(c: &mut Criterion) {
    let chain = ChainParams::devnet();
    let dir = tempfile::tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.ensure().unwrap();
    let mut state = UtxoSet::new();
    for index in 0..1000u32 {
        state
            .add_utxo(
                OutPoint::new([(index % 251) as u8; 32], index),
                Utxo {
                    value: 1_000,
                    owner: pubkey(1),
                    height: 1,
                    is_genesis: false,
                },
            )
            .unwrap();
    }
    let genesis_hash = [3u8; 32];
    let block_hash = [4u8; 32];

    c.bench_function("state_snapshot_save_1000_utxos_fsync", |b| {
        b.iter(|| {
            StateSnapshot::save(&paths, &chain, &genesis_hash, 1, block_hash, &state).unwrap();
            black_box(())
        })
    });
}

criterion_group!(
    benches,
    benchmark_full_verification,
    benchmark_snapshot_save
);
criterion_main!(benches);
