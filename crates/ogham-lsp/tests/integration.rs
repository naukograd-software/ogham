//! Integration tests for ogham-lsp via LspService + tower::Service::call.

use serde_json::{json, Value};
use tower::Service;
use tower_lsp::LspService;

fn build() -> LspService<ogham_lsp::Backend> {
    let (service, socket) = ogham_lsp::build_service();
    // Drive the client socket in background — without this,
    // publish_diagnostics and other server→client notifications block.
    tokio::spawn(async move {
        // socket is a Stream — just consume it silently
        use futures::StreamExt;
        let mut socket = socket;
        while socket.next().await.is_some() {}
    });
    service
}

async fn request(
    service: &mut LspService<ogham_lsp::Backend>,
    method: &'static str,
    id: i64,
    params: Value,
) -> Value {
    let req = tower_lsp::jsonrpc::Request::build(method)
        .id(id)
        .params(params)
        .finish();

    let resp = service.call(req).await.unwrap();
    match resp {
        Some(r) => serde_json::to_value(r).unwrap(),
        None => Value::Null,
    }
}

async fn notify(
    service: &mut LspService<ogham_lsp::Backend>,
    method: &'static str,
    params: Value,
) {
    let req = tower_lsp::jsonrpc::Request::build(method)
        .params(params)
        .finish();
    let _ = service.call(req).await;
}

async fn init_and_open(
    service: &mut LspService<ogham_lsp::Backend>,
    uri: &'static str,
    text: &str,
) {
    // Initialize
    let _ = request(
        service,
        "initialize",
        1,
        json!({
            "capabilities": {},
            "processId": null,
            "rootUri": null,
        }),
    )
    .await;

    notify(service, "initialized", json!({})).await;

    // Open file
    notify(
        service,
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "ogham",
                "version": 1,
                "text": text,
            }
        }),
    )
    .await;
}

// ── Tests ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_initialize_capabilities() {
    let mut service = build();
    let resp = request(
        &mut service,
        "initialize",
        1,
        json!({ "capabilities": {}, "processId": null, "rootUri": null }),
    )
    .await;

    let caps = resp
        .pointer("/result/capabilities")
        .expect("no capabilities");

    assert!(caps.get("hoverProvider").is_some());
    assert!(caps.get("definitionProvider").is_some());
    assert!(caps.get("completionProvider").is_some());
    assert!(caps.get("documentSymbolProvider").is_some());
    assert!(caps.get("referencesProvider").is_some());
    assert!(caps.get("renameProvider").is_some());
    assert!(caps.get("documentFormattingProvider").is_some());
    assert!(caps.get("semanticTokensProvider").is_some());
    assert!(caps.get("codeActionProvider").is_some());
    assert!(caps.get("signatureHelpProvider").is_some());
    assert!(caps.get("inlayHintProvider").is_some());
}

#[tokio::test]
async fn test_hover() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\ntype User {\n    string email = 1;\n    string name = 2;\n}\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/hover",
        2,
        json!({
            "textDocument": { "uri": "file:///test.ogham" },
            "position": { "line": 1, "character": 6 }
        }),
    )
    .await;

    let content = resp
        .pointer("/result/contents/value")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        content.contains("type") && content.contains("User"),
        "expected type User hover, got: {}",
        content
    );
}

#[tokio::test]
async fn test_goto_definition() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\ntype Address { string city = 1; }\ntype User { Address home = 1; }\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/definition",
        3,
        json!({
            "textDocument": { "uri": "file:///test.ogham" },
            "position": { "line": 2, "character": 12 }
        }),
    )
    .await;

    let result = resp.get("result");
    assert!(
        result.is_some() && !result.unwrap().is_null(),
        "expected definition: {:?}",
        resp
    );
}

#[tokio::test]
async fn test_completion_has_types_and_keywords() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\ntype User { string email = 1; }\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/completion",
        4,
        json!({
            "textDocument": { "uri": "file:///test.ogham" },
            "position": { "line": 2, "character": 0 }
        }),
    )
    .await;

    let items = resp
        .pointer("/result")
        .and_then(|r| r.as_array())
        .expect("expected completion items");

    let labels: Vec<&str> = items
        .iter()
        .filter_map(|i| i.get("label")?.as_str())
        .collect();

    assert!(labels.contains(&"type"), "missing keyword 'type': {:?}", &labels[..5.min(labels.len())]);
    assert!(labels.contains(&"string"), "missing builtin 'string'");
}

#[tokio::test]
async fn test_document_symbols() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\ntype User { string email = 1; }\nenum Status { Active = 1; }\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/documentSymbol",
        5,
        json!({ "textDocument": { "uri": "file:///test.ogham" } }),
    )
    .await;

    let symbols = resp
        .pointer("/result")
        .and_then(|r| r.as_array())
        .expect("expected symbols");

    let names: Vec<&str> = symbols
        .iter()
        .filter_map(|s| s.get("name")?.as_str())
        .collect();
    assert!(names.contains(&"User"), "missing User: {:?}", names);
    assert!(names.contains(&"Status"), "missing Status: {:?}", names);
}

#[tokio::test]
async fn test_find_references() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\ntype Address { string city = 1; }\ntype User { Address home = 1; }\ntype Order { Address billing = 1; }\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/references",
        6,
        json!({
            "textDocument": { "uri": "file:///test.ogham" },
            "position": { "line": 1, "character": 6 },
            "context": { "includeDeclaration": true }
        }),
    )
    .await;

    let refs = resp
        .pointer("/result")
        .and_then(|r| r.as_array())
        .expect("expected references");
    assert!(refs.len() >= 3, "expected >= 3 refs for Address, got {}", refs.len());
}

#[tokio::test]
async fn test_semantic_tokens() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\ntype User { string email = 1; }\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/semanticTokens/full",
        7,
        json!({ "textDocument": { "uri": "file:///test.ogham" } }),
    )
    .await;

    let data = resp
        .pointer("/result/data")
        .and_then(|d| d.as_array())
        .expect("expected token data");
    assert!(!data.is_empty(), "expected non-empty semantic tokens");
}

#[tokio::test]
async fn test_rename() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\ntype User { string email = 1; }\ntype Order { User owner = 1; }\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/rename",
        8,
        json!({
            "textDocument": { "uri": "file:///test.ogham" },
            "position": { "line": 1, "character": 6 },
            "newName": "Customer"
        }),
    )
    .await;

    let changes = resp.pointer("/result/changes");
    assert!(changes.is_some(), "expected rename changes: {:?}", resp);
}

#[tokio::test]
async fn test_code_action_add_package() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "type User { string email = 1; }\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/codeAction",
        9,
        json!({
            "textDocument": { "uri": "file:///test.ogham" },
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
            "context": { "diagnostics": [] }
        }),
    )
    .await;

    if let Some(actions) = resp.pointer("/result").and_then(|r| r.as_array()) {
        let titles: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.get("title")?.as_str())
            .collect();
        assert!(
            titles.iter().any(|t| t.contains("package")),
            "expected 'add package' action: {:?}",
            titles
        );
    }
}

#[tokio::test]
async fn test_std_types_in_completion() {
    let mut service = build();
    init_and_open(
        &mut service,
        "file:///test.ogham",
        "package test;\n",
    )
    .await;

    let resp = request(
        &mut service,
        "textDocument/completion",
        10,
        json!({
            "textDocument": { "uri": "file:///test.ogham" },
            "position": { "line": 1, "character": 0 }
        }),
    )
    .await;

    let items = resp
        .pointer("/result")
        .and_then(|r| r.as_array())
        .expect("expected items");

    let labels: Vec<&str> = items
        .iter()
        .filter_map(|i| i.get("label")?.as_str())
        .collect();

    // UUID should come from std index
    assert!(labels.contains(&"UUID"), "expected UUID from std: {:?}", &labels[..10.min(labels.len())]);
}
