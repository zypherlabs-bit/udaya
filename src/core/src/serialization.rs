use crate::transaction::Transaction;
use crate::types::{BlockHash, BlockHeader, MerkleRoot, OutPoint};
use std::io::Read;

/// Trait for objects that can be serialized to/from bytes
pub trait Serializable: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> anyhow::Result<Self>;
}

/// Serialize a u32 in little-endian format
pub fn serialize_u32_le(val: u32) -> [u8; 4] {
    val.to_le_bytes()
}

/// Serialize a u64 in little-endian format
pub fn serialize_u64_le(val: u64) -> [u8; 8] {
    val.to_le_bytes()
}

/// Serialize a u16 in little-endian format
pub fn serialize_u16_le(val: u16) -> [u8; 2] {
    val.to_le_bytes()
}

/// Deserialize a u32 from little-endian bytes
pub fn deserialize_u32_le(bytes: &[u8]) -> anyhow::Result<u32> {
    if bytes.len() < 4 {
        anyhow::bail!("Not enough bytes for u32");
    }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&bytes[..4]);
    Ok(u32::from_le_bytes(arr))
}

/// Deserialize a u64 from little-endian bytes
pub fn deserialize_u64_le(bytes: &[u8]) -> anyhow::Result<u64> {
    if bytes.len() < 8 {
        anyhow::bail!("Not enough bytes for u64");
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes[..8]);
    Ok(u64::from_le_bytes(arr))
}

/// Deserialize a variable-length integer
pub fn deserialize_varint(data: &[u8]) -> anyhow::Result<(u64, usize)> {
    if data.is_empty() {
        anyhow::bail!("Empty data for varint");
    }

    match data[0] {
        0xFF => {
            if data.len() < 9 {
                anyhow::bail!("Not enough bytes for 16-byte varint");
            }
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&data[1..9]);
            Ok((u64::from_le_bytes(arr), 9))
        }
        0xFE => {
            if data.len() < 5 {
                anyhow::bail!("Not enough bytes for 4-byte varint");
            }
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&data[1..5]);
            Ok((u32::from_le_bytes(arr) as u64, 5))
        }
        0xFD => {
            if data.len() < 3 {
                anyhow::bail!("Not enough bytes for 2-byte varint");
            }
            let mut arr = [0u8; 2];
            arr.copy_from_slice(&data[1..3]);
            Ok((u16::from_le_bytes(arr) as u64, 3))
        }
        _ => Ok((data[0] as u64, 1)),
    }
}

/// Generate a checksum for data integrity verification
pub fn calculate_checksum(data: &[u8]) -> [u8; 4] {
    use sha2::{Digest, Sha256};
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(hash1);
    let bytes = hash2.to_vec();
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

/// Verify data integrity using checksum
pub fn verify_checksum(data: &[u8], expected_checksum: &[u8; 4]) -> bool {
    calculate_checksum(data) == *expected_checksum
}

/// Encode a value using variable-length encoding (Bitcoin-style)
pub fn encode_varint(val: u64) -> Vec<u8> {
    if val < 0xFD {
        vec![val as u8]
    } else if val <= 0xFFFF {
        let mut buf = vec![0xFD];
        buf.extend_from_slice(&(val as u16).to_le_bytes());
        buf
    } else if val <= 0xFFFF_FFFF {
        let mut buf = vec![0xFE];
        buf.extend_from_slice(&(val as u32).to_le_bytes());
        buf
    } else {
        let mut buf = vec![0xFF];
        buf.extend_from_slice(&val.to_le_bytes());
        buf
    }
}

/// Serialize a transaction list with varint count prefix
pub fn serialize_transaction_list(txs: &[Transaction]) -> Vec<u8> {
    let mut data = encode_varint(txs.len() as u64);
    for tx in txs {
        data.extend_from_slice(&tx.serialize());
    }
    data
}

impl BlockHeader {
    /// Deserialize a block header from bytes
    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 80 {
            anyhow::bail!("Block header too short: {} bytes", data.len());
        }

        use byteorder::{LittleEndian, ReadBytesExt};
        let mut cursor = std::io::Cursor::new(data);

        let version = cursor.read_i32::<LittleEndian>()?;

        let mut prev_hash = [0u8; 32];
        cursor.read_exact(&mut prev_hash)?;

        let mut merkle = [0u8; 32];
        cursor.read_exact(&mut merkle)?;

        let timestamp = cursor.read_u32::<LittleEndian>()?;
        let bits = cursor.read_u32::<LittleEndian>()?;
        let nonce = cursor.read_u32::<LittleEndian>()?;

        Ok(BlockHeader {
            version,
            previous_block_hash: BlockHash(prev_hash),
            merkle_root: MerkleRoot(merkle),
            timestamp,
            bits,
            nonce,
        })
    }
}

impl Default for BlockHeader {
    fn default() -> Self {
        Self {
            version: crate::BLOCK_VERSION,
            previous_block_hash: BlockHash([0u8; 32]),
            merkle_root: MerkleRoot([0u8; 32]),
            timestamp: 0,
            bits: crate::consensus::GENESIS_BITS,
            nonce: 0,
        }
    }
}

impl std::fmt::Display for OutPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.txid, self.vout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encoding() {
        // Single byte
        assert_eq!(encode_varint(0x00), vec![0x00]);
        assert_eq!(encode_varint(0xFC), vec![0xFC]);

        // 2 bytes
        let encoded = encode_varint(0xFD);
        assert_eq!(encoded[0], 0xFD);
        assert_eq!(encoded.len(), 3);

        // 4 bytes
        let encoded = encode_varint(0x10000);
        assert_eq!(encoded[0], 0xFE);
        assert_eq!(encoded.len(), 5);

        // 8 bytes
        let encoded = encode_varint(0x100000000);
        assert_eq!(encoded[0], 0xFF);
        assert_eq!(encoded.len(), 9);
    }

    #[test]
    fn test_varint_deserialization() {
        let data = encode_varint(42);
        let (val, _) = deserialize_varint(&data).unwrap();
        assert_eq!(val, 42);

        let data = encode_varint(0xFFFF);
        let (val, _) = deserialize_varint(&data).unwrap();
        assert_eq!(val, 0xFFFF);
    }

    #[test]
    fn test_checksum() {
        let data = b"Udaya test data";
        let checksum = calculate_checksum(data);
        assert!(verify_checksum(data, &checksum));
        assert!(!verify_checksum(data, &[0u8; 4]));
    }

    #[test]
    fn test_block_header_serialization_roundtrip() {
        let header = BlockHeader {
            version: 1,
            previous_block_hash: BlockHash([0xAB; 32]),
            merkle_root: MerkleRoot([0xCD; 32]),
            timestamp: 1234567890,
            bits: 0x1D00FFFF,
            nonce: 42,
        };

        let serialized = header.serialize();
        assert_eq!(serialized.len(), 80);

        let deserialized = BlockHeader::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.version, header.version);
        assert_eq!(deserialized.previous_block_hash, header.previous_block_hash);
        assert_eq!(deserialized.merkle_root, header.merkle_root);
        assert_eq!(deserialized.timestamp, header.timestamp);
        assert_eq!(deserialized.bits, header.bits);
        assert_eq!(deserialized.nonce, header.nonce);

        // Hash must be deterministic
        assert_eq!(deserialized.hash(), header.hash());
    }
}
