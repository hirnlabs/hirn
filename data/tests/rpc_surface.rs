//! The JSON-RPC surface: envelopes, doc methods, query, mutate, subscriptions.

use std::path::Path;

use hirn_data::backend::DataBackend;
use hirn_data::rpc::codes;
use hirn_data::rpc::{Request, RpcServer};
use hirn_data::types::{DocPath, NodeHandle};
use serde_json::{Value, json};

const NOTE_SCHEMA: &str = r#"
required = ["title"]

[properties.title]
type = "string"

[properties.status]
type = "string"
"#;

const NOTE: &str = "---
title: Protocol note
status: draft
---

# RPC

The surface is transport agnostic.
";

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn server() -> (tempfile::TempDir, RpcServer) {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "notes/schema.toml", NOTE_SCHEMA);
    write_file(dir.path(), "notes/note.md", NOTE);
    let backend = DataBackend::open(dir.path()).unwrap();
    (dir, RpcServer::new(backend))
}

fn call(server: &RpcServer, id: i64, method: &str, params: Value) -> Value {
    let response = server.handle(&Request {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(id)),
        method: method.to_string(),
        params,
    });
    serde_json::to_value(response).unwrap()
}

fn result(response: &Value) -> &Value {
    assert!(response["error"].is_null(), "unexpected error: {response}");
    &response["result"]
}

#[test]
fn load_returns_frontmatter_blocks_and_handles() {
    let (_dir, server) = server();
    let response = call(
        &server,
        1,
        "data.doc.load",
        json!({ "doc": "notes/note.md" }),
    );
    let result = result(&response);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(result["doc_path"], "notes/note.md");
    assert_eq!(result["type"], "notes");
    assert_eq!(result["frontmatter"]["title"], "Protocol note");

    let blocks = result["blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0]["kind"], "heading");

    let handles = result["handles"].as_array().unwrap();
    assert_eq!(handles.len(), 2);
    assert_eq!(handles[0]["doc_path"], "notes/note.md");
    assert_eq!(
        handles[0]["block_id"].as_str().unwrap().len(),
        10,
        "handles carry the stable block id"
    );
}

#[test]
fn move_block_reorders_and_persists() {
    let (dir, server) = server();
    call(
        &server,
        1,
        "data.doc.load",
        json!({ "doc": "notes/note.md" }),
    );
    let response = call(
        &server,
        2,
        "data.doc.moveBlock",
        json!({ "doc": "notes/note.md", "from": 0, "to": 1 }),
    );
    result(&response);

    let written = std::fs::read_to_string(dir.path().join("notes/note.md")).unwrap();
    let heading = written.find("# RPC").unwrap();
    let paragraph = written.find("The surface is transport agnostic.").unwrap();
    assert!(
        paragraph < heading,
        "heading moved after the paragraph:\n{written}"
    );
}

#[test]
fn mutate_set_and_append_round_trip_through_markdown() {
    let (dir, server) = server();
    let loaded = call(
        &server,
        1,
        "data.doc.load",
        json!({ "doc": "notes/note.md" }),
    );
    let handle = &result(&loaded)["handles"][0];

    let response = call(
        &server,
        2,
        "data.mutate",
        json!({
            "op": "set",
            "handle": { "doc_path": "notes/note.md", "property_key": "status" },
            "value": "done",
        }),
    );
    result(&response);

    let response = call(
        &server,
        3,
        "data.mutate",
        json!({ "op": "append", "handle": handle, "text": "Appended by the SDK." }),
    );
    result(&response);

    let written = std::fs::read_to_string(dir.path().join("notes/note.md")).unwrap();
    assert!(written.contains("status: done"));
    assert!(written.contains("Appended by the SDK."));
}

#[test]
fn query_serves_sql_graph_and_text() {
    let (_dir, server) = server();
    call(
        &server,
        1,
        "data.doc.load",
        json!({ "doc": "notes/note.md" }),
    );

    let rows = result(&call(
        &server,
        2,
        "data.query",
        json!({ "sql": "SELECT title FROM type_notes" }),
    ))
    .clone();
    assert_eq!(rows["rows"][0]["title"], "Protocol note");

    let rows = result(&call(
        &server,
        3,
        "data.query",
        json!({ "graph": "MATCH (d:Document) RETURN count(d) AS n" }),
    ))
    .clone();
    assert_eq!(rows["rows"][0]["n"], 1);

    let rows = result(&call(
        &server,
        4,
        "data.query",
        json!({ "graph": "MATCH (d:Document {is_ghost: false}) RETURN d.path AS path" }),
    ))
    .clone();
    assert_eq!(rows["rows"][0]["path"], "notes/note.md");

    let rows = result(&call(
        &server,
        5,
        "data.query",
        json!({ "text": "transport", "limit": 5 }),
    ))
    .clone();
    assert_eq!(rows["rows"].as_array().unwrap().len(), 1);

    let rejected = call(
        &server,
        6,
        "data.query",
        json!({ "sql": "DELETE FROM documents" }),
    );
    assert_eq!(rejected["error"]["code"], codes::INVALID_PARAMS);

    let rejected = call(
        &server,
        7,
        "data.query",
        json!({ "sql": "WITH x AS (SELECT 1) DELETE FROM documents" }),
    );
    assert_eq!(
        rejected["error"]["code"],
        codes::INVALID_PARAMS,
        "a mutation hidden in a CTE is refused"
    );

    let findings = result(&call(
        &server,
        8,
        "data.lint",
        json!({ "doc": "notes/note.md" }),
    ))
    .clone();
    assert!(
        findings["findings"].is_array(),
        "data.lint answers with a findings array: {findings}"
    );
}

#[test]
fn delete_and_rename_keep_every_tier_consistent() {
    let (dir, server) = server();
    call(
        &server,
        1,
        "data.doc.load",
        json!({ "doc": "notes/note.md" }),
    );

    let response = call(
        &server,
        2,
        "data.doc.rename",
        json!({ "doc": "notes/note.md", "to": "notes/renamed.md" }),
    );
    result(&response);
    assert!(dir.path().join("notes/renamed.md").exists());
    assert!(!dir.path().join("notes/note.md").exists());
    let rows = result(&call(
        &server,
        3,
        "data.query",
        json!({ "sql": "SELECT doc_path FROM documents" }),
    ))
    .clone();
    assert_eq!(rows["rows"][0]["doc_path"], "notes/renamed.md");

    let response = call(
        &server,
        4,
        "data.doc.delete",
        json!({ "doc": "notes/renamed.md" }),
    );
    result(&response);
    assert!(!dir.path().join("notes/renamed.md").exists());
    let rows = result(&call(
        &server,
        5,
        "data.query",
        json!({ "sql": "SELECT count(*) AS n FROM documents" }),
    ))
    .clone();
    assert_eq!(rows["rows"][0]["n"], 0);
}

#[test]
fn change_type_moves_the_file_into_the_type_folder() {
    let (dir, server) = server();
    write_file(
        dir.path(),
        "journal/schema.toml",
        "required = [\"title\"]\n",
    );
    call(
        &server,
        1,
        "data.doc.load",
        json!({ "doc": "notes/note.md" }),
    );

    let response = call(
        &server,
        2,
        "data.doc.changeType",
        json!({ "doc": "notes/note.md", "type": "journal" }),
    );
    let changed = result(&response);
    assert_eq!(changed["doc_path"], "journal/note.md");
    assert!(dir.path().join("journal/note.md").exists());

    let rows = result(&call(
        &server,
        3,
        "data.query",
        json!({ "sql": "SELECT type_name FROM documents WHERE doc_path = 'journal/note.md'" }),
    ))
    .clone();
    assert_eq!(rows["rows"][0]["type_name"], "journal");
}

#[test]
fn subscriptions_register_and_unregister() {
    let (_dir, server) = server();
    let id = result(&call(&server, 1, "data.subscribe", json!({}))).clone();
    let subscription = id["subscription"].as_str().unwrap().to_string();
    assert!(subscription.starts_with("sub-"));

    let response = call(
        &server,
        2,
        "data.unsubscribe",
        json!({ "subscription": subscription }),
    );
    assert_eq!(result(&response)["removed"], true);
}

#[test]
fn a_mutation_re_emits_to_every_subscription() {
    let (_dir, server) = server();
    let first = result(&call(&server, 1, "data.subscribe", json!({}))).clone()["subscription"]
        .as_str()
        .unwrap()
        .to_string();
    let second = result(&call(&server, 2, "data.subscribe", json!({}))).clone()["subscription"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(first, second);

    let handle = NodeHandle::property(DocPath::new("notes/note.md"), "status");
    let change = RpcServer::change_for(&handle, "set");
    let messages = server.notifications(&change);
    assert_eq!(messages.len(), 2, "one notification per subscription");
    for message in &messages {
        assert_eq!(message["jsonrpc"], "2.0");
        assert_eq!(message["method"], "data.subscription.event");
        assert!(message.get("id").is_none(), "notifications carry no id");
        assert_eq!(message["params"]["event"]["doc_path"], "notes/note.md");
        assert_eq!(message["params"]["event"]["operation"], "set");
        assert_eq!(
            message["params"]["event"]["handles"][0]["property_key"],
            "status"
        );
    }

    call(
        &server,
        3,
        "data.unsubscribe",
        json!({ "subscription": first }),
    );
    assert_eq!(
        server.notifications(&change).len(),
        1,
        "an unsubscribed client stops receiving"
    );
}

#[test]
fn mutations_enqueue_notifications_for_drain() {
    let (_dir, server) = server();
    // The default subscription includes `data.mutate`.
    call(&server, 1, "data.subscribe", json!({}));

    let response = call(
        &server,
        2,
        "data.mutate",
        json!({
            "op": "set",
            "handle": { "doc_path": "notes/note.md", "property_key": "status" },
            "value": "review",
        }),
    );
    let result = result(&response);
    assert_eq!(
        result["notifications"].as_array().unwrap().len(),
        1,
        "the response carries the emitted notification"
    );

    let drained = server.drain_notifications();
    assert_eq!(drained.len(), 1, "the transport can take the pending batch");
    assert_eq!(drained[0]["params"]["event"]["operation"], "set");
    assert!(
        server.drain_notifications().is_empty(),
        "draining is one-shot"
    );
}

#[test]
fn subscriptions_filter_by_method() {
    let (_dir, server) = server();
    call(
        &server,
        1,
        "data.subscribe",
        json!({ "methods": ["data.doc.load"] }),
    );
    let response = call(
        &server,
        2,
        "data.mutate",
        json!({
            "op": "set",
            "handle": { "doc_path": "notes/note.md", "property_key": "status" },
            "value": "done",
        }),
    );
    assert!(
        result(&response)["notifications"]
            .as_array()
            .unwrap()
            .is_empty(),
        "a load subscriber does not see a mutate"
    );
    assert!(server.drain_notifications().is_empty());
}

#[test]
fn unknown_methods_and_bad_envelopes_are_errors() {
    let (_dir, server) = server();
    let unknown = call(&server, 1, "data.nope", json!({}));
    assert_eq!(unknown["error"]["code"], codes::METHOD_NOT_FOUND);

    let parse = server.handle_json("{not json");
    let parsed: Value = serde_json::from_str(&parse).unwrap();
    assert_eq!(parsed["error"]["code"], codes::PARSE_ERROR);

    let wrong_version = server.handle_json(
        r#"{"jsonrpc":"1.0","id":2,"method":"data.doc.load","params":{"doc":"notes/note.md"}}"#,
    );
    let parsed: Value = serde_json::from_str(&wrong_version).unwrap();
    assert_eq!(parsed["error"]["code"], codes::INVALID_REQUEST);
}
