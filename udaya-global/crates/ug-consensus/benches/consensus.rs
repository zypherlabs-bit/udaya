//! Consensus signature, certificate and codec benchmarks.
//!
//! Signatures are real Ed25519 signatures over chain-bound digests and
//! certificates are built through the production verifier.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ed25519_dalek::SigningKey;
use ug_consensus::vote::{QuorumCert, Vote, VoteType, NIL_HASH};
use ug_consensus::ValidatorSet;
use ug_core::crypto;
use ug_core::ChainParams;

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn fixture() -> (ChainParams, ValidatorSet, Vec<SigningKey>) {
    let chain = ChainParams::devnet();
    let keys: Vec<SigningKey> = (1..=4).map(key).collect();
    let set = ValidatorSet::new(keys.iter().map(crypto::public_key)).unwrap();
    (chain, set, keys)
}

fn benchmark_votes(c: &mut Criterion) {
    let (chain, set, keys) = fixture();
    let block_hash = [9u8; 32];
    let votes: Vec<Vote> = keys
        .iter()
        .map(|key| Vote::new_signed(&chain, VoteType::PreCommit, 1, 0, block_hash, key).unwrap())
        .collect();
    let encoded = votes[0].encode();

    let mut group = c.benchmark_group("vote");
    group.bench_function("sign", |b| {
        b.iter(|| {
            black_box(
                Vote::new_signed(&chain, VoteType::PreCommit, 1, 0, block_hash, &keys[0]).unwrap(),
            )
        })
    });
    group.bench_function("verify", |b| {
        b.iter(|| {
            votes[0].verify(&chain, &set).unwrap();
            black_box(())
        })
    });
    group.bench_function("decode", |b| {
        b.iter(|| black_box(Vote::decode(&encoded).unwrap()))
    });
    group.finish();
}

fn benchmark_certificate(c: &mut Criterion) {
    let (chain, set, keys) = fixture();
    let block_hash = [9u8; 32];
    let votes: Vec<Vote> = keys
        .iter()
        .take(set.quorum())
        .map(|key| Vote::new_signed(&chain, VoteType::PreCommit, 1, 0, block_hash, key).unwrap())
        .collect();
    let certificate =
        QuorumCert::from_votes(&chain, &set, 1, 0, block_hash, votes.clone()).unwrap();
    let encoded = certificate.encode();

    let mut group = c.benchmark_group("quorum_certificate_4_validators");
    group.bench_function("build_3_votes", |b| {
        b.iter(|| {
            black_box(
                QuorumCert::from_votes(&chain, &set, 1, 0, block_hash, votes.clone()).unwrap(),
            )
        })
    });
    group.bench_function("verify", |b| {
        b.iter(|| {
            certificate.verify(&chain, &set).unwrap();
            black_box(())
        })
    });
    group.bench_function("decode_and_verify", |b| {
        b.iter(|| {
            let decoded = QuorumCert::decode(&encoded).unwrap();
            decoded.verify(&chain, &set).unwrap();
            black_box(())
        })
    });
    group.finish();
}

fn benchmark_nil_vote_rejection(c: &mut Criterion) {
    let (chain, set, keys) = fixture();
    let vote = Vote::new_signed(&chain, VoteType::PreVote, 1, 0, NIL_HASH, &keys[0]).unwrap();
    c.bench_function("nil_vote_verify", |b| {
        b.iter(|| {
            vote.verify(&chain, &set).unwrap();
            black_box(())
        })
    });
}

criterion_group!(
    benches,
    benchmark_votes,
    benchmark_certificate,
    benchmark_nil_vote_rejection
);
criterion_main!(benches);
