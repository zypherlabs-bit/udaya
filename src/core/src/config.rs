use serde::{Deserialize, Serialize};

/// Udaya node configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct UdayaConfig {
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub consensus: ConsensusConfig,
    pub mining: MiningConfig,
    pub wallet: WalletConfig,
    pub rpc: RPCConfig,
    pub logging: LoggingConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_port: u16,
    pub external_ip: Option<String>,
    pub max_peers: u32,
    pub enable_upnp: bool,
    pub preferred_peers: Vec<String>,
    pub ban_duration_secs: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
    pub prune_blocks: bool,
    pub prune_target_gb: u64,
    pub db_cache_size_mb: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub network: String, // mainnet, testnet, regtest
    pub min_tx_fee: u64,
    pub max_block_size: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    pub enable: bool,
    pub mine_on_startup: bool,
    pub num_miner_threads: usize,
    pub coinbase_address: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub enable: bool,
    pub wallet_file: String,
    pub default_fee_rate: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RPCConfig {
    pub enable: bool,
    pub listen_addr: String,
    pub listen_port: u16,
    pub username: String,
    pub password: String,
    pub enable_ws: bool,
    pub ws_port: u16,
    pub cors_domains: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub enable_json: bool,
}

impl Default for UdayaConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                listen_port: 9798,
                external_ip: None,
                max_peers: 125,
                enable_upnp: false,
                preferred_peers: Vec::new(),
                ban_duration_secs: 86400,
            },
            storage: StorageConfig {
                data_dir: "data/Udaya".to_string(),
                prune_blocks: false,
                prune_target_gb: 10,
                db_cache_size_mb: 512,
            },
            consensus: ConsensusConfig {
                network: "mainnet".to_string(),
                min_tx_fee: 1000,
                max_block_size: 1_000_000,
            },
            mining: MiningConfig {
                enable: false,
                mine_on_startup: false,
                num_miner_threads: 1,
                coinbase_address: None,
            },
            wallet: WalletConfig {
                enable: true,
                wallet_file: "wallet.dat".to_string(),
                default_fee_rate: 10,
            },
            rpc: RPCConfig {
                enable: true,
                listen_addr: "127.0.0.1".to_string(),
                listen_port: 8332,
                username: String::new(),
                password: String::new(),
                enable_ws: true,
                ws_port: 8333,
                cors_domains: vec!["*".to_string()],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
                enable_json: false,
            },
        }
    }
}

impl UdayaConfig {
    /// Load configuration from a file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
