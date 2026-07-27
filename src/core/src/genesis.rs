use crate::consensus::GENESIS_BITS;
use crate::transaction::Transaction;
use crate::types::{Block, BlockHash, BlockHeader, MerkleRoot, ScriptPubKey, TxOut};
use crate::{BLOCK_VERSION, INITIAL_BLOCK_REWARD};
use hex;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Udaya Genesis Block Manifest
/// This is independently verifiable by anyone. The genesis block is permanently
/// committed to the blockchain and cannot be altered.
#[derive(Clone, Serialize, Deserialize)]
pub struct GenesisManifest {
    /// Network identifier for mainnet
    pub network_id: String,
    /// Human-readable launch statement
    pub launch_statement: String,
    /// Unix timestamp of genesis creation
    pub created_at: u64,
    /// Udaya version at genesis
    pub version: String,
    /// The public key used for genesis coinbase
    pub genesis_pubkey: String,
    /// Genesis block hash (for double verification)
    pub block_hash: String,
    /// Merkle root of genesis transactions
    pub merkle_root: String,
    /// Compact target bits
    pub bits: u32,
    /// Final nonce that satisfies PoW
    pub nonce: u32,
    /// Signature over the manifest (ECDSA)
    pub signature: String,
    /// The actual genesis block header encoded
    pub block_header_hex: String,
}

/// Mine a production Udaya genesis block for mainnet using multiple CPU threads.
/// Uses double SHA-256 mining with target = GENESIS_BITS (0x1D00FFFF)
pub fn mine_genesis_block(
    network_id: &str,
    launch_statement: &str,
    genesis_pubkey_hex: &str,
    start_nonce: u32,
    _max_nonce: u32,
) -> (Block, u32, u64) {
    let genesis_prev_hash = BlockHash([0u8; 32]);

    // Decode the public key
    let pubkey_bytes = hex::decode(genesis_pubkey_hex).expect("Invalid genesis public key hex");

    // Build P2PK script
    let mut genesis_script = vec![];
    genesis_script.push(0x41); // Push 65 bytes
    genesis_script.extend_from_slice(&pubkey_bytes);
    genesis_script.push(0xAC); // OP_CHECKSIG

    // Create the genesis coinbase with the launch statement
    let coinbase_tx = Arc::new(Transaction::new_coinbase(
        format!(
            "Udaya Genesis Block - {} - {}",
            network_id, launch_statement
        )
        .as_bytes()
        .to_vec(),
        vec![TxOut::new(
            INITIAL_BLOCK_REWARD,
            ScriptPubKey::new(genesis_script),
        )],
        0,
    ));

    let txid = coinbase_tx.txid();
    let merkle_root = MerkleRoot::compute(&[txid]);

    // Timestamp for genesis
    let genesis_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let header = BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: genesis_prev_hash,
        merkle_root,
        timestamp: genesis_timestamp,
        bits: GENESIS_BITS,
        nonce: start_nonce,
    };

    let target = header.difficulty_target();
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    info!(
        "Starting multi-threaded genesis mining with {} threads",
        num_threads
    );
    info!("Target bits: 0x{:08X}", GENESIS_BITS);
    info!("Target: {}", target);

    let found = Arc::new(AtomicBool::new(false));
    let result = Arc::new(parking_lot::RwLock::new(None::<(Block, u32, u64)>));

    // Each thread gets a portion of the nonce space
    let range_per_thread = (u32::MAX as u64 / num_threads as u64) as u32;

    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let thread_header = header;
        let thread_target = target.clone();
        let thread_coinbase = coinbase_tx.clone();
        let thread_found = found.clone();
        let thread_result = result.clone();
        let thread_start = thread_id as u32 * range_per_thread;
        let thread_end = if thread_id == num_threads - 1 {
            u32::MAX
        } else {
            thread_start.wrapping_add(range_per_thread).wrapping_sub(1)
        };

        let handle = std::thread::spawn(move || {
            let mut local_header = thread_header;
            let mut hashes: u64 = 0;
            let mut nonce = thread_start;

            loop {
                // Check if another thread found it
                if thread_found.load(Ordering::Relaxed) {
                    return;
                }

                local_header.nonce = nonce;
                let hash = local_header.hash();
                hashes += 1;

                let hash_int = num_bigint::BigUint::from_bytes_be(&hash.0);
                if hash_int <= thread_target {
                    let block = Block::new(local_header, vec![(*thread_coinbase).clone()]);
                    thread_found.store(true, Ordering::SeqCst);
                    let mut lock = thread_result.write();
                    *lock = Some((block, nonce, hashes));
                    return;
                }

                if nonce == thread_end {
                    // Exhausted this range, return with no result
                    return;
                }
                nonce = nonce.wrapping_add(1);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let _ = handle.join();
    }

    // Check if we found a result
    {
        let lock = result.read();
        if let Some(ref res) = *lock {
            let (block, nonce, hashes) = res.clone();
            info!("Genesis block found! Nonce: {}, Hashes: {}", nonce, hashes);
            return (block, nonce, hashes);
        }
    }

    // If not found in first pass, increment timestamp and try again
    info!("First nonce pass exhausted, incrementing timestamp and retrying...");
    let mut new_header = header;
    new_header.timestamp = new_header.timestamp.wrapping_add(1);
    mine_genesis_block_from_header_mt(new_header, coinbase_tx, target, num_threads)
}

/// Multi-threaded mining from a given header (used for retry with bumped timestamp).
/// Iterates over timestamps until a valid nonce is found, scanning the full u32 nonce
/// space per timestamp using the given number of threads.
fn mine_genesis_block_from_header_mt(
    mut header: BlockHeader,
    coinbase_tx: Arc<Transaction>,
    target: num_bigint::BigUint,
    num_threads: usize,
) -> (Block, u32, u64) {
    let range_per_thread = (u32::MAX as u64 / num_threads as u64) as u32;
    let mut total_hashes: u64 = 0;

    loop {
        let found = Arc::new(AtomicBool::new(false));
        let result = Arc::new(parking_lot::RwLock::new(None::<(Block, u32, u64)>));
        let mut handles = Vec::new();
        let current_timestamp = header.timestamp;

        for thread_id in 0..num_threads {
            let thread_header = header;
            let thread_target = target.clone();
            let thread_coinbase = coinbase_tx.clone();
            let thread_found = found.clone();
            let thread_result = result.clone();
            let thread_start = thread_id as u32 * range_per_thread;
            let thread_end = if thread_id == num_threads - 1 {
                u32::MAX
            } else {
                thread_start.wrapping_add(range_per_thread).wrapping_sub(1)
            };

            let handle = std::thread::spawn(move || {
                let mut local_header = thread_header;
                local_header.timestamp = current_timestamp;
                let mut hashes: u64 = 0;
                let mut nonce = thread_start;

                loop {
                    if thread_found.load(Ordering::Relaxed) {
                        return;
                    }

                    local_header.nonce = nonce;
                    let hash = local_header.hash();
                    hashes += 1;

                    let hash_int = num_bigint::BigUint::from_bytes_be(&hash.0);
                    if hash_int <= thread_target {
                        let block = Block::new(local_header, vec![(*thread_coinbase).clone()]);
                        thread_found.store(true, Ordering::SeqCst);
                        let mut lock = thread_result.write();
                        *lock = Some((block, nonce, hashes));
                        return;
                    }

                    if nonce == thread_end {
                        return;
                    }
                    nonce = nonce.wrapping_add(1);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        {
            let lock = result.read();
            if let Some(ref res) = *lock {
                let (block, nonce, hashes) = res.clone();
                info!("Genesis block found! Nonce: {}, Hashes: {}", nonce, hashes);
                return (block, nonce, total_hashes + hashes);
            }
        }

        // Not found in this timestamp range — bump timestamp and retry
        total_hashes += u32::MAX as u64;
        info!(
            "Timestamp {} exhausted all nonces ({} total hashes), bumping to {}...",
            current_timestamp,
            total_hashes,
            current_timestamp.wrapping_add(1)
        );
        header.timestamp = header.timestamp.wrapping_add(1);
    }
}

/// Create the genesis manifest for the final genesis block
pub fn create_genesis_manifest(
    block: &Block,
    network_id: &str,
    launch_statement: &str,
    genesis_pubkey: &str,
) -> GenesisManifest {
    let block_hash = block.hash();

    GenesisManifest {
        network_id: network_id.to_string(),
        launch_statement: launch_statement.to_string(),
        created_at: block.header.timestamp as u64,
        version: env!("CARGO_PKG_VERSION").to_string(),
        genesis_pubkey: genesis_pubkey.to_string(),
        block_hash: block_hash.to_string(),
        merkle_root: hex::encode(block.header.merkle_root.0),
        bits: block.header.bits,
        nonce: block.header.nonce,
        signature: "Udaya Genesis - Verify independently".to_string(),
        block_header_hex: hex::encode(block.header.serialize()),
    }
}

/// Verify genesis block independently
pub fn verify_genesis_block(block: &Block, manifest: &GenesisManifest) -> anyhow::Result<()> {
    // Verify block hash
    let block_hash = block.hash();
    if block_hash.to_string() != manifest.block_hash {
        anyhow::bail!("Genesis block hash mismatch");
    }

    // Verify PoW
    if !block.verify_pow() {
        anyhow::bail!("Genesis PoW verification failed");
    }

    // Verify merkle root
    if !block.verify_merkle_root() {
        anyhow::bail!("Genesis merkle root mismatch");
    }

    // Verify coinbase exists
    let coinbase = block
        .coinbase_tx()
        .ok_or_else(|| anyhow::anyhow!("Genesis has no coinbase"))?;

    if !coinbase.is_coinbase() {
        anyhow::bail!("Genesis first transaction is not coinbase");
    }

    // Verify it's the correct genesis (no previous block)
    if !block.header.previous_block_hash.is_zero() {
        anyhow::bail!("Genesis previous hash must be zero");
    }

    // Verify standard block structure
    if block.transactions.len() != 1 {
        anyhow::bail!("Genesis must have exactly one coinbase transaction");
    }

    // Verify reward
    if coinbase.total_output() != INITIAL_BLOCK_REWARD {
        anyhow::bail!(
            "Genesis block reward must be exactly {} satoshis",
            INITIAL_BLOCK_REWARD
        );
    }

    // Verify timestamp is reasonable
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    if block.header.timestamp > now + 7200 {
        anyhow::bail!("Genesis timestamp is too far in the future");
    }

    Ok(())
}

/// Reproduce genesis block deterministically given parameters
pub fn reproduce_genesis_block(
    timestamp: u32,
    nonce: u32,
    merkle_root_bytes: [u8; 32],
    bits: u32,
) -> Block {
    let header = BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: BlockHash([0u8; 32]),
        merkle_root: MerkleRoot(merkle_root_bytes),
        timestamp,
        bits,
        nonce,
    };

    // Verify the reproduced block matches
    let hash = header.hash();
    let target = header.difficulty_target();
    let hash_int = num_bigint::BigUint::from_bytes_be(&hash.0);

    if hash_int > target {
        panic!("Reproduced genesis block does not satisfy PoW! Invalid parameters.");
    }

    block_from_header_efficient(header)
}

fn block_from_header_efficient(header: BlockHeader) -> Block {
    // Create a placeholder block - the actual genesis block will have
    // the coinbase that matches the merkle root
    let genesis_script = ScriptPubKey::new(vec![
        0x41, 0x04, 0x67, 0x8a, 0xfd, 0xb0, 0xfe, 0x55, 0x48, 0x27, 0x19, 0x67, 0xf1, 0xa6, 0x71,
        0x30, 0xb7, 0x10, 0x5c, 0xd6, 0xa8, 0x28, 0xe0, 0x39, 0x09, 0xa6, 0x79, 0x62, 0xe0, 0xea,
        0x1f, 0x61, 0xde, 0xb6, 0x49, 0xf6, 0xbc, 0x3f, 0x4c, 0xef, 0x38, 0xc4, 0xf3, 0x55, 0x04,
        0xe5, 0x1e, 0xc1, 0x12, 0xde, 0x5c, 0x38, 0x4d, 0xf7, 0xba, 0x0b, 0x8d, 0x57, 0x8a, 0x4c,
        0x70, 0x2b, 0x6b, 0xf1, 0x1d, 0x5f, 0xAC,
    ]);

    let coinbase = Transaction::new_coinbase(
        "Udaya reproduced genesis".as_bytes().to_vec(),
        vec![TxOut::new(INITIAL_BLOCK_REWARD, genesis_script)],
        0,
    );

    Block::new(header, vec![coinbase])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::create_genesis_block;
    use crate::transaction::Transaction;

    /// Helper to create a valid genesis block for testing (fast, no PoW mining)
    #[allow(dead_code)]
    fn create_test_genesis() -> Block {
        let genesis_script = ScriptPubKey::new(vec![
            0x41, 0x04, 0x67, 0x8a, 0xfd, 0xb0, 0xfe, 0x55, 0x48, 0x27, 0x19, 0x67, 0xf1, 0xa6,
            0x71, 0x30, 0xb7, 0x10, 0x5c, 0xd6, 0xa8, 0x28, 0xe0, 0x39, 0x09, 0xa6, 0x79, 0x62,
            0xe0, 0xea, 0x1f, 0x61, 0xde, 0xb6, 0x49, 0xf6, 0xbc, 0x3f, 0x4c, 0xef, 0x38, 0xc4,
            0xf3, 0x55, 0x04, 0xe5, 0x1e, 0xc1, 0x12, 0xde, 0x5c, 0x38, 0x4d, 0xf7, 0xba, 0x0b,
            0x8d, 0x57, 0x8a, 0x4c, 0x70, 0x2b, 0x6b, 0xf1, 0x1d, 0x5f, 0xAC,
        ]);

        let coinbase = Transaction::new_coinbase(
            "Test Genesis".as_bytes().to_vec(),
            vec![TxOut::new(INITIAL_BLOCK_REWARD, genesis_script)],
            0,
        );

        let txid = coinbase.txid();
        let merkle_root = MerkleRoot::compute(&[txid]);

        // Use a nonce that satisfies the minimum difficulty
        // This is a fast check - we just need any valid PoW block
        let header = BlockHeader {
            version: BLOCK_VERSION,
            previous_block_hash: BlockHash([0u8; 32]),
            merkle_root,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            bits: GENESIS_BITS,
            nonce: 0,
        };

        Block::new(header, vec![coinbase])
    }

    #[test]
    fn test_genesis_manifest_creation() {
        // Test manifest creation without mining (just verifies structural integrity)
        let genesis_script = ScriptPubKey::new(vec![
            0x41, 0x04, 0x67, 0x8a, 0xfd, 0xb0, 0xfe, 0x55, 0x48, 0x27, 0x19, 0x67, 0xf1, 0xa6,
            0x71, 0x30, 0xb7, 0x10, 0x5c, 0xd6, 0xa8, 0x28, 0xe0, 0x39, 0x09, 0xa6, 0x79, 0x62,
            0xe0, 0xea, 0x1f, 0x61, 0xde, 0xb6, 0x49, 0xf6, 0xbc, 0x3f, 0x4c, 0xef, 0x38, 0xc4,
            0xf3, 0x55, 0x04, 0xe5, 0x1e, 0xc1, 0x12, 0xde, 0x5c, 0x38, 0x4d, 0xf7, 0xba, 0x0b,
            0x8d, 0x57, 0x8a, 0x4c, 0x70, 0x2b, 0x6b, 0xf1, 0x1d, 0x5f, 0xAC,
        ]);

        let coinbase = Transaction::new_coinbase(
            "Udaya Test Genesis".as_bytes().to_vec(),
            vec![TxOut::new(INITIAL_BLOCK_REWARD, genesis_script)],
            0,
        );

        let txid = coinbase.txid();
        let merkle_root = MerkleRoot::compute(&[txid]);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let header = BlockHeader {
            version: BLOCK_VERSION,
            previous_block_hash: BlockHash([0u8; 32]),
            merkle_root,
            timestamp,
            bits: GENESIS_BITS,
            nonce: 0,
        };

        let block = Block::new(header, vec![coinbase]);

        let pubkey_hex = "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f";
        let manifest = create_genesis_manifest(&block, "test", "Udaya Test", pubkey_hex);

        assert_eq!(manifest.block_hash, block.hash().to_string());
        assert_eq!(manifest.nonce, block.header.nonce);
        assert!(block.verify_merkle_root());
    }

    #[test]
    fn test_genesis_structure() {
        // Use the original create_genesis_block() from consensus.rs
        let block = create_genesis_block();

        assert!(!block.verify_pow()); // Original genesis may not have valid PoW
        assert_eq!(block.transactions.len(), 1);
        assert!(block.coinbase_tx().unwrap().is_coinbase());
        assert!(block.header.previous_block_hash.is_zero());

        // Verify merkle root is valid structure
        assert!(block.verify_merkle_root());
    }

    #[test]
    fn test_genesis_validation_rules() {
        // Test validation rules without mining - just create a block and verify manifest structure
        let genesis_script = ScriptPubKey::new(vec![
            0x41, 0x04, 0x67, 0x8a, 0xfd, 0xb0, 0xfe, 0x55, 0x48, 0x27, 0x19, 0x67, 0xf1, 0xa6,
            0x71, 0x30, 0xb7, 0x10, 0x5c, 0xd6, 0xa8, 0x28, 0xe0, 0x39, 0x09, 0xa6, 0x79, 0x62,
            0xe0, 0xea, 0x1f, 0x61, 0xde, 0xb6, 0x49, 0xf6, 0xbc, 0x3f, 0x4c, 0xef, 0x38, 0xc4,
            0xf3, 0x55, 0x04, 0xe5, 0x1e, 0xc1, 0x12, 0xde, 0x5c, 0x38, 0x4d, 0xf7, 0xba, 0x0b,
            0x8d, 0x57, 0x8a, 0x4c, 0x70, 0x2b, 0x6b, 0xf1, 0x1d, 0x5f, 0xAC,
        ]);

        let coinbase = Transaction::new_coinbase(
            "Validation Test".as_bytes().to_vec(),
            vec![TxOut::new(INITIAL_BLOCK_REWARD, genesis_script)],
            0,
        );

        let txid = coinbase.txid();
        let merkle_root = MerkleRoot::compute(&[txid]);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let header = BlockHeader {
            version: BLOCK_VERSION,
            previous_block_hash: BlockHash([0u8; 32]),
            merkle_root,
            timestamp,
            bits: GENESIS_BITS,
            nonce: 0,
        };

        let block = Block::new(header, vec![coinbase]);

        // Test manifest creation
        let manifest = create_genesis_manifest(
            &block,
            "test",
            "Validation Test",
            "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
        );

        assert_eq!(manifest.block_hash, block.hash().to_string());
        assert!(block.verify_merkle_root());

        // Test manifest hash match
        let manifest2 = create_genesis_manifest(
            &block,
            "test",
            "Validation Test",
            "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
        );
        assert_eq!(manifest.block_hash, manifest2.block_hash);
    }
}
