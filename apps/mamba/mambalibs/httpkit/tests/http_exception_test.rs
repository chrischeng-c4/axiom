// SPEC-MANAGED: .score/tech_design/projects/httpkit/http-exception.md#tests
// CODEGEN-BEGIN

use mambalibs_http::http_exception::HTTPException;
use std::collections::HashMap;

#[test]
fn preserves_explicit_detail() {
    let exc = HTTPException::new(404, Some("not in cache".into()), HashMap::new()).unwrap();
    assert_eq!(exc.status_code, 404);
    assert_eq!(exc.detail, "not in cache");
}

#[test]
fn omitted_detail_fills_from_status_phrase() {
    let exc = HTTPException::new(500, None, HashMap::new()).unwrap();
    assert_eq!(exc.status_code, 500);
    assert_eq!(exc.detail, "Internal Server Error");
}

#[test]
fn status_code_out_of_range_low_returns_err() {
    let result = HTTPException::new(50, None, HashMap::new());
    assert!(result.is_err());
}

#[test]
fn status_code_out_of_range_high_returns_err() {
    let result = HTTPException::new(700, None, HashMap::new());
    assert!(result.is_err());
}

#[test]
fn headers_round_trip_when_provided() {
    let mut h = HashMap::new();
    h.insert("x-retry-after".to_string(), "30".to_string());
    let exc = HTTPException::new(429, None, h).unwrap();
    assert_eq!(
        exc.headers
            .as_ref()
            .unwrap()
            .get("x-retry-after")
            .map(|s| s.as_str()),
        Some("30")
    );
}
// CODEGEN-END
