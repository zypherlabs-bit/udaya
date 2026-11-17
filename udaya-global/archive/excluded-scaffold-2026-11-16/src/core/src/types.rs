use hex;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

// A 256-bit hash used throughout the system
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockHash(pub [u8; 32]);

impl BlockHash {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        let n = bytes.len().min(32);
        hash[..n].copy_from_slice(&bytes[..n]);
        BlockHash(hash)
    }
    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hasher2 = Sha256::new();
        hasher2.update(result);
        let final_result = hasher2.finalize();
        BlockHash(final_result.into())
    }
    pub fn double_sha256(data: &[u8]) -> Self { Self::compute(data) }
    pub fn as_bytes(&self) -> &[u8; 32] { &self.0 }
    pub fn to_vec(&self) -> Vec<u8> { self.0.to_vec() }
    pub fn is_zero(&self) -> bool { self.0.iter().all(|&b| b == 0) }
}

impl fmt::Debug for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockHash({})", hex::encode(self.0))
    }
}
impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", hex::encode(self.0)) }
}
impl From<[u8; 32]> for BlockHash { fn from(arr: [u8; 32]) -> Self { BlockHash(arr) } }
impl From<BlockHash> for [u8; 32] { fn from(h: BlockHash) -> [u8; 32] { h.0 } }

pub fn target_to_hex(target: &BigUint) -> String { format!("{:064x}", target) }

pub fn compact_to_target(bits: u32) -> Option<BigUint> {
    let size = bits >> 24;
    let word = bits & 0x007f_ffff;
    if bits & 0x0080_0000 != 0 { return None; }
    if size > 34 || (word > 0xff && size > 33) || (word > 0xffff && size > 32) { return None; }
    Some(if size <= 3 { BigUint::from(word >> (8 * (3 - size))) } else { BigUint::from(word) << (8 * (size - 3)) })
}

pub fn target_to_compact(target: &BigUint) -> u32 {
    if target.is_zero() { return 0; }
    let mut size = ((target.bits() + 7) / 8) as u32;
    let mut compact = if size <= 3 { (target << (8 * (3 - size))).to_u32().unwrap_or(0) } else { (target >> (8 * (size - 3))).to_u32().unwrap_or(0) };
    if compact & 0x0080_0000 != 0 { compact >>= 8; size += 1; }
    (size << 24) | (compact & 0x007f_ffff)
}

pub type Txid = BlockHash;

pub mod script {
    use serde::{Deserialize, Serialize};
    use std::fmt;
    #[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
    pub struct Script(pub Vec<u8>);
    impl Script {
        pub fn new(data: Vec<u8>) -> Self { Script(data) }
        pub fn data(&self) -> &[u8] { &self.0 }
        pub fn len(&self) -> usize { self.0.len() }
        pub fn is_empty(&self) -> bool { self.0.is_empty() }
    }
    impl fmt::Debug for Script {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Script({})", hex::encode(&self.0)) }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct OutPoint { pub txid: Txid, pub vout: u32 }
impl OutPoint {
    pub fn new(txid: Txid, vout: u32) -> Self { OutPoint { txid, vout } }
    pub fn is_coinbase(&self) -> bool { self.txid.is_zero() && self.vout == 0xffffffff }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TxIn { pub previous_output: OutPoint, pub script_sig: script::Script, pub sequence: u32 }
impl TxIn {
    pub fn new_coinbase(script_data: Vec<u8>) -> Self {
        TxIn { previous_output: OutPoint::default(), script_sig: script::Script::new(script_data), sequence: 0xffffffff }
    }
    pub fn is_coinbase(&self) -> bool { self.previous_output.txid.is_zero() && self.previous_output.vout == 0xffffffff }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TxOut { pub value: u64, pub script_pubkey: script::Script }
impl TxOut { pub fn new(value: u64, script_pubkey: script::Script) -> Self { TxOut { value, script_pubkey } } }

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default, Debug)]
pub struct BlockHeader {
    pub version: i32,
    pub prev_block: BlockHash,
    pub merkle_root: BlockHash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
}

pub type MerkleRoot = BlockHash;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default, Debug)]
pub struct Block { pub header: BlockHeader, pub transactions: Vec<Vec<u8>> }
