// SPEC-MANAGED: .score/tech_design/projects/httpkit/request-response.md#tests
// CODEGEN-BEGIN

use mambalibs_http::request_response::{Cookie, Request, Response};
use std::collections::HashMap;

#[test]
fn cookie_with_secure_http_only_defaults_to_false() {
    let c = Cookie::new(
        "session".to_string(),
        "abc123".to_string(),
        None,
        None,
        false,
        false,
        None,
    )
    .unwrap();
    assert_eq!(c.name, "session");
    assert_eq!(c.value, "abc123");
    assert!(!c.secure);
    assert!(!c.http_only);
    assert!(c.max_age.is_none());
}

#[test]
fn cookie_secure_flag_roundtrips() {
    let c = Cookie::new(
        "sid".to_string(),
        "xyz".to_string(),
        Some("/".to_string()),
        Some("example.com".to_string()),
        true,
        true,
        Some(3600),
    )
    .unwrap();
    assert!(c.secure);
    assert!(c.http_only);
    assert_eq!(c.max_age, Some(3600));
    assert_eq!(c.path.as_deref(), Some("/"));
}

#[test]
fn request_defaults_have_empty_collections() {
    let req = Request::new(
        "GET".to_string(),
        "/health".to_string(),
        HashMap::new(),
        HashMap::new(),
        Vec::new(),
        Vec::new(),
        HashMap::new(),
    )
    .unwrap();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/health");
    assert!(req.query_params.as_ref().map_or(true, |m| m.is_empty()));
    assert!(req.body.as_ref().map_or(true, |b| b.is_empty()));
}

#[test]
fn response_rejects_out_of_range_status() {
    let result = Response::new(
        700,
        Vec::new(),
        HashMap::new(),
        Vec::new(),
        "application/json".to_string(),
    );
    assert!(result.is_err());
}

#[test]
fn response_accepts_default_json_media_type() {
    let resp = Response::new(
        200,
        br#"{"ok":true}"#.to_vec(),
        HashMap::new(),
        Vec::new(),
        "application/json".to_string(),
    )
    .unwrap();
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.media_type, "application/json");
    assert!(resp.body.as_ref().is_some_and(|b| !b.is_empty()));
}
// CODEGEN-END
