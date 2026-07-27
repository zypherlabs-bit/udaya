use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A 256-bit hash used throughout the system
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockHash(pub [u8; 32]);

impl BlockHash {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes[..32.min(bytes.len())]);
        BlockHash(hash)
    }

    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        // Double SHA-256
        let mut hasher2 = Sha256::new();
        hasher2.update(result);
        let final_result = hasher2.finalize();
        BlockHash(final_result.into())
    }

    pub fn double_sha256(data: &[u8]) -> Self {
        Self::compute(data)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Debug for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockHash({})", hex::encode(self.0))
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<[u8; 32]> for BlockHash {
    fn from(bytes: [u8; 32]) -> Self {
        BlockHash(bytes)
    }
}

/// Transaction ID (double SHA-256 of transaction data)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Txid(pub [u8; 32]);

impl Txid {
    pub fn compute(data: &[u8]) -> Self {
        Txid(BlockHash::compute(data).0)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl fmt::Debug for Txid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Txid({})", hex::encode(self.0))
    }
}

impl fmt::Display for Txid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<[u8; 32]> for Txid {
    fn from(bytes: [u8; 32]) -> Self {
        Txid(bytes)
    }
}

/// Merkle root hash
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MerkleRoot(pub [u8; 32]);

impl MerkleRoot {
    pub fn compute(txids: &[Txid]) -> Self {
        if txids.is_empty() {
            return MerkleRoot([0u8; 32]);
        }

        let mut layer: Vec<[u8; 32]> = txids.iter().map(|t| t.0).collect();

        while layer.len() > 1 {
            if !layer.len().is_multiple_of(2) {
                layer.push(*layer.last().unwrap());
            }

            let mut next_layer = Vec::with_capacity(layer.len() / 2);
            for chunk in layer.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                hasher.update(chunk[1]);
                let first = hasher.finalize();
                let mut hasher2 = Sha256::new();
                hasher2.update(first);
                next_layer.push(hasher2.finalize().into());
            }
            layer = next_layer;
        }

        MerkleRoot(layer[0])
    }
}

impl fmt::Debug for MerkleRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MerkleRoot({})", hex::encode(self.0))
    }
}

/// A script signature (the unlocking script)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSig {
    pub data: Vec<u8>,
}

impl ScriptSig {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// A script public key (the locking script)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPubKey {
    pub data: Vec<u8>,
    pub address: Option<String>,
}

impl ScriptPubKey {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            address: None,
        }
    }

    pub fn with_address(data: Vec<u8>, address: String) -> Self {
        Self {
            data,
            address: Some(address),
        }
    }

    pub fn is_coinbase(&self) -> bool {
        self.data.is_empty()
    }
}

/// A transaction input
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxIn {
    pub previous_output: OutPoint,
    pub script_sig: ScriptSig,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

impl TxIn {
    pub fn new_coinbase(coinbase_data: Vec<u8>) -> Self {
        Self {
            previous_output: OutPoint::null(),
            script_sig: ScriptSig::new(coinbase_data),
            sequence: 0xFFFFFFFF,
            witness: Vec::new(),
        }
    }

    pub fn is_coinbase(&self) -> bool {
        self.previous_output.is_null()
    }
}

/// A transaction output
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOut {
    pub value: u64,
    pub script_pubkey: ScriptPubKey,
}

impl TxOut {
    pub fn new(value: u64, script_pubkey: ScriptPubKey) -> Self {
        Self {
            value,
            script_pubkey,
        }
    }
}

/// An outpoint referencing a specific UTXO
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: Txid, vout: u32) -> Self {
        Self { txid, vout }
    }

    pub fn null() -> Self {
        Self {
            txid: Txid([0u8; 32]),
            vout: 0xFFFFFFFF,
        }
    }

    pub fn is_null(&self) -> bool {
        self.txid == Txid([0u8; 32]) && self.vout == 0xFFFFFFFF
    }
}

impl fmt::Debug for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OutPoint({}:{})", self.txid, self.vout)
    }
}

/// A block header
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: i32,
    pub previous_block_hash: BlockHash,
    pub merkle_root: MerkleRoot,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn new(
        version: i32,
        previous_block_hash: BlockHash,
        merkle_root: MerkleRoot,
        timestamp: u32,
        bits: u32,
        nonce: u32,
    ) -> Self {
        Self {
            version,
            previous_block_hash,
            merkle_root,
            timestamp,
            bits,
            nonce,
        }
    }

    /// Compute the block hash (Proof-of-Work)
    pub fn hash(&self) -> BlockHash {
        let data = self.serialize();
        BlockHash::double_sha256(&data)
    }

    /// Serialize the block header for hashing
    pub fn serialize(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        let mut buf = Vec::with_capacity(80);
        buf.write_i32::<LittleEndian>(self.version).unwrap();
        buf.extend_from_slice(&self.previous_block_hash.0);
        buf.extend_from_slice(&self.merkle_root.0);
        buf.write_u32::<LittleEndian>(self.timestamp).unwrap();
        buf.write_u32::<LittleEndian>(self.bits).unwrap();
        buf.write_u32::<LittleEndian>(self.nonce).unwrap();
        buf
    }

    /// Get the difficulty target from bits
    /// Uses Bitcoin's compact target format:
    ///   target = mantissa * 2^(8 * (exponent - 3))
    /// where exponent = (bits >> 24) and mantissa = (bits & 0x007FFFFF)
    pub fn difficulty_target(&self) -> num_bigint::BigUint {
        let exponent = (self.bits >> 24) as usize;
        let mantissa = self.bits & 0x007FFFFF;

        // The mantissa (3 bytes) is placed at byte position (exponent - 3)
        // in a 256-bit (32-byte) big-endian representation.
        // Position from the left = 32 - exponent
        let mut target_bytes = vec![0u8; 32];
        if exponent <= 32 {
            let pos = 32 - exponent;
            if pos < 32 {
                target_bytes[pos] = ((mantissa >> 16) & 0xFF) as u8;
            }
            if pos + 1 < 32 {
                target_bytes[pos + 1] = ((mantissa >> 8) & 0xFF) as u8;
            }
            if pos + 2 < 32 {
                target_bytes[pos + 2] = (mantissa & 0xFF) as u8;
            }
        }

        num_bigint::BigUint::from_bytes_be(&target_bytes)
    }

    /// Verify proof-of-work
    pub fn verify_pow(&self) -> bool {
        let hash = self.hash();
        let hash_int = num_bigint::BigUint::from_bytes_be(&hash.0);
        hash_int <= self.difficulty_target()
    }
}

impl fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BlockHeader {{ version: {}, prev_hash: {}, merkle: {:?}, time: {}, bits: {:08x}, nonce: {} }}",
            self.version, self.previous_block_hash, self.merkle_root, self.timestamp, self.bits, self.nonce
        )
    }
}

/// A complete block
#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<super::transaction::Transaction>,
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<super::transaction::Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    /// Compute the block hash
    pub fn hash(&self) -> BlockHash {
        self.header.hash()
    }

    /// Compute the merkle root from transactions
    pub fn compute_merkle_root(&self) -> MerkleRoot {
        let txids: Vec<Txid> = self.transactions.iter().map(|tx| tx.txid()).collect();
        MerkleRoot::compute(&txids)
    }

    /// Verify the block's merkle root matches
    pub fn verify_merkle_root(&self) -> bool {
        self.compute_merkle_root() == self.header.merkle_root
    }

    /// Verify all proof-of-work
    pub fn verify_pow(&self) -> bool {
        self.header.verify_pow()
    }

    /// Get coinbase transaction
    pub fn coinbase_tx(&self) -> Option<&super::transaction::Transaction> {
        self.transactions.first().filter(|tx| tx.is_coinbase())
    }

    /// Get non-coinbase transactions
    pub fn txns(&self) -> &[super::transaction::Transaction] {
        &self.transactions[1..]
    }

    /// Serialize block to bytes
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Deserialize block from bytes
    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }

    /// Block size in bytes
    pub fn size(&self) -> usize {
        self.serialize().len()
    }

    /// Number of transactions in the block
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("hash", &self.hash())
            .field("height", &self.header.bits) // placeholder
            .field("tx_count", &self.transactions.len())
            .finish()
    }
}

/// Block locator for sync
#[derive(Clone, Serialize, Deserialize)]
pub struct BlockLocator {
    pub hashes: Vec<BlockHash>,
    pub stop: BlockHash,
}

impl BlockLocator {
    pub fn new(hashes: Vec<BlockHash>, stop: BlockHash) -> Self {
        Self { hashes, stop }
    }
}

/// Inventory type for P2P messages
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum InvType {
    Error = 0,
    Tx = 1,
    Block = 2,
    FilteredBlock = 3,
    CompactBlock = 4,
}

/// Inventory vector
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvVector {
    pub inv_type: InvType,
    pub hash: BlockHash,
}

impl InvVector {
    pub fn new(inv_type: InvType, hash: BlockHash) -> Self {
        Self { inv_type, hash }
    }
}
