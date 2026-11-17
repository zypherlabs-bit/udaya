//! Canonical network messages.
//!
//! All messages use the strict `ug-core` codec. Counts and byte lengths are
//! bounded before allocation, and decoding rejects trailing bytes. Blocks,
//! transactions, votes and proposals are decoded with their production
//! validators; the node still executes blocks before accepting them.

use crate::identity::PeerId;
use crate::{
    Error, Result, MAX_BLOCKS_PER_MESSAGE, MAX_COMMITS_PER_MESSAGE, MAX_FRAME_BYTES,
    MAX_LISTEN_ADDR_BYTES, MAX_LOCATOR_HASHES, MAX_PEERS_PER_MESSAGE, MAX_QC_ENCODED_BYTES,
};
use std::net::SocketAddr;
use ug_consensus::vote::{Proposal, QuorumCert, Vote, VOTE_ENCODED_LEN};
use ug_core::block::Block;
use ug_core::codec::{Decoder, Encoder};
use ug_core::crypto::Hash32;
use ug_core::tx::Transaction;
use ug_core::{ChainParams, Height};

pub const MESSAGE_VERSION: u8 = 1;

const TAG_PING: u8 = 1;
const TAG_PONG: u8 = 2;
const TAG_GET_PEERS: u8 = 3;
const TAG_PEERS: u8 = 4;
const TAG_GET_BLOCK: u8 = 5;
const TAG_BLOCK: u8 = 6;
const TAG_NEW_BLOCK: u8 = 7;
const TAG_NEW_TRANSACTION: u8 = 8;
const TAG_NEW_VOTE: u8 = 9;
const TAG_NEW_PROPOSAL: u8 = 10;
const TAG_GET_BLOCKS: u8 = 11;
const TAG_BLOCKS: u8 = 12;
const TAG_NEW_COMMIT: u8 = 13;
const TAG_GET_COMMITS: u8 = 14;
const TAG_COMMITS: u8 = 15;

/// One advertised peer address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerAddress {
    pub peer_id: PeerId,
    pub address: SocketAddr,
}

/// A finalized block together with its quorum certificate.
///
/// This is the unit of commit catch-up: the certificate is cryptographic proof
/// that at least `2f+1` validators pre-committed the block. A node validates
/// the certificate and fully executes the block before persisting either.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitData {
    pub block: Block,
    pub certificate: QuorumCert,
}

impl CommitData {
    fn encode_body(&self) -> Result<Vec<u8>> {
        let block = self.block.encode();
        let certificate = self.certificate.encode();
        let mut e = Encoder::with_capacity(block.len() + certificate.len() + 16);
        e.bytes(&block);
        e.bytes(&certificate);
        Ok(e.finish())
    }

    fn decode_body(data: &[u8], chain: &ChainParams) -> Result<Self> {
        let mut d = Decoder::new(data);
        let block = Block::decode(d.bytes(chain.max_block_bytes)?, chain)?;
        let certificate = QuorumCert::decode(d.bytes(MAX_QC_ENCODED_BYTES)?)?;
        d.finish()?;
        Ok(Self { block, certificate })
    }
}

/// Network message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Ping { nonce: u64 },
    Pong { nonce: u64 },
    GetPeers,
    Peers { peers: Vec<PeerAddress> },
    GetBlock { block_hash: Hash32 },
    Block { block: Block },
    NewBlock { block: Block },
    NewTransaction { transaction: Transaction },
    NewVote { vote: Vote },
    NewProposal { proposal: Proposal },
    GetBlocks { locator: Vec<Hash32>, stop: Hash32 },
    Blocks { blocks: Vec<Block> },
    NewCommit { commit: CommitData },
    GetCommits { from: Height, limit: u32 },
    Commits { commits: Vec<CommitData> },
}

impl Message {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut e = Encoder::with_capacity(256);
        e.u8(MESSAGE_VERSION);
        match self {
            Message::Ping { nonce } => {
                e.u8(TAG_PING);
                e.u64(*nonce);
            }
            Message::Pong { nonce } => {
                e.u8(TAG_PONG);
                e.u64(*nonce);
            }
            Message::GetPeers => e.u8(TAG_GET_PEERS),
            Message::Peers { peers } => {
                e.u8(TAG_PEERS);
                if peers.len() > MAX_PEERS_PER_MESSAGE {
                    return Err(Error::Codec("too many peer addresses".into()));
                }
                e.u32(peers.len() as u32);
                for peer in peers {
                    e.fixed(&peer.peer_id.0);
                    let address = peer.address.to_string();
                    if address.len() > MAX_LISTEN_ADDR_BYTES {
                        return Err(Error::Codec("peer address is too long".into()));
                    }
                    e.bytes(address.as_bytes());
                }
            }
            Message::GetBlock { block_hash } => {
                e.u8(TAG_GET_BLOCK);
                e.fixed(block_hash);
            }
            Message::Block { block } => {
                e.u8(TAG_BLOCK);
                e.bytes(&block.encode());
            }
            Message::NewBlock { block } => {
                e.u8(TAG_NEW_BLOCK);
                e.bytes(&block.encode());
            }
            Message::NewTransaction { transaction } => {
                e.u8(TAG_NEW_TRANSACTION);
                e.bytes(&transaction.encode());
            }
            Message::NewVote { vote } => {
                e.u8(TAG_NEW_VOTE);
                e.bytes(&vote.encode());
            }
            Message::NewProposal { proposal } => {
                e.u8(TAG_NEW_PROPOSAL);
                e.bytes(&proposal.encode());
            }
            Message::GetBlocks { locator, stop } => {
                e.u8(TAG_GET_BLOCKS);
                if locator.len() > MAX_LOCATOR_HASHES {
                    return Err(Error::Codec("locator is too long".into()));
                }
                e.u32(locator.len() as u32);
                for hash in locator {
                    e.fixed(hash);
                }
                e.fixed(stop);
            }
            Message::Blocks { blocks } => {
                e.u8(TAG_BLOCKS);
                if blocks.len() > MAX_BLOCKS_PER_MESSAGE {
                    return Err(Error::Codec("too many blocks in one message".into()));
                }
                e.u32(blocks.len() as u32);
                for block in blocks {
                    e.bytes(&block.encode());
                }
            }
            Message::NewCommit { commit } => {
                e.u8(TAG_NEW_COMMIT);
                e.bytes(&commit.encode_body()?);
            }
            Message::GetCommits { from, limit } => {
                e.u8(TAG_GET_COMMITS);
                if *limit == 0 || *limit > MAX_COMMITS_PER_MESSAGE as u32 {
                    return Err(Error::Codec("invalid commit request limit".into()));
                }
                e.u64(*from);
                e.u32(*limit);
            }
            Message::Commits { commits } => {
                e.u8(TAG_COMMITS);
                if commits.len() > MAX_COMMITS_PER_MESSAGE {
                    return Err(Error::Codec("too many commits in one message".into()));
                }
                e.u32(commits.len() as u32);
                for commit in commits {
                    e.bytes(&commit.encode_body()?);
                }
            }
        }
        let bytes = e.finish();
        if bytes.len() > MAX_FRAME_BYTES {
            return Err(Error::FrameTooLarge(bytes.len()));
        }
        Ok(bytes)
    }

    pub fn decode(data: &[u8], chain: &ChainParams) -> Result<Self> {
        if data.len() > MAX_FRAME_BYTES {
            return Err(Error::FrameTooLarge(data.len()));
        }
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        if version != MESSAGE_VERSION {
            return Err(Error::Codec(format!(
                "unsupported message version {version}"
            )));
        }
        let tag = d.u8()?;
        let message = match tag {
            TAG_PING => Message::Ping { nonce: d.u64()? },
            TAG_PONG => Message::Pong { nonce: d.u64()? },
            TAG_GET_PEERS => Message::GetPeers,
            TAG_PEERS => {
                let count = d.len(MAX_PEERS_PER_MESSAGE)?;
                let mut peers = Vec::with_capacity(count);
                for _ in 0..count {
                    let peer_id = PeerId(d.fixed::<32>()?);
                    let address_bytes = d.bytes(MAX_LISTEN_ADDR_BYTES)?;
                    let address = std::str::from_utf8(address_bytes)
                        .map_err(|_| Error::Codec("peer address is not UTF-8".into()))?
                        .parse::<SocketAddr>()
                        .map_err(|_| Error::Codec("peer address is not a socket address".into()))?;
                    if address.port() == 0 {
                        return Err(Error::Codec("peer address has port zero".into()));
                    }
                    peers.push(PeerAddress { peer_id, address });
                }
                Message::Peers { peers }
            }
            TAG_GET_BLOCK => Message::GetBlock {
                block_hash: d.fixed::<32>()?,
            },
            TAG_BLOCK => {
                let bytes = d.bytes(chain.max_block_bytes)?;
                Message::Block {
                    block: Block::decode(bytes, chain)?,
                }
            }
            TAG_NEW_BLOCK => {
                let bytes = d.bytes(chain.max_block_bytes)?;
                Message::NewBlock {
                    block: Block::decode(bytes, chain)?,
                }
            }
            TAG_NEW_TRANSACTION => {
                let bytes = d.bytes(chain.max_tx_bytes)?;
                Message::NewTransaction {
                    transaction: Transaction::decode(bytes, chain)?,
                }
            }
            TAG_NEW_VOTE => {
                let bytes = d.bytes(VOTE_ENCODED_LEN)?;
                Message::NewVote {
                    vote: Vote::decode(bytes)?,
                }
            }
            TAG_NEW_PROPOSAL => {
                let bytes = d.bytes(chain.max_block_bytes.saturating_add(512))?;
                Message::NewProposal {
                    proposal: Proposal::decode(bytes, chain)?,
                }
            }
            TAG_GET_BLOCKS => {
                let count = d.len(MAX_LOCATOR_HASHES)?;
                let mut locator = Vec::with_capacity(count);
                for _ in 0..count {
                    locator.push(d.fixed::<32>()?);
                }
                Message::GetBlocks {
                    locator,
                    stop: d.fixed::<32>()?,
                }
            }
            TAG_BLOCKS => {
                let count = d.len(MAX_BLOCKS_PER_MESSAGE)?;
                let mut blocks = Vec::with_capacity(count);
                for _ in 0..count {
                    let bytes = d.bytes(chain.max_block_bytes)?;
                    blocks.push(Block::decode(bytes, chain)?);
                }
                Message::Blocks { blocks }
            }
            TAG_NEW_COMMIT => {
                let bytes = d.bytes(chain.max_block_bytes.saturating_add(MAX_QC_ENCODED_BYTES))?;
                Message::NewCommit {
                    commit: CommitData::decode_body(bytes, chain)?,
                }
            }
            TAG_GET_COMMITS => {
                let from = d.u64()?;
                let limit = d.u32()?;
                if limit == 0 || limit > MAX_COMMITS_PER_MESSAGE as u32 {
                    return Err(Error::Codec("invalid commit request limit".into()));
                }
                Message::GetCommits { from, limit }
            }
            TAG_COMMITS => {
                let count = d.len(MAX_COMMITS_PER_MESSAGE)?;
                let mut commits = Vec::with_capacity(count);
                for _ in 0..count {
                    let bytes =
                        d.bytes(chain.max_block_bytes.saturating_add(MAX_QC_ENCODED_BYTES))?;
                    commits.push(CommitData::decode_body(bytes, chain)?);
                }
                Message::Commits { commits }
            }
            other => {
                return Err(Error::Codec(format!("unknown message tag {other}")));
            }
        };
        d.finish()?;
        Ok(message)
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..1024)) {
            let chain = ChainParams::devnet();
            let _ = Message::decode(&bytes, &chain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use ug_consensus::vote::{Proposal, VoteType};
    use ug_core::block::{BlockHeader, BLOCK_VERSION};
    use ug_core::crypto::{self, public_key};
    use ug_core::tx::{OutPoint, TxIn, TxOut};

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn block(chain: &ChainParams, height: u64) -> Block {
        let proposer = public_key(&key(1));
        let header = BlockHeader {
            version: BLOCK_VERSION,
            height,
            previous_hash: [height as u8; 32],
            timestamp: 1_700_000_000 + height,
            tx_root: crypto::merkle_root(&[]),
            state_root: [0xAB; 32],
            proposer,
            round: 0,
        };
        let _ = chain;
        Block {
            header,
            transactions: vec![],
        }
    }

    fn transaction() -> Transaction {
        Transaction::new(
            vec![TxIn::new(OutPoint::new([1u8; 32], 0))],
            vec![TxOut::new(900_000, public_key(&key(2)))],
            100_000,
        )
    }

    #[test]
    fn simple_messages_roundtrip() {
        let chain = chain();
        for message in [
            Message::Ping { nonce: 42 },
            Message::Pong { nonce: 43 },
            Message::GetPeers,
            Message::GetBlock {
                block_hash: [7u8; 32],
            },
            Message::GetBlocks {
                locator: vec![[1u8; 32], [2u8; 32]],
                stop: [3u8; 32],
            },
        ] {
            let bytes = message.encode().unwrap();
            assert_eq!(Message::decode(&bytes, &chain).unwrap(), message);
        }
    }

    #[test]
    fn peer_and_block_messages_roundtrip() {
        let chain = chain();
        let peers = Message::Peers {
            peers: vec![PeerAddress {
                peer_id: PeerId([1u8; 32]),
                address: "127.0.0.1:29798".parse().unwrap(),
            }],
        };
        assert_eq!(
            Message::decode(&peers.encode().unwrap(), &chain).unwrap(),
            peers
        );

        let block = block(&chain, 1);
        for message in [
            Message::Block {
                block: block.clone(),
            },
            Message::NewBlock {
                block: block.clone(),
            },
            Message::Blocks {
                blocks: vec![block.clone()],
            },
        ] {
            assert_eq!(
                Message::decode(&message.encode().unwrap(), &chain).unwrap(),
                message
            );
        }
    }

    #[test]
    fn transaction_and_consensus_messages_roundtrip() {
        let chain = chain();
        let tx = Message::NewTransaction {
            transaction: transaction(),
        };
        assert_eq!(Message::decode(&tx.encode().unwrap(), &chain).unwrap(), tx);

        let vote = Vote::new_signed(&chain, VoteType::PreVote, 1, 0, [5u8; 32], &key(1)).unwrap();
        let message = Message::NewVote { vote };
        assert_eq!(
            Message::decode(&message.encode().unwrap(), &chain).unwrap(),
            message
        );

        let proposal = Proposal::new_signed(&chain, block(&chain, 1), &key(1)).unwrap();
        let message = Message::NewProposal { proposal };
        assert_eq!(
            Message::decode(&message.encode().unwrap(), &chain).unwrap(),
            message
        );
    }

    #[test]
    fn oversized_counts_trailing_bytes_and_truncation_are_rejected() {
        let chain = chain();

        let mut e = Encoder::new();
        e.u8(MESSAGE_VERSION);
        e.u8(TAG_PEERS);
        e.u32(MAX_PEERS_PER_MESSAGE as u32 + 1);
        assert!(Message::decode(&e.finish(), &chain).is_err());

        let bytes = Message::GetPeers.encode().unwrap();
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Message::decode(&trailing, &chain).is_err());
        assert!(Message::decode(&bytes[..bytes.len() - 1], &chain).is_err());

        let mut e = Encoder::new();
        e.u8(MESSAGE_VERSION);
        e.u8(TAG_NEW_TRANSACTION);
        e.bytes(&[0u8; 8]);
        assert!(Message::decode(&e.finish(), &chain).is_err());
    }

    #[test]
    fn unknown_version_and_tag_are_rejected() {
        let chain = chain();
        let mut e = Encoder::new();
        e.u8(9);
        e.u8(TAG_GET_PEERS);
        assert!(Message::decode(&e.finish(), &chain).is_err());

        let mut e = Encoder::new();
        e.u8(MESSAGE_VERSION);
        e.u8(200);
        assert!(Message::decode(&e.finish(), &chain).is_err());
    }
}
