use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Udaya address types
#[derive(Debug, Clone, PartialEq)]
pub enum AddressType {
    P2PKH,  // Pay-to-PubKey-Hash
    P2SH,   // Pay-to-Script-Hash
    Bech32, // SegWit native
}

/// A Udaya address
#[derive(Debug, Clone)]
pub struct Address {
    pub addr_type: AddressType,
    pub hash: [u8; 20],
    pub network: Network,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Network {
    Mainnet,
    Testnet,
    Regtest,
}

impl Address {
    /// Create a P2PKH address from a public key hash
    pub fn p2pkh(hash: [u8; 20], network: Network) -> Self {
        Self {
            addr_type: AddressType::P2PKH,
            hash,
            network,
        }
    }

    /// Create a P2SH address from a script hash
    pub fn p2sh(hash: [u8; 20], network: Network) -> Self {
        Self {
            addr_type: AddressType::P2SH,
            hash,
            network,
        }
    }

    /// Create address from public key
    pub fn from_public_key(pubkey: &[u8], network: Network) -> Self {
        let hash = hash160(pubkey);
        Self::p2pkh(hash, network)
    }

    /// Encode address to Base58 format
    pub fn to_base58(&self) -> String {
        let version_byte = match (self.addr_type.clone(), self.network.clone()) {
            (AddressType::P2PKH, Network::Mainnet) => 0x00u8,
            (AddressType::P2SH, Network::Mainnet) => 0x05u8,
            (AddressType::P2PKH, Network::Testnet) => 0x6Fu8,
            (AddressType::P2SH, Network::Testnet) => 0xC4u8,
            _ => 0x00u8,
        };

        let mut data = vec![version_byte];
        data.extend_from_slice(&self.hash);

        // Checksum (double SHA-256, first 4 bytes)
        let checksum = double_sha256_first_4(&data);
        data.extend_from_slice(&checksum);

        bs58_encode(&data)
    }

    /// Parse address from Base58 string
    pub fn from_base58(s: &str) -> anyhow::Result<Self> {
        let data = bs58_decode(s)?;
        if data.len() < 5 {
            anyhow::bail!("Invalid address length");
        }

        let version = data[0];
        let hash: [u8; 20] = data[1..21].try_into()?;

        // Verify checksum with constant-time comparison
        let checksum = double_sha256_first_4(&data[..data.len() - 4]);
        if bool::from(!data[data.len() - 4..].ct_eq(&checksum)) {
            anyhow::bail!("Invalid address checksum");
        }

        let (addr_type, network) = match version {
            0x00 => (AddressType::P2PKH, Network::Mainnet),
            0x05 => (AddressType::P2SH, Network::Mainnet),
            0x6F => (AddressType::P2PKH, Network::Testnet),
            0xC4 => (AddressType::P2SH, Network::Testnet),
            _ => anyhow::bail!("Unknown address version: {}", version),
        };

        Ok(Self {
            addr_type,
            hash,
            network,
        })
    }
}

/// RIPEMD-160(SHA-256(data))
pub fn hash160(data: &[u8]) -> [u8; 20] {
    let sha256 = Sha256::digest(data);
    let ripemd = Ripemd160::digest(sha256);
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&ripemd);
    hash
}

/// Double SHA-256, return first 4 bytes
fn double_sha256_first_4(data: &[u8]) -> [u8; 4] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    let mut result = [0u8; 4];
    result.copy_from_slice(&second[..4]);
    result
}

/// Simple Base58 encoding
fn bs58_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    if data.is_empty() {
        return String::new();
    }

    let mut zero_count = 0;
    for &b in data {
        if b == 0 {
            zero_count += 1;
        } else {
            break;
        }
    }

    let mut result = Vec::new();
    let mut num = num_bigint::BigUint::from_bytes_be(data);
    let base = num_bigint::BigUint::from(58u32);

    while num > BigUint::from(0u32) {
        let remainder = &num % &base;
        num /= &base;
        result.push(ALPHABET[remainder.to_usize().unwrap_or(0)]);
    }

    for _ in 0..zero_count {
        result.push(ALPHABET[0]);
    }

    result.reverse();
    String::from_utf8(result).unwrap_or_default()
}

/// Simple Base58 decoding
fn bs58_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    // Count leading '1's which decode to leading zero bytes
    let mut leading_zeros = 0;
    for c in s.chars() {
        if c == '1' {
            leading_zeros += 1;
        } else {
            break;
        }
    }

    let mut result = BigUint::from(0u32);
    let base = BigUint::from(58u32);

    for c in s.chars().skip(leading_zeros) {
        if let Some(idx) = ALPHABET.iter().position(|&a| a == c as u8) {
            result = result * &base + BigUint::from(idx as u32);
        } else {
            anyhow::bail!("Invalid Base58 character: {}", c);
        }
    }

    let mut bytes = result.to_bytes_be();
    // Prepend leading zeros for each leading '1'
    let mut output = vec![0u8; leading_zeros];
    output.append(&mut bytes);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let pubkey = vec![
            0x02, 0xce, 0x6b, 0x0b, 0x50, 0x6f, 0x9d, 0x36, 0x5f, 0x5e, 0x0c, 0x5f, 0x6f, 0x9d,
            0x36, 0x5f, 0x5e, 0x0c, 0x5f, 0x6f, 0x9d, 0x36, 0x5f, 0x5e, 0x0c, 0x5f, 0x6f, 0x9d,
            0x36, 0x5f, 0x5e, 0x0c, 0x5f,
        ];

        let addr = Address::from_public_key(&pubkey, Network::Mainnet);
        let encoded = addr.to_base58();
        assert!(!encoded.is_empty());

        let decoded = Address::from_base58(&encoded).unwrap();
        assert_eq!(decoded.hash, addr.hash);
    }

    #[test]
    fn test_hash160() {
        let data = b"hello world";
        let hash = hash160(data);
        assert_eq!(hash.len(), 20);
    }
}
