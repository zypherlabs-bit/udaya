use thiserror::Error;

/// Core blockchain errors
#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Block validation error: {0}")]
    BlockValidation(String),

    #[error("Transaction validation error: {0}")]
    TransactionValidation(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Mining error: {0}")]
    Mining(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Peer error: {0}")]
    Peer(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Insufficient funds: have {have}, need {need}")]
    InsufficientFunds { have: u64, need: u64 },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid proof of work")]
    InvalidPoW,

    #[error("Orphan block: missing parent {0}")]
    OrphanBlock(String),

    #[error("Chain reorganization too deep")]
    ReorgTooDeep,

    #[error("Checkpoint mismatch at height {height}")]
    CheckpointMismatch { height: u64 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<BlockchainError> for anyhow::Error {
    fn from(err: BlockchainError) -> Self {
        anyhow::anyhow!("{}", err)
    }
}