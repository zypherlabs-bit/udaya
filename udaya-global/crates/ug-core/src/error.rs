//! Error types for the Udaya Global core.
//!
//! Every externally reachable decoding and validation path returns a typed
//! error. No validation path panics on attacker-controlled input.

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("canonical decoding failed: {0}")]
    Codec(String),

    #[error("invalid chain parameters: {0}")]
    Chain(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("transaction conflicts with another transaction spending {0}")]
    DoubleSpend(String),

    #[error("missing input {0}")]
    MissingInput(String),

    #[error("invalid signature on input {0}")]
    InvalidSignature(usize),

    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("state error: {0}")]
    State(String),

    #[error("mempool rejected transaction: {0}")]
    Mempool(String),

    #[error("genesis error: {0}")]
    Genesis(String),

    #[error("fee policy error: {0}")]
    Fee(String),

    #[error("reward accounting error: {0}")]
    Reward(String),

    #[error("io error: {0}")]
    Io(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Genesis(format!("json: {e}"))
    }
}
