//! IPC and JSON-RPC protocol definitions
//!
//! Defines the request/response protocol for communication between
//! Electron main process and Rust torrent engine.

use serde::{Deserialize, Serialize};

/// JSON-RPC request wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

/// JSON-RPC response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Torrent engine RPC methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum EngineMethod {
    // Torrent management
    #[serde(rename = "torrent.add")]
    AddTorrent { path: String },

    #[serde(rename = "torrent.remove")]
    RemoveTorrent { id: String },

    #[serde(rename = "torrent.pause")]
    PauseTorrent { id: String },

    #[serde(rename = "torrent.resume")]
    ResumeTorrent { id: String },

    #[serde(rename = "torrent.list")]
    ListTorrents,

    #[serde(rename = "torrent.info")]
    GetTorrentInfo { id: String },

    // Engine control
    #[serde(rename = "engine.status")]
    EngineStatus,

    #[serde(rename = "engine.config")]
    GetConfig,

    #[serde(rename = "engine.shutdown")]
    Shutdown,
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: serde_json::Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

impl JsonRpcResponse {
    pub fn success(result: serde_json::Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(code: i32, message: impl Into<String>, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request() {
        let req = JsonRpcRequest::new(
            "test.method",
            serde_json::json!({"key": "value"}),
            1,
        );
        assert_eq!(req.method, "test.method");
        assert_eq!(req.id, 1);
    }

    #[test]
    fn test_jsonrpc_response() {
        let resp = JsonRpcResponse::success(serde_json::json!({"result": true}), 1);
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error() {
        let resp = JsonRpcResponse::error(-32600, "Invalid Request", 1);
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }
}
