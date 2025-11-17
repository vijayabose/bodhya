/// JSON-RPC 2.0 message types for MCP communication
///
/// This module implements the JSON-RPC 2.0 protocol specification
/// for use with Model Context Protocol (MCP) servers.
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Request ID (for matching responses)
    pub id: RequestId,
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Request ID (matches request)
    pub id: RequestId,
    /// Result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Request ID (can be number or string)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Number(u64),
    String(String),
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: RequestId, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }

    /// Create request with numeric ID
    pub fn with_id(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self::new(RequestId::Number(id), method, params)
    }
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if response is an error
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

impl JsonRpcError {
    /// Parse error (-32700)
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: message.into(),
            data: None,
        }
    }

    /// Invalid request (-32600)
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: message.into(),
            data: None,
        }
    }

    /// Method not found (-32601)
    pub fn method_not_found(message: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: message.into(),
            data: None,
        }
    }

    /// Invalid params (-32602)
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
            data: None,
        }
    }

    /// Internal error (-32603)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: message.into(),
            data: None,
        }
    }

    /// Server error (-32000 to -32099)
    pub fn server_error(code: i32, message: impl Into<String>) -> Self {
        assert!(
            (-32099..=-32000).contains(&code),
            "Server error codes must be between -32000 and -32099"
        );
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req =
            JsonRpcRequest::with_id(1, "test_method", Some(serde_json::json!({"key": "value"})));

        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.id, RequestId::Number(1));
        assert_eq!(req.method, "test_method");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_response_success() {
        let resp =
            JsonRpcResponse::success(RequestId::Number(1), serde_json::json!({"result": "ok"}));

        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.is_success());
        assert!(!resp.is_error());
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_error() {
        let error = JsonRpcError::method_not_found("Method 'foo' not found");
        let resp = JsonRpcResponse::error(RequestId::Number(1), error);

        assert_eq!(resp.jsonrpc, "2.0");
        assert!(!resp.is_success());
        assert!(resp.is_error());
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcError::parse_error("test").code, -32700);
        assert_eq!(JsonRpcError::invalid_request("test").code, -32600);
        assert_eq!(JsonRpcError::method_not_found("test").code, -32601);
        assert_eq!(JsonRpcError::invalid_params("test").code, -32602);
        assert_eq!(JsonRpcError::internal_error("test").code, -32603);
    }

    #[test]
    fn test_serialization() {
        let req = JsonRpcRequest::with_id(1, "test", None);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test");
        assert_eq!(req.id, RequestId::Number(1));
    }
}
