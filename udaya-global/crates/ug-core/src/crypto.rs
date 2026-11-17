//! Cryptographic helpers for Udaya Global.
//!
//! Only reviewed libraries are used: `sha2` for SHA-256 and `ed25519-dalek`
//! for Ed25519. No custom primitive is implemented here. Udaya Global uses a
//! deliberately small cryptographic surface:
//!
//! * SHA-256d for hashes (same construction as the existing Udaya chain, but
//!   every consensus hash is domain-separated and bound to the Global chain ID).
//! * Ed25519 for transaction and consensus signatures.
//! * bech32 for human-readable addresses with a 6-character checksum.
//!
//! Signatures are produced over a domain-separated 32-byte digest. Every digest
//! includes the chain ID, so a signature created for the existing Udaya mainnet
//! or for another experimental network cannot be replayed on Udaya Global.

use crate::error::{Error, Result};
use crate::{ChainParams, ADDRESS_VERSION};
use bech32::{Bech32, Hrp};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};

/// A 256-bit hash.
pub type Hash32 = [u8; 32];
/// A 64-byte Ed25519 signature.
pub type SignatureBytes = [u8; 64];

const DIGEST_PREFIX: &[u8] = b"UDAYA-GLOBAL/v1";

/// SHA-256d over length-prefixed parts.
///
/// Length prefixes make the encoding unambiguous: `sha256d(["ab","c"])` can
/// never equal `sha256d(["a","bc"])`.
pub fn sha256d_parts(parts: &[&[u8]]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(DIGEST_PREFIX);
    for part in parts {
        hasher.update((part.len() as u32).to_le_bytes());
        hasher.update(part);
    }
    let first = hasher.finalize();
    let second = Sha256::digest(first);
    let mut out = [0u8; 32];
    out.copy_from_slice(&second);
    out
}

/// Domain-separated tagged hash.
pub fn tagged_hash(tag: &str, parts: &[&[u8]]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(DIGEST_PREFIX);
    hasher.update(b"/tag");
    hasher.update((tag.len() as u16).to_le_bytes());
    hasher.update(tag.as_bytes());
    for part in parts {
        hasher.update((part.len() as u32).to_le_bytes());
        hasher.update(part);
    }
    let first = hasher.finalize();
    let second = Sha256::digest(first);
    let mut out = [0u8; 32];
    out.copy_from_slice(&second);
    out
}

/// Merkle root over ordered leaves.
///
/// Leaves and internal nodes use different domain tags. When a layer has an odd
/// number of nodes the last node is promoted rather than duplicated. Promoting
/// avoids the Bitcoin CVE-2012-2459 class of ambiguity where duplicating the
/// final node can produce an identical root for different transaction lists.
pub fn merkle_root(leaves: &[Hash32]) -> Hash32 {
    if leaves.is_empty() {
        return tagged_hash("ug/merkle/empty", &[]);
    }
    let mut layer: Vec<Hash32> = leaves
        .iter()
        .map(|leaf| tagged_hash("ug/merkle/leaf", &[leaf]))
        .collect();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        let mut i = 0;
        while i + 1 < layer.len() {
            next.push(tagged_hash("ug/merkle/node", &[&layer[i], &layer[i + 1]]));
            i += 2;
        }
        if i < layer.len() {
            next.push(layer[i]);
        }
        layer = next;
    }
    layer[0]
}

/// Encode a public key as a chain-specific bech32 address.
pub fn address_from_pubkey(params: &ChainParams, pubkey: &[u8; 32]) -> Result<String> {
    let hrp = Hrp::parse(&params.address_hrp)
        .map_err(|e| Error::Chain(format!("invalid address HRP: {e}")))?;
    let mut data = Vec::with_capacity(33);
    data.push(ADDRESS_VERSION);
    data.extend_from_slice(pubkey);
    bech32::encode::<Bech32>(hrp, &data)
        .map_err(|e| Error::Chain(format!("address encoding failed: {e}")))
}

/// Decode and validate a chain-specific bech32 address.
pub fn pubkey_from_address(params: &ChainParams, address: &str) -> Result<[u8; 32]> {
    // Bound the input before the decoder allocates anything.
    if address.len() > 128 {
        return Err(Error::Chain("address is too long".into()));
    }
    let (hrp, data) =
        bech32::decode(address).map_err(|e| Error::Chain(format!("invalid address: {e}")))?;
    if hrp.to_string() != params.address_hrp {
        return Err(Error::Chain(format!(
            "address belongs to a different network (HRP {})",
            hrp
        )));
    }
    if data.len() != 33 || data[0] != ADDRESS_VERSION {
        return Err(Error::Chain("invalid address payload".into()));
    }
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&data[1..]);
    Ok(pubkey)
}

/// Generate a new Ed25519 signing key from the operating system CSPRNG.
pub fn generate_key() -> SigningKey {
    SigningKey::generate(&mut rand::rngs::OsRng)
}

/// Public key bytes for a signing key.
pub fn public_key(key: &SigningKey) -> [u8; 32] {
    key.verifying_key().to_bytes()
}

/// Sign a domain-separated digest.
pub fn sign(key: &SigningKey, tag: &str, parts: &[&[u8]]) -> SignatureBytes {
    key.sign(&tagged_hash(tag, parts)).to_bytes()
}

/// Verify a domain-separated signature. Returns false on any malformed input.
pub fn verify(pubkey: &[u8; 32], signature: &SignatureBytes, tag: &str, parts: &[&[u8]]) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(pubkey) else {
        return false;
    };
    let signature = Signature::from_bytes(signature);
    verifying_key
        .verify_strict(&tagged_hash(tag, parts), &signature)
        .is_ok()
}

/// True when `b` is a lexicographically smaller hash than `a`.
pub fn hash_lt(a: &Hash32, b: &Hash32) -> bool {
    a < b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_hashes_are_domain_separated_and_unambiguous() {
        let a = tagged_hash("one", &[b"ab", b"c"]);
        let b = tagged_hash("one", &[b"a", b"bc"]);
        let c = tagged_hash("two", &[b"ab", b"c"]);
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn merkle_root_is_deterministic_and_order_sensitive() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        assert_eq!(merkle_root(&[a, b]), merkle_root(&[a, b]));
        assert_ne!(merkle_root(&[a, b]), merkle_root(&[b, a]));
        assert_ne!(merkle_root(&[]), merkle_root(&[a]));
    }

    #[test]
    fn merkle_root_odd_promotion_differs_from_duplication() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let c = [3u8; 32];
        let promoted = merkle_root(&[a, b, c]);
        // Manual duplication of the last leaf yields a different root because
        // internal nodes and leaves have different domain tags.
        let manual = merkle_root(&[a, b, c, c]);
        assert_ne!(promoted, manual);
    }

    #[test]
    fn ed25519_sign_verify_and_tamper_rejection() {
        let key = SigningKey::from_bytes(&[7u8; 32]);
        let pubkey = public_key(&key);
        let sig = sign(&key, "test", &[b"hello", b"world"]);
        assert!(verify(&pubkey, &sig, "test", &[b"hello", b"world"]));
        assert!(!verify(&pubkey, &sig, "test", &[b"hello", b"worle"]));
        assert!(!verify(&pubkey, &sig, "other", &[b"hello", b"world"]));
        let mut bad = sig;
        bad[0] ^= 1;
        assert!(!verify(&pubkey, &bad, "test", &[b"hello", b"world"]));
    }

    #[test]
    fn chain_specific_addresses_roundtrip_and_reject_other_networks() {
        let params = ChainParams::devnet();
        let pubkey = [9u8; 32];
        let addr = address_from_pubkey(&params, &pubkey).unwrap();
        assert_eq!(pubkey_from_address(&params, &addr).unwrap(), pubkey);

        let mut other = ChainParams::devnet();
        other.address_hrp = "other".into();
        assert!(pubkey_from_address(&other, &addr).is_err());
    }

    #[test]
    fn malformed_addresses_are_rejected_without_panicking() {
        let params = ChainParams::devnet();
        for bad in ["", "x", "ugdev1", "not-bech32", &"a".repeat(500)] {
            assert!(pubkey_from_address(&params, bad).is_err());
        }
    }
}
