//! P2P codec and Noise handshake benchmarks.
//!
//! The handshake benchmark performs a complete Noise XX handshake plus both
//! Ed25519 identity Hello exchanges over real loopback TCP sockets. It includes
//! the loopback connect/accept setup, so it is an upper bound on production
//! handshake latency.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ed25519_dalek::SigningKey;
use std::net::SocketAddr;
use ug_consensus::vote::{QuorumCert, Vote, VoteType};
use ug_consensus::ValidatorSet;
use ug_core::block::{Block, BlockHeader, BLOCK_VERSION};
use ug_core::crypto::{self, Hash32};
use ug_core::genesis::{create_config, Allocation, Genesis};
use ug_core::tx::{OutPoint, Transaction, TxIn, TxOut};
use ug_core::ChainParams;
use ug_net::{CommitData, LocalIdentity, Message, NetworkConfig};

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn pubkey(seed: u8) -> Hash32 {
    crypto::public_key(&key(seed))
}

fn fixture() -> (ChainParams, Genesis, Transaction, Block, QuorumCert) {
    let genesis = Genesis::build(create_config(
        1_700_000_000,
        vec![Allocation {
            recipient: pubkey(1),
            value: 1_000_000,
        }],
        (200..204).map(pubkey).collect(),
    ))
    .unwrap();
    let chain = genesis.chain_params();
    let funding = genesis.block().transactions[0].txid(&chain);
    let mut tx = Transaction::new(
        vec![TxIn::new(OutPoint::new(funding, 0))],
        vec![TxOut::new(998_000, pubkey(2))],
        2_000,
    );
    tx.sign_input_with_chain(0, &key(1), &chain).unwrap();

    let block = Block {
        header: BlockHeader {
            version: BLOCK_VERSION,
            height: 1,
            previous_hash: genesis.hash(),
            timestamp: genesis.block().header.timestamp + 1,
            tx_root: crypto::merkle_root(&[tx.txid(&chain)]),
            state_root: genesis.block().header.state_root,
            proposer: pubkey(200),
            round: 0,
        },
        transactions: vec![tx.clone()],
    };
    let validators = ValidatorSet::new((200..204).map(pubkey)).unwrap();
    let block_hash = block.hash(&chain);
    let votes: Vec<Vote> = (200..204)
        .map(|seed| {
            Vote::new_signed(&chain, VoteType::PreCommit, 1, 0, block_hash, &key(seed)).unwrap()
        })
        .collect();
    let certificate = QuorumCert::from_votes(&chain, &validators, 1, 0, block_hash, votes).unwrap();
    (chain, genesis, tx, block, certificate)
}

fn benchmark_messages(c: &mut Criterion) {
    let (chain, _genesis, tx, block, certificate) = fixture();
    let transaction_message = Message::NewTransaction { transaction: tx };
    let commit_message = Message::NewCommit {
        commit: CommitData { block, certificate },
    };
    let tx_bytes = transaction_message.encode().unwrap();
    let commit_bytes = commit_message.encode().unwrap();

    let mut group = c.benchmark_group("p2p_message");
    group.bench_function("new_transaction_encode", |b| {
        b.iter(|| black_box(transaction_message.encode().unwrap()))
    });
    group.bench_function("new_transaction_decode", |b| {
        b.iter(|| black_box(Message::decode(&tx_bytes, &chain).unwrap()))
    });
    group.bench_function("new_commit_encode", |b| {
        b.iter(|| black_box(commit_message.encode().unwrap()))
    });
    group.bench_function("new_commit_decode", |b| {
        b.iter(|| black_box(Message::decode(&commit_bytes, &chain).unwrap()))
    });
    group.finish();
}

fn benchmark_handshake(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let chain = ChainParams::devnet();
    let server = NetworkConfig::new(
        chain.clone(),
        [7u8; 32],
        LocalIdentity::from_seed(&[21u8; 32]),
        "127.0.0.1:41001".parse::<SocketAddr>().unwrap(),
    );
    let client = NetworkConfig::new(
        chain,
        [7u8; 32],
        LocalIdentity::from_seed(&[22u8; 32]),
        "127.0.0.1:41002".parse::<SocketAddr>().unwrap(),
    );

    c.bench_function("noise_xx_handshake_loopback", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                let address = listener.local_addr().unwrap();
                let server = server.clone();
                let accept = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.unwrap();
                    ug_net::accept(stream, &server).await.unwrap()
                });
                let outbound = ug_net::connect(&client, address).await.unwrap();
                let inbound = accept.await.unwrap();
                black_box((outbound, inbound));
            })
        })
    });
}

criterion_group!(benches, benchmark_messages, benchmark_handshake);
criterion_main!(benches);
