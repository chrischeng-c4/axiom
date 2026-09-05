//! External structure contract for the shared MCP transport shell.

#[test]
fn sift_keeps_tools_while_service_mcp_owns_transports_and_security() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Sift manifest");
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mcp.rs"))
        .expect("read Sift MCP adapter");

    assert!(manifest.contains("service-mcp ="));
    assert!(source.contains("impl service_mcp::McpApplication for SiftMcpServer"));
    assert!(source.contains("service_mcp::serve_stdio"));
    assert!(source.contains("service_mcp::streamable_http_router"));
    for local_mechanism in [
        "StreamableHttpService::new",
        "LocalSessionManager::default",
        ".serve(rmcp::transport::stdio())",
        "fn csv_env(",
        "service_auth::bearer_token",
    ] {
        assert!(
            !source.contains(local_mechanism),
            "Sift must not retain MCP transport mechanism {local_mechanism}"
        );
    }
}
