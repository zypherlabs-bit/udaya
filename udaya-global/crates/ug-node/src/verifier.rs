//! Full block verification used by the consensus engine.
//!
//! The verifier executes the block against the parent UTXO set and checks every
//! consensus commitment: parent link, height, timestamp drift, transaction
//! signatures, value conservation, fee burn, transaction root and state root.

use ug_consensus::ProposalVerifier;
use ug_core::block::{Block, BlockHeader};
use ug_core::state::UtxoSet;
use ug_core::ChainParams;

/// Verifier bound to one parent header and parent state.
pub struct NodeVerifier<'a> {
    pub chain: &'a ChainParams,
    pub parent: &'a BlockHeader,
    pub parent_state: &'a UtxoSet,
    pub now_unix: u64,
}

impl ProposalVerifier for NodeVerifier<'_> {
    fn verify(&self, chain: &ChainParams, block: &Block) -> std::result::Result<(), String> {
        if chain != self.chain {
            return Err("verifier chain mismatch".into());
        }
        block
            .execute(chain, self.parent, self.parent_state, self.now_unix)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use ug_core::block::BLOCK_VERSION;
    use ug_core::crypto::{self, public_key};
    use ug_core::genesis::{create_config, Allocation, Genesis};

    fn setup() -> (Genesis, ChainParams) {
        let keys: Vec<SigningKey> = (1..=4)
            .map(|seed| SigningKey::from_bytes(&[seed; 32]))
            .collect();
        let genesis = Genesis::build(create_config(
            1_700_000_000,
            vec![Allocation {
                recipient: public_key(&keys[0]),
                value: 10_000_000,
            }],
            keys.iter().map(public_key).collect(),
        ))
        .unwrap();
        let chain = genesis.chain_params();
        (genesis, chain)
    }

    #[test]
    fn verifier_accepts_an_empty_child_and_rejects_a_wrong_root() {
        let (genesis, chain) = setup();
        let parent = genesis.block().clone();
        let validator = public_key(&SigningKey::from_bytes(&[1u8; 32]));
        let mut block = Block {
            header: BlockHeader {
                version: BLOCK_VERSION,
                height: 1,
                previous_hash: genesis.hash(),
                timestamp: 1_700_000_001,
                tx_root: crypto::merkle_root(&[]),
                state_root: genesis.block().header.state_root,
                proposer: validator,
                round: 0,
            },
            transactions: vec![],
        };
        let verifier = NodeVerifier {
            chain: &chain,
            parent: &parent.header,
            parent_state: genesis.state(),
            now_unix: 1_700_000_010,
        };
        verifier.verify(&chain, &block).unwrap();

        block.header.state_root[0] ^= 1;
        assert!(verifier.verify(&chain, &block).is_err());

        block.header.previous_hash[0] ^= 1;
        assert!(verifier.verify(&chain, &block).is_err());
    }
}
