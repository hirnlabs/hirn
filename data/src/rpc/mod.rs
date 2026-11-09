//! JSON-RPC 2.0 surface. Transport-agnostic: no Tauri, no channels.
//!
//! Methods mirror the settled surface (HIR-81 resolution):
//! `data.doc.load`, `data.doc.moveBlock`, `data.doc.rename`,
//! `data.doc.delete`, `data.doc.changeType`, `data.query`, `data.mutate`,
//! `data.subscribe` / `data.unsubscribe`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::backend::DataBackend;
use crate::error::{Error, Result};
use crate::types::{
    BlockId, BlockKind, DocPath, NodeHandle, PropertyValue, SubscriptionId, TypeName,
};

/// JSON-RPC version marker.
pub const JSONRPC_VERSION: &str = "2.0";

/// One request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// One response envelope: exactly one of `result` or `error` is set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// The error codes this surface uses.
pub mod codes {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
}

/// A document changed, for re-emission to subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeNotification {
    pub doc_path: DocPath,
    /// Handles touched by the mutation.
    pub handles: Vec<NodeHandle>,
    /// The mutation that caused it: `set`, `delete` or `append`.
    pub operation: String,
}

/// The RPC handler over one [`DataBackend`].
pub struct RpcServer {
    backend: Arc<Mutex<DataBackend>>,
    subscriptions: Arc<Mutex<HashMap<SubscriptionId, Vec<String>>>>,
    pending: Arc<Mutex<Vec<Value>>>,
    next_subscription: AtomicU64,
}

impl RpcServer {
    /// Wrap a backend.
    pub fn new(backend: DataBackend) -> Self {
        Self {
            backend: Arc::new(Mutex::new(backend)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            pending: Arc::new(Mutex::new(Vec::new())),
            next_subscription: AtomicU64::new(1),
        }
    }

    /// Take the notifications enqueued by completed mutations.
    ///
    /// A transport calls this after [`RpcServer::handle`] returns and writes
    /// the messages to its outgoing stream.
    pub fn drain_notifications(&self) -> Vec<Value> {
        self.pending
            .lock()
            .map(|mut pending| std::mem::take(&mut *pending))
            .unwrap_or_default()
    }

    /// Handle one request and return the response envelope.
    pub fn handle(&self, request: &Request) -> Response {
        if request.jsonrpc != JSONRPC_VERSION {
            return Response::error(
                request.id.clone(),
                codes::INVALID_REQUEST,
                "jsonrpc must be \"2.0\"",
            );
        }
        match self.dispatch(&request.method, &request.params) {
            Ok(result) => Response {
                jsonrpc: JSONRPC_VERSION.to_string(),
                id: request.id.clone(),
                result: Some(result),
                error: None,
            },
            Err(error) => {
                let (code, message) = match &error {
                    Error::NotFound(msg) => (codes::INVALID_PARAMS, format!("not found: {msg}")),
                    Error::Invalid(msg) => (codes::INVALID_PARAMS, msg.clone()),
                    Error::Rpc(msg) => (codes::METHOD_NOT_FOUND, msg.clone()),
                    other => (codes::INTERNAL_ERROR, other.to_string()),
                };
                Response::error(request.id.clone(), code, &message)
            }
        }
    }

    /// Handle a raw JSON string, as a transport would receive it.
    pub fn handle_json(&self, raw: &str) -> String {
        let request: Request = match serde_json::from_str(raw) {
            Ok(request) => request,
            Err(error) => {
                return serde_json::to_string(&Response::error(
                    None,
                    codes::PARSE_ERROR,
                    &error.to_string(),
                ))
                .unwrap_or_default();
            }
        };
        serde_json::to_string(&self.handle(&request)).unwrap_or_default()
    }

    fn dispatch(&self, method: &str, params: &Value) -> Result<Value> {
        match method {
            "data.doc.load" => self.doc_load(params),
            "data.doc.moveBlock" => self.doc_move_block(params),
            "data.doc.rename" => self.doc_rename(params),
            "data.doc.delete" => self.doc_delete(params),
            "data.doc.changeType" => self.doc_change_type(params),
            "data.query" => self.query(params),
            "data.lint" => self.lint(params),
            "data.mutate" => self.mutate(params),
            "data.subscribe" => self.subscribe(params),
            "data.unsubscribe" => self.unsubscribe(params),
            other => Err(Error::Rpc(format!("unknown method `{other}`"))),
        }
    }

    fn doc_load(&self, params: &Value) -> Result<Value> {
        let doc = doc_param(params)?;
        let mut backend = self.lock()?;
        let loaded = backend.load(&doc)?;
        Ok(json!({
            "doc_path": loaded.path.as_str(),
            "type": loaded.type_name.as_ref().map(TypeName::as_str),
            "frontmatter": loaded.snapshot.frontmatter.iter()
                .map(|(key, value)| (key.clone(), value.to_sql()))
                .collect::<serde_json::Map<_, _>>(),
            "blocks": loaded.snapshot.blocks,
            "handles": loaded.snapshot.blocks.iter()
                .map(|block| NodeHandle::block(loaded.path.clone(), block.id.clone()))
                .collect::<Vec<_>>(),
        }))
    }

    fn doc_move_block(&self, params: &Value) -> Result<Value> {
        let doc = doc_param(params)?;
        let from = usize_param(params, "from")?;
        let to = usize_param(params, "to")?;
        let mut backend = self.lock()?;
        backend.load(&doc)?;
        let document = backend
            .document(&doc)
            .ok_or_else(|| Error::NotFound(doc.to_string()))?;
        document.move_block(from, to)?;
        backend.flush(&doc)?;
        Ok(json!({ "doc_path": doc.as_str(), "from": from, "to": to }))
    }

    fn doc_rename(&self, params: &Value) -> Result<Value> {
        let doc = doc_param(params)?;
        let new_path = DocPath::new(str_param(params, "to")?);
        let mut backend = self.lock()?;
        backend.rename(&doc, &new_path)?;
        Ok(json!({
            "doc_path": new_path.as_str(),
            "previous": doc.as_str(),
        }))
    }

    fn doc_delete(&self, params: &Value) -> Result<Value> {
        let doc = doc_param(params)?;
        let mut backend = self.lock()?;
        backend.delete(&doc)?;
        Ok(json!({ "deleted": doc.as_str() }))
    }

    fn doc_change_type(&self, params: &Value) -> Result<Value> {
        let doc = doc_param(params)?;
        let target = TypeName::new(str_param(params, "type")?);
        let mut backend = self.lock()?;
        let moved = backend.path_in_type(&doc, &target)?;
        let violations = backend.change_type(&doc, &target)?;
        Ok(json!({
            "doc_path": moved.as_str(),
            "previous": doc.as_str(),
            "type": target.as_str(),
            "violations": violations,
        }))
    }

    fn query(&self, params: &Value) -> Result<Value> {
        let backend = self.lock()?;
        if let Some(sql) = params.get("sql").and_then(Value::as_str) {
            return Ok(json!({ "rows": backend.query_sql(sql)? }));
        }
        if let Some(cypher) = params
            .get("graph")
            .and_then(Value::as_str)
            .or_else(|| params.get("cypher").and_then(Value::as_str))
        {
            return Ok(json!({ "rows": backend.query_graph(cypher)? }));
        }
        if let Some(text) = params.get("text").and_then(Value::as_str) {
            let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
            return Ok(json!({ "rows": backend.query_full_text(text, limit)? }));
        }
        if params.get("vector").is_some() {
            return Err(Error::Invalid(
                "vector queries need an embedder: open the vector tier through `SqliteVectorIndex`"
                    .into(),
            ));
        }
        Err(Error::Invalid(
            "data.query needs one of `sql`, `graph`, `text`".into(),
        ))
    }

    fn lint(&self, params: &Value) -> Result<Value> {
        let doc = doc_param(params)?;
        let backend = self.lock()?;
        Ok(json!({ "findings": backend.lint(&doc)? }))
    }

    fn mutate(&self, params: &Value) -> Result<Value> {
        let handle: NodeHandle = serde_json::from_value(
            params
                .get("handle")
                .cloned()
                .ok_or_else(|| Error::Invalid("data.mutate needs `handle`".into()))?,
        )
        .map_err(|e| Error::Invalid(format!("bad handle: {e}")))?;
        let operation = str_param(params, "op")?;
        let mut backend = self.lock()?;
        backend.load(&handle.doc_path)?;
        let document = backend
            .document(&handle.doc_path)
            .ok_or_else(|| Error::NotFound(handle.doc_path.to_string()))?;

        let frontmatter_only = match operation {
            "set" => {
                let value = PropertyValue::from_json(params.get("value").unwrap_or(&Value::Null));
                if let Some(key) = &handle.property_key {
                    document.set_property(key, &value)?;
                    true
                } else if let Some(block_id) = &handle.block_id {
                    let text = match &value {
                        PropertyValue::String(text) => text.clone(),
                        other => other.to_sql().to_string(),
                    };
                    document.set_block_text(block_id, &text)?;
                    false
                } else {
                    return Err(Error::Invalid(
                        "`set` needs a property_key or a block_id on the handle".into(),
                    ));
                }
            }
            "delete" => {
                if let Some(key) = &handle.property_key {
                    document.delete_property(key)?;
                    true
                } else if let Some(block_id) = &handle.block_id {
                    document.delete_block(block_id)?;
                    false
                } else {
                    return Err(Error::Invalid(
                        "`delete` needs a property_key or a block_id on the handle".into(),
                    ));
                }
            }
            "append" => {
                let text = str_param(params, "text")?;
                document.append_block(BlockKind::Paragraph, text)?;
                false
            }
            other => {
                return Err(Error::Invalid(format!(
                    "unknown mutation `{other}`: use set, delete or append"
                )));
            }
        };
        if frontmatter_only {
            backend.flush_frontmatter(&handle.doc_path)?;
        } else {
            backend.flush(&handle.doc_path)?;
        }
        drop(backend);
        let change = Self::change_for(&handle, operation);
        let notifications = self.notifications(&change);
        if let Ok(mut pending) = self.pending.lock() {
            pending.extend(notifications.iter().cloned());
        }
        Ok(json!({
            "handle": handle,
            "op": operation,
            "notifications": notifications,
        }))
    }

    fn subscribe(&self, params: &Value) -> Result<Value> {
        let methods: Vec<String> = params
            .get("methods")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![
                    "data.doc.load".to_string(),
                    "data.query".to_string(),
                    "data.mutate".to_string(),
                ]
            });
        let id = SubscriptionId::new(format!(
            "sub-{}",
            self.next_subscription.fetch_add(1, Ordering::Relaxed)
        ));
        if let Ok(mut subscriptions) = self.subscriptions.lock() {
            subscriptions.insert(id.clone(), methods);
        }
        Ok(json!({ "subscription": id }))
    }

    fn unsubscribe(&self, params: &Value) -> Result<Value> {
        let id = SubscriptionId::new(str_param(params, "subscription")?);
        let removed = self
            .subscriptions
            .lock()
            .map(|mut subscriptions| subscriptions.remove(&id).is_some())
            .unwrap_or(false);
        Ok(json!({ "subscription": id, "removed": removed }))
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, DataBackend>> {
        self.backend
            .lock()
            .map_err(|_| Error::Invalid("backend lock poisoned".into()))
    }

    /// Notifications for the subscriptions a mutation affects.
    ///
    /// Every registered subscription is re-emitted once per mutation, with the
    /// handles the mutation touched, so a client can re-run its query. The
    /// messages are JSON-RPC notifications (no `id`): a transport writes them
    /// to the same stream it answers requests on.
    pub fn notifications(&self, change: &ChangeNotification) -> Vec<Value> {
        let method = operation_method(&change.operation);
        let Ok(subscriptions) = self.subscriptions.lock() else {
            return Vec::new();
        };
        subscriptions
            .iter()
            .filter(|(_, methods)| methods.iter().any(|wanted| wanted == method))
            .map(|(id, _)| {
                json!({
                    "jsonrpc": JSONRPC_VERSION,
                    "method": "data.subscription.event",
                    "params": {
                        "subscription": id.as_str(),
                        "event": change,
                    },
                })
            })
            .collect()
    }

    /// The change a mutation on `handle` produced.
    pub fn change_for(handle: &NodeHandle, operation: &str) -> ChangeNotification {
        ChangeNotification {
            doc_path: handle.doc_path.clone(),
            handles: vec![handle.clone()],
            operation: operation.to_string(),
        }
    }
}

impl Response {
    fn error(id: Option<Value>, code: i64, message: &str) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

/// The RPC method a change operation came through.
fn operation_method(operation: &str) -> &'static str {
    match operation {
        "load" => "data.doc.load",
        "query" => "data.query",
        _ => "data.mutate",
    }
}

fn doc_param(params: &Value) -> Result<DocPath> {
    Ok(DocPath::new(str_param(params, "doc")?))
}

fn str_param<'a>(params: &'a Value, key: &str) -> Result<&'a str> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Invalid(format!("missing string parameter `{key}`")))
}

fn usize_param(params: &Value, key: &str) -> Result<usize> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .ok_or_else(|| Error::Invalid(format!("missing integer parameter `{key}`")))
}

/// A block id, for callers building handles from RPC results.
pub fn block_handle(doc: &DocPath, id: &BlockId) -> NodeHandle {
    NodeHandle::block(doc.clone(), id.clone())
}
