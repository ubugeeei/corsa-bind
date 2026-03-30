pub use crate::RpcResponseError;
use crate::{Result, TsgoError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tsgo_rs_core::fast::{CompactString, compact_format};

use super::RequestId;

/// Raw JSON-RPC envelope used on the wire.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use tsgo_rs_jsonrpc::{RawMessage, RequestId};
///
/// let message = RawMessage::request(RequestId::integer(1), "ping", json!({ "value": 1 }));
/// assert_eq!(message.method.as_deref(), Some("ping"));
/// assert_eq!(message.id, Some(RequestId::integer(1)));
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawMessage {
    /// JSON-RPC protocol version, normally `"2.0"`.
    #[serde(default = "jsonrpc_version")]
    pub jsonrpc: CompactString,
    /// Request/response identifier, absent for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    /// Method name for requests and notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<CompactString>,
    /// Request or notification parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Successful response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcResponseError>,
}

fn jsonrpc_version() -> CompactString {
    CompactString::from("2.0")
}

impl RawMessage {
    /// Builds a JSON-RPC request message.
    pub fn request(id: RequestId, method: impl Into<CompactString>, params: Value) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: Some(id),
            method: Some(method.into()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    /// Builds a JSON-RPC notification message.
    pub fn notification(method: impl Into<CompactString>, params: Value) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: None,
            method: Some(method.into()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    /// Builds a successful JSON-RPC response message.
    pub fn response(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// Builds an error JSON-RPC response message.
    pub fn error(id: RequestId, error: RpcResponseError) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// Classifies the envelope into a higher-level message kind.
    pub fn kind(&self) -> Result<MessageKind> {
        match (&self.id, &self.method, &self.result, &self.error) {
            (Some(id), Some(method), _, _) => Ok(MessageKind::Request {
                id: id.clone(),
                method: method.clone(),
                params: self.params.clone().unwrap_or(Value::Null),
            }),
            (None, Some(method), _, _) => Ok(MessageKind::Notification {
                method: method.clone(),
                params: self.params.clone().unwrap_or(Value::Null),
            }),
            (Some(id), None, result, error) => Ok(MessageKind::Response {
                id: id.clone(),
                result: result.clone(),
                error: error.clone(),
            }),
            _ => Err(TsgoError::UnexpectedMessage(compact_format(format_args!(
                "{self:?}"
            )))),
        }
    }
}

/// Parsed view of a [`RawMessage`].
#[derive(Clone, Debug)]
pub enum MessageKind {
    /// JSON-RPC request envelope.
    Request {
        /// Request identifier.
        id: RequestId,
        /// Method name.
        method: CompactString,
        /// Request parameters.
        params: Value,
    },
    /// JSON-RPC notification envelope.
    Notification {
        /// Method name.
        method: CompactString,
        /// Notification parameters.
        params: Value,
    },
    /// JSON-RPC response envelope.
    Response {
        /// Request identifier being answered.
        id: RequestId,
        /// Successful result body, when present.
        result: Option<Value>,
        /// Error body, when present.
        error: Option<RpcResponseError>,
    },
}
