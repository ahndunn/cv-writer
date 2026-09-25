use base64::Engine;
use cv_writer_mcp::mcp::JsonRpcRequest;
use cv_writer_mcp::server::Server;
use serde_json::json;

#[tokio::test]
async fn test_mcp_initialize_and_tools_list() {
    let server = Server::new().expect("Failed to initialize server");

    // 1. Initialize
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: None,
    };
    let init_resp = server.handle_request(init_req).await.expect("Expected init response");
    assert_eq!(init_resp.id, Some(json!(1)));
    let result = init_resp.result.expect("Expected result");
    assert_eq!(result.get("serverInfo").unwrap().get("name").unwrap(), "cv-writer-mcp");

    // 2. List Tools
    let list_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: None,
    };
    let list_resp = server.handle_request(list_req).await.expect("Expected tools/list response");
    let tools = list_resp.result.unwrap().get("tools").unwrap().as_array().unwrap().clone();
    let tool_names: Vec<String> = tools
        .iter()
        .map(|t| t.get("name").unwrap().as_str().unwrap().to_string())
        .collect();

    assert!(tool_names.contains(&"get_cv_schema".to_string()));
    assert!(tool_names.contains(&"get_sample_profile".to_string()));
    assert!(tool_names.contains(&"get_template_info".to_string()));
    assert!(tool_names.contains(&"render_cv".to_string()));
}

#[tokio::test]
async fn test_mcp_tool_call_render_cv() {
    let server = Server::new().expect("Failed to initialize server");

    // Get sample profile first
    let sample_call = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(10)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "get_sample_profile",
            "arguments": {}
        })),
    };
    let sample_resp = server.handle_request(sample_call).await.unwrap();
    let sample_text = sample_resp.result.unwrap().get("content").unwrap()[0]
        .get("text")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let sample_profile: serde_json::Value = serde_json::from_str(&sample_text).unwrap();

    // Call render_cv with sample profile
    let render_call = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(11)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "render_cv",
            "arguments": {
                "profile": sample_profile
            }
        })),
    };
    let render_resp = server.handle_request(render_call).await.unwrap();
    let content = render_resp.result.unwrap().get("content").unwrap()[0]
        .get("text")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let render_payload: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(render_payload.get("status").unwrap(), "success");
    let b64 = render_payload.get("pdf_base64").unwrap().as_str().unwrap();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .expect("Base64 decode failed");
    assert_eq!(&decoded[0..5], b"%PDF-");
}
