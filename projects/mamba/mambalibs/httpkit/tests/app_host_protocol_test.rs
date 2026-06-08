use mambalibs_http::app::{App, BackgroundTask, BackgroundTasks, Endpoint, RouteParameter, Router};
use mambalibs_http::host::{HostCapabilities, HostConfig};
use mambalibs_http::protocol::{BodyMode, HttpProtocol, NativeRequestHead, NativeResponseHead};
use std::collections::HashMap;

#[test]
fn app_is_native_route_registry_not_asgi_callable() {
    let mut app = App::new(HashMap::from([("title".to_string(), "Cue".to_string())]));

    app.add_route("GET", "health");

    assert_eq!(app.metadata.get("title").map(String::as_str), Some("Cue"));
    assert_eq!(app.route_count(), 1);
    assert_eq!(app.router.full_path(&app.router.routes[0]), "/health");
}

#[test]
fn router_joins_prefix_and_route_without_asgi_state() {
    let mut router = Router::new("/api/", vec!["system".to_string()]);

    router.add_route("POST", "workitems");

    assert_eq!(router.full_path(&router.routes[0]), "/api/workitems");
}

#[test]
fn app_endpoint_registration_preserves_simple_routes() {
    let mut app = App::new(HashMap::from([("title".to_string(), "Cue".to_string())]));

    app.add_route("GET", "/health");
    app.add_endpoint(
        Endpoint::new("post", "items")
            .with_handler_name("create_item")
            .with_dependency_key("current_user")
            .with_request_model("ItemCreate")
            .with_response_model("ItemRead")
            .with_status_code(201),
    );

    assert_eq!(app.route_count(), 2);
    assert_eq!(app.endpoint_count(), 1);
    assert_eq!(app.router.full_path(&app.router.routes[0]), "/health");
    assert_eq!(app.router.full_path(&app.router.routes[1]), "/items");

    let endpoint = &app.endpoints()[0];
    assert_eq!(endpoint.method, "POST");
    assert_eq!(endpoint.path, "/items");
    assert_eq!(endpoint.handler_name.as_deref(), Some("create_item"));
    assert_eq!(endpoint.dependency_keys, vec!["current_user".to_string()]);
    assert_eq!(endpoint.request_model.as_deref(), Some("ItemCreate"));
    assert_eq!(endpoint.response_model.as_deref(), Some("ItemRead"));
    assert_eq!(endpoint.status_code, 201);
}

#[test]
fn app_openapi_json_exports_route_dependency_and_model_refs() {
    let mut app = App::new(HashMap::from([
        ("title".to_string(), "Inventory".to_string()),
        ("version".to_string(), "1.2.3".to_string()),
    ]));

    app.add_endpoint(
        Endpoint::new("post", "/items")
            .with_handler_name("create_item")
            .with_dependency_key("current_user")
            .with_request_model("ItemCreate")
            .with_request_schema_json(
                r#"{"title":"ItemCreate","type":"object","properties":{"name":{"type":"string","minLength":3}},"required":["name"]}"#,
            )
            .with_response_model("ItemRead")
            .with_response_schema_json(
                r#"{"title":"ItemRead","type":"object","properties":{"name":{"type":"string"}},"required":["name"]}"#,
            )
            .with_status_code(201),
    );

    let doc: serde_json::Value =
        serde_json::from_str(&app.openapi_json()).expect("openapi json should parse");
    assert_eq!(doc["openapi"].as_str(), Some("3.1.0"));
    assert_eq!(doc["info"]["title"].as_str(), Some("Inventory"));
    assert_eq!(
        doc["paths"]["/items"]["post"]["responses"]["201"]["content"]["application/json"]["schema"]
            ["$ref"]
            .as_str(),
        Some("#/components/schemas/ItemRead")
    );
    assert_eq!(
        doc["paths"]["/items"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"]
            .as_str(),
        Some("#/components/schemas/ItemCreate")
    );
    assert_eq!(
        doc["paths"]["/items"]["post"]["x-mamba-dependencies"][0].as_str(),
        Some("current_user")
    );
    assert_eq!(
        doc["components"]["schemas"]["ItemCreate"]["title"].as_str(),
        Some("ItemCreate")
    );
    assert_eq!(
        doc["components"]["schemas"]["ItemCreate"]["properties"]["name"]["minLength"].as_i64(),
        Some(3)
    );
    assert_eq!(
        doc["components"]["schemas"]["ItemRead"]["title"].as_str(),
        Some("ItemRead")
    );
}

#[test]
fn app_preflight_json_validates_body_and_reports_dependencies() {
    let mut app = App::new(HashMap::from([(
        "title".to_string(),
        "Inventory".to_string(),
    )]));
    app.add_endpoint(
        Endpoint::new("post", "/items")
            .with_dependency_key("current_user")
            .with_request_model("ItemCreate")
            .with_request_schema_json(
                r#"{"title":"ItemCreate","type":"object","properties":{"name":{"type":"string","minLength":3},"age":{"type":"integer","minimum":1},"tags":{"type":"array","items":{"type":"string"}}},"required":["age","name"]}"#,
            )
            .with_response_model("ItemRead")
            .with_status_code(201),
    );

    let ok: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
        "POST",
        "/items",
        serde_json::json!({"name":"alice","age":2,"tags":["api"]}),
        HashMap::from([("current_user".to_string(), "alice".to_string())]),
    ))
    .expect("valid preflight report should parse");
    assert_eq!(ok["matched"].as_bool(), Some(true));
    assert_eq!(ok["status_code"].as_i64(), Some(201));
    assert_eq!(ok["body"]["age"].as_i64(), Some(2));
    assert_eq!(ok["dependencies"]["current_user"].as_str(), Some("alice"));

    let invalid: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
        "POST",
        "/items",
        serde_json::json!({"name":"al","age":"two","tags":[1]}),
        HashMap::from([("current_user".to_string(), "alice".to_string())]),
    ))
    .expect("invalid preflight report should parse");
    assert_eq!(invalid["status_code"].as_i64(), Some(422));
    assert!(invalid["errors"].as_array().is_some_and(|errors| errors
        .iter()
        .any(|error| error.as_str().is_some_and(|msg| msg.contains("$.age")))));

    let missing_dependency: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
        "POST",
        "/items",
        serde_json::json!({"name":"alice","age":2,"tags":["api"]}),
        HashMap::new(),
    ))
    .expect("dependency error report should parse");
    assert_eq!(missing_dependency["status_code"].as_i64(), Some(500));
    assert!(missing_dependency["dependency_errors"]
        .as_array()
        .is_some_and(|errors| errors.iter().any(|error| error
            .as_str()
            .is_some_and(|msg| msg.contains("current_user")))));

    let missing_route: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
        "GET",
        "/missing",
        serde_json::json!({}),
        HashMap::new(),
    ))
    .expect("missing route report should parse");
    assert_eq!(missing_route["matched"].as_bool(), Some(false));
    assert_eq!(missing_route["status_code"].as_i64(), Some(404));
    assert_eq!(missing_route["body"], serde_json::json!({}));
}

#[test]
fn app_preflight_json_validates_nullable_any_of_body_schema() {
    let mut app = App::new(HashMap::from([(
        "title".to_string(),
        "Inventory".to_string(),
    )]));
    app.add_endpoint(
        Endpoint::new("post", "/profiles")
            .with_request_model("Profile")
            .with_request_schema_json(
                r#"{"title":"Profile","type":"object","properties":{"name":{"type":"string"},"nickname":{"anyOf":[{"type":"null"},{"type":"string","minLength":2}]}},"required":["name"]}"#,
            )
            .with_status_code(201),
    );

    for body in [
        serde_json::json!({"name":"alice","nickname":null}),
        serde_json::json!({"name":"alice","nickname":"ally"}),
        serde_json::json!({"name":"alice"}),
    ] {
        let ok: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
            "POST",
            "/profiles",
            body,
            HashMap::new(),
        ))
        .expect("nullable anyOf preflight report should parse");
        assert_eq!(ok["matched"].as_bool(), Some(true));
        assert_eq!(ok["status_code"].as_i64(), Some(201));
        assert_eq!(ok["errors"], serde_json::json!([]));
    }

    let invalid: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
        "POST",
        "/profiles",
        serde_json::json!({"name":"alice","nickname":7}),
        HashMap::new(),
    ))
    .expect("invalid nullable anyOf preflight report should parse");
    assert_eq!(invalid["status_code"].as_i64(), Some(422));
    assert!(invalid["errors"]
        .as_array()
        .is_some_and(|errors| errors.iter().any(|error| error
            .as_str()
            .is_some_and(|msg| { msg.contains("$.nickname") && msg.contains("anyOf") }))));
}

#[test]
fn app_preflight_json_validates_pydantic_style_schema_constraints() {
    let mut app = App::new(HashMap::from([(
        "title".to_string(),
        "Inventory".to_string(),
    )]));
    app.add_endpoint(
        Endpoint::new("post", "/inventory")
            .with_request_model("InventoryItem")
            .with_request_schema_json(
                r#"{"title":"InventoryItem","type":"object","properties":{"sku":{"type":"string","pattern":"^[A-Z]{3}$"},"count":{"type":"integer","exclusiveMinimum":0},"batch":{"type":"integer","multipleOf":5},"score":{"type":"number","exclusiveMaximum":1.0},"tags":{"type":"array","uniqueItems":true,"items":{"type":"string"}}},"required":["batch","count","score","sku","tags"]}"#,
            )
            .with_status_code(201),
    );

    let ok: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
        "POST",
        "/inventory",
        serde_json::json!({"sku":"ABC","count":1,"batch":10,"score":0.5,"tags":["new","sale"]}),
        HashMap::new(),
    ))
    .expect("valid constraint preflight report should parse");
    assert_eq!(ok["matched"].as_bool(), Some(true));
    assert_eq!(ok["status_code"].as_i64(), Some(201));
    assert_eq!(ok["errors"], serde_json::json!([]));

    let invalid: serde_json::Value = serde_json::from_str(&app.preflight_request_json(
        "POST",
        "/inventory",
        serde_json::json!({"sku":"ab1","count":0,"batch":6,"score":1.0,"tags":["new","new"]}),
        HashMap::new(),
    ))
    .expect("invalid constraint preflight report should parse");
    assert_eq!(invalid["status_code"].as_i64(), Some(422));
    let messages = invalid["errors"]
        .as_array()
        .expect("preflight errors")
        .iter()
        .filter_map(|error| error.as_str())
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|msg| msg.contains("$.sku") && msg.contains("pattern")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|msg| msg.contains("$.count") && msg.contains("expected > 0")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|msg| msg.contains("$.batch") && msg.contains("multiple of 5")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|msg| msg.contains("$.score") && msg.contains("expected < 1")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|msg| msg.contains("$.tags") && msg.contains("unique items")),
        "{messages:?}"
    );
}

#[test]
fn background_tasks_record_queue_handoff_metadata() {
    let mut tasks = BackgroundTasks::new();

    assert!(tasks.is_empty());
    assert!(tasks.add_task(
        BackgroundTask::new("send_email")
            .payload_json(r#"{"user_id":42}"#)
            .queue("email")
    ));
    assert!(tasks.add_named_task(
        "audit.write",
        Some(r#"{"event":"created"}"#.to_string()),
        Some("audit".to_string())
    ));

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks.task_count, 2);
    let value = tasks.tasks_value();
    assert_eq!(value[0]["name"].as_str(), Some("send_email"));
    assert_eq!(value[0]["payload"]["user_id"].as_i64(), Some(42));
    assert_eq!(value[0]["queue"].as_str(), Some("email"));
    assert_eq!(value[1]["name"].as_str(), Some("audit.write"));
    assert_eq!(value[1]["payload"]["event"].as_str(), Some("created"));
    assert_eq!(value[1]["queue"].as_str(), Some("audit"));

    let drained = tasks.drain();
    assert_eq!(drained.len(), 2);
    assert!(tasks.is_empty());
    assert_eq!(tasks.task_count, 0);
}

#[test]
fn app_openapi_and_preflight_handle_query_header_parameters() {
    let mut app = App::new(HashMap::from([("title".to_string(), "Search".to_string())]));
    app.add_endpoint(
        Endpoint::new("get", "/search")
            .with_parameter(
                RouteParameter::query("q")
                    .description("Search term")
                    .schema_json(r#"{"type":"string","minLength":1}"#),
            )
            .with_parameter(
                RouteParameter::header("X-Trace-ID")
                    .optional()
                    .default_json(r#""local""#)
                    .schema_json(r#"{"type":"string"}"#),
            ),
    );

    let doc: serde_json::Value =
        serde_json::from_str(&app.openapi_json()).expect("openapi json should parse");
    let parameters = doc["paths"]["/search"]["get"]["parameters"]
        .as_array()
        .expect("parameters");
    assert!(parameters.iter().any(|param| {
        param["name"].as_str() == Some("q")
            && param["in"].as_str() == Some("query")
            && param["required"].as_bool() == Some(true)
            && param["description"].as_str() == Some("Search term")
            && param["schema"]["type"].as_str() == Some("string")
    }));
    assert!(parameters.iter().any(|param| {
        param["name"].as_str() == Some("X-Trace-ID")
            && param["in"].as_str() == Some("header")
            && param["required"].as_bool() == Some(false)
            && param["schema"]["default"].as_str() == Some("local")
    }));

    let ok: serde_json::Value = serde_json::from_str(&app.preflight_request_json_with_context(
        "GET",
        "/search",
        serde_json::json!({}),
        HashMap::new(),
        HashMap::from([("q".to_string(), "mamba".to_string())]),
        HashMap::new(),
    ))
    .expect("preflight report should parse");
    assert_eq!(ok["status_code"].as_i64(), Some(200));
    assert_eq!(ok["parameters"]["q"].as_str(), Some("mamba"));
    assert_eq!(ok["parameters"]["X-Trace-ID"].as_str(), Some("local"));

    let missing: serde_json::Value =
        serde_json::from_str(&app.preflight_request_json_with_context(
            "GET",
            "/search",
            serde_json::json!({}),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        ))
        .expect("missing parameter report should parse");
    assert_eq!(missing["status_code"].as_i64(), Some(422));
    assert!(missing["detail"].as_array().is_some_and(|details| details
        .iter()
        .any(|detail| detail["loc"] == serde_json::json!(["query", "q"])
            && detail["type"].as_str() == Some("missing"))));
}

#[test]
fn router_endpoint_registration_joins_prefix() {
    let mut router = Router::new("/api", vec!["items".to_string()]);

    router.add_endpoint(Endpoint::new("GET", "/items").with_response_model("ItemRead"));

    assert_eq!(router.endpoint_count(), 1);
    assert_eq!(router.full_path(&router.routes[0]), "/api/items");
    assert_eq!(
        router.endpoints[0].response_model.as_deref(),
        Some("ItemRead")
    );
}

#[test]
fn host_defaults_enable_native_http1_and_http2() {
    let config = HostConfig::default();
    let capabilities = HostCapabilities::default();

    assert!(config.supports(HttpProtocol::Http1));
    assert!(config.supports(HttpProtocol::Http2));
    assert!(config.keep_alive.enabled);
    assert!(capabilities.native_dispatch);
    assert!(!capabilities.asgi_compatibility);
    assert!(capabilities.long_lived_connections);
}

#[test]
fn protocol_heads_normalize_wire_protocol_before_dispatch() {
    let request = NativeRequestHead::new(HttpProtocol::Http2, "GET", "/health");
    let mut response = NativeResponseHead::new(200);
    response.body_mode = BodyMode::Buffered;

    assert_eq!(request.protocol, HttpProtocol::Http2);
    assert_eq!(request.path, "/health");
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body_mode, BodyMode::Buffered);
}
