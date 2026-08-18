use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Udaya API Infrastructure
/// JSON-RPC, REST, and WebSocket API endpoints

/// JSON-RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

/// JSON-RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// RPC method definitions
#[derive(Debug, Clone)]
pub enum RpcMethod {
    // Blockchain
    GetBlockchainInfo,
    GetBlockCount,
    GetBlockHash(u64),
    GetBlock(String),
    GetTransaction(String),
    GetTxOut(String, u32),
    GetMempoolInfo,
    GetMempoolEntry(String),

    // Mining
    GetMiningInfo,
    GetNetworkHashPs,
    SubmitBlock(String),
    GetBlockTemplate,

    // Wallet
    GetBalance,
    GetNewAddress,
    GetAddressesByLabel(String),
    SendToAddress(String, f64),
    ListUnspent,
    ListTransactions,

    // Network
    GetPeerInfo,
    GetNetworkInfo,
    AddNode(String, String),
    GetConnectionCount,

    // Control
    Stop,
    Uptime,
    GetInfo,
}

/// RPC handler registry
pub struct RpcHandler {
    handlers: HashMap<String, Arc<dyn Fn(JsonRpcRequest) -> JsonRpcResponse + Send + Sync>>,
}

impl Default for RpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, method: &str, handler: F)
    where
        F: Fn(JsonRpcRequest) -> JsonRpcResponse + Send + Sync + 'static,
    {
        self.handlers.insert(method.to_string(), Arc::new(handler));
    }

    pub fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        if request.method.is_empty() {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(RpcError {
                    code: -32600,
                    message: "Invalid request: method cannot be empty".to_string(),
                }),
            };
        }
        if let Some(handler) = self.handlers.get(&request.method) {
            handler(request)
        } else {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(RpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
            }
        }
    }
}

/// REST API endpoints
pub struct RestApi {
    pub blockchain_info: Option<serde_json::Value>,
    pub mempool_snapshot: Option<serde_json::Value>,
    pub network_stats: Option<serde_json::Value>,
}

impl Default for RestApi {
    fn default() -> Self {
        Self::new()
    }
}

impl RestApi {
    pub fn new() -> Self {
        Self {
            blockchain_info: None,
            mempool_snapshot: None,
            network_stats: None,
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub rpc_enabled: bool,
    pub rpc_bind: String,
    pub rpc_port: u16,
    pub rest_enabled: bool,
    pub rest_bind: String,
    pub rest_port: u16,
    pub ws_enabled: bool,
    pub ws_bind: String,
    pub ws_port: u16,
    pub allowed_hosts: Vec<String>,
    pub rate_limit_per_sec: u32,
    pub max_body_size: usize,
    pub enable_auth: bool,
    pub rpc_user: String,
    pub rpc_password: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            rpc_enabled: true,
            rpc_bind: "127.0.0.1".to_string(),
            rpc_port: 8332,
            rest_enabled: true,
            rest_bind: "127.0.0.1".to_string(),
            rest_port: 8334,
            ws_enabled: true,
            ws_bind: "127.0.0.1".to_string(),
            ws_port: 8335,
            allowed_hosts: vec!["127.0.0.1".to_string()],
            rate_limit_per_sec: 100,
            max_body_size: 10_000_000,
            enable_auth: true,
            rpc_user: String::new(),
            rpc_password: String::new(),
        }
    }
}

/// Create standard blockchain info response
pub fn blockchain_info_to_json(
    chain: &str,
    blocks: u64,
    headers: u64,
    best_block_hash: &str,
    difficulty: f64,
    median_time: u64,
    chain_work: &str,
    size_on_disk: u64,
    pruned: bool,
    softforks: Vec<SoftForkInfo>,
) -> serde_json::Value {
    serde_json::json!({
        "chain": chain,
        "blocks": blocks,
        "headers": headers,
        "bestblockhash": best_block_hash,
        "difficulty": difficulty,
        "mediantime": median_time,
        "chainwork": chain_work,
        "size_on_disk": size_on_disk,
        "pruned": pruned,
        "softforks": softforks,
    })
}

/// Soft fork information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftForkInfo {
    pub id: String,
    pub version: u32,
    pub reject: SoftForkReject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftForkReject {
    pub status: bool,
    pub found: u64,
    pub required: u64,
    pub window: u64,
}

/// Transaction output info for RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutInfo {
    pub bestblock: String,
    pub confirmations: u64,
    pub value: f64,
    pub script_pub_key: ScriptPubKeyInfo,
    pub coinbase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPubKeyInfo {
    pub asm: String,
    pub hex: String,
    pub req_sigs: u32,
    pub type_: String,
    pub addresses: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_handler_registration() {
        let mut handler = RpcHandler::new();

        handler.register("getblockchaininfo", |req| JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(serde_json::json!({"chain": "mainnet", "blocks": 0})),
            error: None,
        });

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getblockchaininfo".to_string(),
            params: vec![],
        };

        let response = handler.handle(request);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_rpc_method_not_found() {
        let handler = RpcHandler::new();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "nonexistent".to_string(),
            params: vec![],
        };

        let response = handler.handle(request);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[test]
    fn test_blockchain_info_json() {
        let info = blockchain_info_to_json(
            "mainnet",
            0,
            0,
            "0000000000000000000000000000000000000000000000000000000000000000",
            1.0,
            0,
            "00",
            0,
            false,
            vec![],
        );

        assert_eq!(info["chain"], "mainnet");
        assert_eq!(info["blocks"], 0);
    }
}
