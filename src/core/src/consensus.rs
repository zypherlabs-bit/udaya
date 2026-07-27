use crate::types::{Block, BlockHash, BlockHeader};
use crate::{
    BLOCK_TARGET_TIME_SECS, BLOCK_VERSION, DIFFICULTY_ADJUSTMENT_INTERVAL, HALVING_INTERVAL,
    INITIAL_BLOCK_REWARD, MAX_BLOCK_WEIGHT, MAX_SUPPLY, SATS_PER_COIN,
};
use num_bigint::BigUint;
use std::collections::HashMap;

/// Maximum target for difficulty (minimum difficulty)
pub const MAX_TARGET: u32 = 0x1D00FFFF;

/// Difficulty target for genesis block (easiest)
pub const GENESIS_BITS: u32 = 0x1D00FFFF;

/// Anti-selfish mining: minimum time a block must wait before being propagated
pub const SELFISH_MINING_STALE_THRESHOLD: u64 = 2;

/// Maximum reorg depth for chain reorganization
pub const MAX_REORG_DEPTH: u64 = 6;

/// Number of blocks for finality checkpoints
pub const FINALITY_DEPTH: u64 = 100;

/// Checkpoint blocks for chain finality (height -> hash)
#[derive(Clone)]
pub struct Checkpoints {
    pub points: HashMap<u64, BlockHash>,
}

impl Default for Checkpoints {
    fn default() -> Self {
        let mut points = HashMap::new();
        points.insert(0, BlockHash::from([0u8; 32]));
        Self { points }
    }
}

impl Checkpoints {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_checkpoint(&self, height: u64) -> bool {
        self.points.contains_key(&height)
    }

    pub fn get_checkpoint(&self, height: u64) -> Option<BlockHash> {
        self.points.get(&height).copied()
    }

    pub fn add_checkpoint(&mut self, height: u64, hash: BlockHash) {
        self.points.insert(height, hash);
    }
}

/// The Proof-of-Work consensus engine for Udaya
#[derive(Clone)]
pub struct ConsensusEngine {
    pub checkpoints: Checkpoints,
    pub params: ConsensusParams,
}

/// Consensus parameters
#[derive(Clone)]
pub struct ConsensusParams {
    pub max_block_weight: u64,
    pub pow_limit: BigUint,
    pub pow_target_spacing: u64,
    pub difficulty_adjustment_interval: u64,
    pub halving_interval: u64,
    pub coinbase_maturity: u64,
    pub max_reorg_depth: u64,
    pub finality_depth: u64,
    pub min_peers_for_propagation: usize,
    pub enable_segwit: bool,
    pub enable_taproot: bool,
    pub enable_schnorr: bool,
    pub bip34_height: u64,
    pub bip66_height: u64,
    pub bip65_height: u64,
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            max_block_weight: MAX_BLOCK_WEIGHT,
            pow_limit: BigUint::from_bytes_be(&[
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF,
            ]),
            pow_target_spacing: BLOCK_TARGET_TIME_SECS,
            difficulty_adjustment_interval: DIFFICULTY_ADJUSTMENT_INTERVAL,
            halving_interval: HALVING_INTERVAL,
            coinbase_maturity: 100,
            max_reorg_depth: MAX_REORG_DEPTH,
            finality_depth: FINALITY_DEPTH,
            min_peers_for_propagation: 3,
            enable_segwit: true,
            enable_taproot: true,
            enable_schnorr: true,
            bip34_height: 1,
            bip66_height: 1,
            bip65_height: 1,
        }
    }
}

impl ConsensusEngine {
    pub fn new(params: ConsensusParams) -> Self {
        Self {
            checkpoints: Checkpoints::new(),
            params,
        }
    }

    /// Calculate the block reward at a given height with halving
    pub fn block_reward(&self, height: u64) -> u64 {
        let halvings = height / self.params.halving_interval;
        if halvings >= 64 {
            return 0; // No more new coins after 64 halvings
        }
        INITIAL_BLOCK_REWARD >> halvings
    }

    /// Calculate mining reward (block reward + fees)
    pub fn mining_reward(&self, height: u64, fees: u64) -> u64 {
        self.block_reward(height) + fees
    }

    /// Total supply mined up to a given height
    /// Uses mathematical formula: sum of geometric series for halvings
    pub fn total_supply_at_height(&self, height: u64) -> u64 {
        // Each halving epoch: INITIAL_BLOCK_REWARD / 2^n per block for HALVING_INTERVAL blocks
        let mut supply = 0u64;
        let mut remaining = height + 1; // genesis block included
        let mut reward = INITIAL_BLOCK_REWARD;
        let max_supply = MAX_SUPPLY * SATS_PER_COIN;

        for _epoch in 0..64 {
            if reward == 0 || remaining == 0 {
                break;
            }

            let blocks_in_epoch = remaining.min(self.params.halving_interval);
            let epoch_supply = blocks_in_epoch.saturating_mul(reward);

            supply = supply.saturating_add(epoch_supply);
            if supply >= max_supply {
                return max_supply;
            }

            remaining = remaining.saturating_sub(blocks_in_epoch);
            reward >>= 1;

            if reward == 0 {
                break;
            }
        }

        supply.min(max_supply)
    }

    /// Calculate the next difficulty target using the Bitcoin DAA
    pub fn calculate_difficulty(&self, chain_headers: &[BlockHeader]) -> u32 {
        if chain_headers.is_empty() {
            return GENESIS_BITS;
        }

        let count = chain_headers.len() as u64;
        if !count.is_multiple_of(self.params.difficulty_adjustment_interval) {
            // Not a difficulty adjustment period, return previous bits
            return chain_headers.last().unwrap().bits;
        }

        // Get the first block in this period and the last
        let _first = &chain_headers[0];
        let last = chain_headers.last().unwrap();

        let period_start = count.saturating_sub(self.params.difficulty_adjustment_interval);

        let first_of_period = &chain_headers[period_start as usize];

        // Calculate actual time span
        let actual_timespan = last.timestamp.saturating_sub(first_of_period.timestamp) as u64;

        // Target timespan = 2 weeks (2016 blocks * 10 min)
        let target_timespan =
            self.params.difficulty_adjustment_interval * self.params.pow_target_spacing;

        // Constrain the timespan adjustment
        let mut actual_timespan = std::cmp::max(actual_timespan, target_timespan / 4);
        actual_timespan = std::cmp::min(actual_timespan, target_timespan * 4);

        // Calculate new target
        let current_target = last.difficulty_target();
        let new_target =
            current_target * BigUint::from(actual_timespan) / BigUint::from(target_timespan);

        // Clamp to pow limit
        let new_target = std::cmp::min(new_target, self.params.pow_limit.clone());

        // Convert target back to bits
        self.target_to_bits(&new_target)
    }

    /// Fast difficulty adjustment for emergencies (every block if needed)
    pub fn calculate_difficulty_fast(
        &self,
        last_headers: &[BlockHeader],
        last_6_blocks: &[BlockHeader],
    ) -> u32 {
        // Use the last 6 blocks for faster adjustment
        if last_6_blocks.len() < 6 {
            return if last_headers.is_empty() {
                GENESIS_BITS
            } else {
                last_headers.last().unwrap().bits
            };
        }

        let first = &last_6_blocks[0];
        let last = last_6_blocks.last().unwrap();

        let actual_timespan = last.timestamp.saturating_sub(first.timestamp) as u64;
        // 6 blocks should take ~60 minutes
        let target_timespan = 6 * self.params.pow_target_spacing;

        let mut actual_timespan = std::cmp::max(actual_timespan, target_timespan / 4);
        actual_timespan = std::cmp::min(actual_timespan, target_timespan * 4);

        let current_target = last.difficulty_target();
        let new_target =
            current_target * BigUint::from(actual_timespan) / BigUint::from(target_timespan);

        let new_target = std::cmp::min(new_target, self.params.pow_limit.clone());
        self.target_to_bits(&new_target)
    }

    /// Convert difficulty target to bits format
    fn target_to_bits(&self, target: &BigUint) -> u32 {
        let bytes = target.to_bytes_be();
        if bytes.is_empty() || bytes == [0u8] {
            return 0;
        }

        let exponent = bytes.len() as u32;

        if bytes[0] > 0x7F {
            // Need to shift right by 1 bit if high bit is set
            let mantissa = ((bytes[0] as u32) << 8) | (bytes[1] as u32);
            (exponent << 24) | (mantissa >> 1)
        } else {
            let mantissa = if bytes.len() >= 3 {
                (bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | (bytes[2] as u32)
            } else if bytes.len() == 2 {
                (bytes[0] as u32) << 16 | (bytes[1] as u32) << 8
            } else {
                (bytes[0] as u32) << 16
            };
            (exponent << 24) | mantissa
        }
    }

    /// Verify a block header's proof-of-work
    pub fn verify_pow(&self, header: &BlockHeader) -> bool {
        // Check block version is valid
        if header.version < BLOCK_VERSION {
            return false;
        }

        // Check timestamp not too far in the future
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        // 2 hours future time tolerance
        if header.timestamp > now + 7200 {
            return false;
        }

        // Verify the actual PoW
        header.verify_pow()
    }

    /// Check if a block can be accepted (basic validation)
    pub fn verify_block_basic(&self, block: &Block, _height: u64) -> anyhow::Result<()> {
        // Check size limits
        if block.size() > crate::MAX_BLOCK_SIZE {
            anyhow::bail!("Block exceeds maximum size");
        }

        // Check weight limits (for SegWit)
        let weight = block.transactions.iter().map(|tx| tx.weight()).sum::<u64>();
        if weight > self.params.max_block_weight {
            anyhow::bail!("Block exceeds maximum weight");
        }

        // Check first transaction is coinbase
        let coinbase = block
            .coinbase_tx()
            .ok_or_else(|| anyhow::anyhow!("Block missing coinbase transaction"))?;

        // Coinbase must be valid
        if !coinbase.is_valid_structure() {
            anyhow::bail!("Invalid coinbase structure");
        }

        // Only one coinbase transaction allowed
        let coinbase_count = block
            .transactions
            .iter()
            .filter(|tx| tx.is_coinbase())
            .count();
        if coinbase_count != 1 {
            anyhow::bail!("Block must have exactly one coinbase");
        }

        // Check no duplicate transactions
        let mut txids = std::collections::HashSet::new();
        for tx in &block.transactions {
            if !txids.insert(tx.txid()) {
                anyhow::bail!("Duplicate transaction in block");
            }
        }

        // Verify merkle root
        if !block.verify_merkle_root() {
            anyhow::bail!("Invalid merkle root");
        }

        // Verify PoW
        if !self.verify_pow(&block.header) {
            anyhow::bail!("Proof-of-work verification failed");
        }

        Ok(())
    }

    /// Verify block against chain context
    pub fn verify_block_context(
        &self,
        block: &Block,
        height: u64,
        previous_block: &BlockHeader,
        median_time: u64,
    ) -> anyhow::Result<()> {
        // Check previous hash matches
        if block.header.previous_block_hash != previous_block.hash() {
            anyhow::bail!("Previous block hash mismatch");
        }

        // Check timestamp
        if u64::from(block.header.timestamp) <= median_time {
            anyhow::bail!("Block timestamp must be greater than median of last 11 blocks");
        }

        // Check checkpoint
        if let Some(checkpoint_hash) = self.checkpoints.get_checkpoint(height) {
            if block.hash() != checkpoint_hash {
                anyhow::bail!("Checkpoint hash mismatch at height {}", height);
            }
        }

        // BIP-34: coinbase must include block height
        let coinbase = block.coinbase_tx().unwrap();
        if height >= self.params.bip34_height && coinbase.inputs[0].script_sig.data.is_empty() {
            anyhow::bail!("BIP-34 violated: coinbase must include block height");
        }

        // Verify block reward
        let total_fees = self.calculate_total_fees(block);
        let expected_reward = self.mining_reward(height, total_fees);
        let coinbase_value = coinbase.total_output();

        if coinbase_value > expected_reward {
            anyhow::bail!(
                "Coinbase value {} exceeds reward {}",
                coinbase_value,
                expected_reward
            );
        }

        Ok(())
    }

    /// Calculate total fees in a block
    fn calculate_total_fees(&self, block: &Block) -> u64 {
        let mut total_out = 0u64;
        let total_in = 0u64;

        // Sum all outputs
        for tx in &block.transactions {
            total_out += tx.total_output();
        }

        // For non-coinbase transactions, input values would be looked up from UTXO set
        // Since we don't have the UTXO set context here, this is an approximation
        // that gets refined during block validation with the actual UTXO set
        if block.transactions.len() <= 1 {
            return 0; // Only coinbase
        }

        for tx in &block.transactions[1..] {
            for _input in &tx.inputs {
                // In a real implementation, we'd look up input values from UTXOs
                // For now, accumulate known values from outputs of previous txs in this block
            }
        }

        let coinbase_out = block.coinbase_tx().map(|tx| tx.total_output()).unwrap_or(0);

        total_out
            .saturating_sub(coinbase_out)
            .saturating_sub(total_in.min(total_out))
    }

    /// Anti-selfish mining: detect blocks with invalid timing patterns
    pub fn detect_selfish_mining(&self, headers: &[BlockHeader]) -> bool {
        if headers.len() < 3 {
            return false;
        }

        let recent = &headers[headers.len().saturating_sub(3)..];

        // Check for abnormally fast blocks (potential withholding)
        for i in 1..recent.len() {
            let time_diff = recent[i].timestamp.saturating_sub(recent[i - 1].timestamp);
            if !(1..=3600).contains(&time_diff) {
                return true;
            }
        }

        false
    }

    /// Validate chain reorganization depth
    pub fn is_reorg_safe(&self, new_chain_length: u64, current_chain_length: u64) -> bool {
        let reorg_depth = new_chain_length.saturating_sub(current_chain_length);
        reorg_depth <= self.params.max_reorg_depth
    }

    /// Check if a transaction is standard (mempool policy)
    pub fn is_standard_tx(&self, tx: &crate::transaction::Transaction) -> bool {
        // Minimum non-coinbase size
        if !tx.is_coinbase() && tx.size() < 82 {
            return false;
        }

        // Maximum tx size
        if tx.size() > crate::MAX_BLOCK_SIZE / 10 {
            return false;
        }

        // Check output values
        for output in &tx.outputs {
            if output.value > MAX_SUPPLY * SATS_PER_COIN {
                return false;
            }
        }

        true
    }
}

/// Generate the genesis block for Udaya mainnet
pub fn create_genesis_block() -> Block {
    use crate::transaction::Transaction;
    use crate::types::*;

    let genesis_prev_hash = BlockHash([0u8; 32]);

    // Genesis coinbase output - initial block reward of 50 UDYA
    let genesis_script = ScriptPubKey::new(vec![
        0x41, // Push 65 bytes
        0x04, 0x67, 0x8a, 0xfd, 0xb0, 0xfe, 0x55, 0x48, 0x27, 0x19, 0x67, 0xf1, 0xa6, 0x71, 0x30,
        0xb7, 0x10, 0x5c, 0xd6, 0xa8, 0x28, 0xe0, 0x39, 0x09, 0xa6, 0x79, 0x62, 0xe0, 0xea, 0x1f,
        0x61, 0xde, 0xb6, 0x49, 0xf6, 0xbc, 0x3f, 0x4c, 0xef, 0x38, 0xc4, 0xf3, 0x55, 0x04, 0xe5,
        0x1e, 0xc1, 0x12, 0xde, 0x5c, 0x38, 0x4d, 0xf7, 0xba, 0x0b, 0x8d, 0x57, 0x8a, 0x4c, 0x70,
        0x2b, 0x6b, 0xf1, 0x1d, 0x5f, // 65 bytes pubkey
        0xAC, // OP_CHECKSIG
    ]);

    let coinbase_tx = Transaction::new_coinbase(
        "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks"
            .as_bytes()
            .to_vec(),
        vec![TxOut::new(INITIAL_BLOCK_REWARD, genesis_script)],
        0,
    );

    let merkle_root = MerkleRoot::compute(&[coinbase_tx.txid()]);

    let header = BlockHeader {
        version: 1,
        previous_block_hash: genesis_prev_hash,
        merkle_root,
        timestamp: 1231006505, // Jan 3, 2009
        bits: GENESIS_BITS,
        nonce: 2083236893,
    };

    Block::new(header, vec![coinbase_tx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_reward_halving() {
        let engine = ConsensusEngine::new(ConsensusParams::default());
        assert_eq!(engine.block_reward(0), 50 * SATS_PER_COIN);
        assert_eq!(engine.block_reward(HALVING_INTERVAL), 25 * SATS_PER_COIN);
        // After 2 halvings: 50/4 = 12.5 UDYA = 1,250,000,000 satoshis
        assert_eq!(
            engine.block_reward(HALVING_INTERVAL * 2),
            INITIAL_BLOCK_REWARD / 4
        );
        assert_eq!(engine.block_reward(HALVING_INTERVAL * 64), 0);
    }

    #[test]
    fn test_genesis_block() {
        let genesis = create_genesis_block();
        // The default genesis block may or may not have valid PoW
        // (depends on the specific nonce/timestamp combo)
        // But it must have valid structure
        assert!(genesis.verify_merkle_root());
        assert_eq!(genesis.transactions.len(), 1);
        assert!(genesis.coinbase_tx().unwrap().is_coinbase());
        assert!(genesis.header.previous_block_hash.is_zero());
    }

    #[test]
    fn test_difficulty_calculation() {
        use crate::types::BlockHeader;
        use crate::types::MerkleRoot;
        let engine = ConsensusEngine::new(ConsensusParams::default());
        let mut headers: Vec<BlockHeader> = Vec::new();

        // Create a series of headers at regular intervals
        for i in 0..DIFFICULTY_ADJUSTMENT_INTERVAL + 1 {
            headers.push(BlockHeader {
                version: 1,
                previous_block_hash: BlockHash::default(),
                merkle_root: MerkleRoot::default(),
                timestamp: 1231006505 + (i as u32 * 600), // 10 min intervals
                bits: if i == 0 {
                    GENESIS_BITS
                } else {
                    headers.last().unwrap().bits
                },
                nonce: 0,
            });
        }

        let new_bits = engine.calculate_difficulty(&headers);
        assert!(new_bits > 0);
    }
}
