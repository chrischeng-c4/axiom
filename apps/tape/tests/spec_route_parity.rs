//! Router ↔ spec parity gate (#2482 regression class, interim for #2495).
//!
//! The axum router and the spec route inventory are two hand-maintained
//! registrations; deriving OpenAPI from annotations guarantees spec ≡
//! registry, not registry ≡ router — `GET /topics/{topic}/retention` was
//! served but missing from every spec surface until #2482. Until #2495
//! makes the declaration single-source, this test extracts every
//! `.route("<path>", <methods>)` literal from `server.rs` and diffs the
//! (method, path) set against `spec::routes_json()` in BOTH directions.
//! The five standard probe routes come from the shared `service-http`
//! shell rather than `server.rs` literals and are injected explicitly.

use std::collections::BTreeSet;

const SERVER_RS: &str = include_str!("../src/server.rs");

/// Extract `(METHOD, path)` pairs from every `.route(` invocation in
/// `server.rs`. Handles multi-line calls by balanced-paren scanning and
/// reads method-router builder names (`get(`/`post(`/`put(`/`delete(`/
/// `patch(`) inside the call's argument text.
fn router_routes_from_source() -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let mut rest = SERVER_RS;
    while let Some(idx) = rest.find(".route(") {
        rest = &rest[idx + ".route(".len()..];
        // Balanced-paren scan for the call's argument text. `server.rs`
        // route args carry no string literals containing parens, so a
        // plain depth counter is sufficient here.
        let mut depth = 1usize;
        let mut end = 0usize;
        for (i, ch) in rest.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let args = &rest[..end];
        let Some(path_start) = args.find('"') else {
            continue;
        };
        let path_rest = &args[path_start + 1..];
        let Some(path_end) = path_rest.find('"') else {
            continue;
        };
        let path = &path_rest[..path_end];
        let methods_text = &path_rest[path_end + 1..];
        for method in ["get", "post", "put", "delete", "patch"] {
            let mut scan = methods_text;
            let needle = format!("{method}(");
            while let Some(pos) = scan.find(&needle) {
                // Reject identifier-suffix matches such as `checkpoint_get(`:
                // a real axum method router token is not preceded by an
                // identifier character.
                let preceded_by_ident = pos > 0
                    && scan[..pos]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
                if !preceded_by_ident {
                    out.insert((method.to_ascii_uppercase(), path.to_string()));
                }
                scan = &scan[pos + needle.len()..];
            }
        }
        rest = &rest[end..];
    }
    // The one-port operational surface is merged from the shared
    // `service_http::standard_probe_routes` shell, not `.route()` literals.
    for probe in ["/healthz", "/readyz", "/metrics", "/openapi.json", "/docs"] {
        out.insert(("GET".to_string(), probe.to_string()));
    }
    out
}

fn spec_routes() -> BTreeSet<(String, String)> {
    let raw = tape::spec::routes_json();
    let value: serde_json::Value = serde_json::from_str(&raw).expect("routes_json parses");
    value["routes"]
        .as_array()
        .expect("routes array")
        .iter()
        .map(|route| {
            (
                route["method"].as_str().expect("method").to_string(),
                route["path"].as_str().expect("path").to_string(),
            )
        })
        .collect()
}

/// R1 (#2482): every served data-plane/probe route is published in the spec
/// inventory, and the spec never lists a route the router does not serve.
#[test]
fn served_routes_and_spec_inventory_match_exactly() {
    let router = router_routes_from_source();
    let spec = spec_routes();

    let unpublished: Vec<_> = router.difference(&spec).collect();
    let phantom: Vec<_> = spec.difference(&router).collect();

    assert!(
        unpublished.is_empty() && phantom.is_empty(),
        "router↔spec drift.\nserved but missing from spec (add to routes_json/openapi — the #2482 class): {unpublished:?}\nin spec but not served (phantom documentation): {phantom:?}"
    );
}

/// The extractor itself must keep seeing the full router surface: if a
/// refactor moves route registration out of `server.rs` string literals,
/// this floor forces the parity gate to be updated rather than silently
/// comparing an empty set.
#[test]
fn source_extractor_sees_the_data_plane() {
    let router = router_routes_from_source();
    assert!(
        router.len() >= 15,
        "route extractor found only {} routes — registration style changed; update the parity gate",
        router.len()
    );
}
