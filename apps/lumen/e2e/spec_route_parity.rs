//! Router ↔ spec parity gate (#2482 regression class).
//!
//! `apps/lumen/src/api.rs`'s axum router (`router_with_admission`'s
//! `.route(...)` literals) and its utoipa-derived `ApiDoc` `paths(...)` list
//! are two hand-maintained registrations — nothing but code review stops a
//! route from being served without ever being spec-documented, or the two
//! copies of a path string drifting apart. This test extracts every
//! `.route("<path>", <methods>)` literal from `api.rs` (balanced-paren scan
//! + method tokens + ident-prefix rejection, mirroring
//! `apps/tape/tests/spec_route_parity.rs`) and diffs the (method, path) set
//! against `lumen::spec::openapi_json()`'s `paths` object in both
//! directions.
//!
//! Three documented asymmetries are excluded from the diff rather than
//! producing false positives:
//! - `/healthz`, `/readyz`, `/metrics`, `/version`, and `/debug/cluster` are
//!   merged from the shared `service_http::standard_probe_routes_canonical_json`
//!   shell and a small `admin` sub-router, not `.route()` literals inside
//!   the `data_plane` router built by this extractor, but they are still
//!   real documented operations in `ApiDoc::paths(...)`, so they are
//!   injected into the router-side set for comparison.
//! - `/openapi.json` and `/docs` are also merged from the shared probe
//!   shell, but they are the self-describing meta endpoints that serve and
//!   render the OpenAPI document itself — documenting "fetch the spec"
//!   inside the spec is circular, so `ApiDoc::paths(...)` intentionally
//!   omits them and this extractor does not inject them either.
//! - The two `QUERY` twins (`QUERY /collections`, `QUERY
//!   /collections/{collection_id}`, epic #1296 R1) are injected into the
//!   spec by `crate::api::inject_query_twins` as a documented POST-twin;
//!   axum has no native `Method::QUERY` combinator yet
//!   (tokio-rs/axum#3799), so there is no `.route()` literal to match
//!   against — the extractor's method whitelist below simply never emits a
//!   `QUERY` entry, so no explicit exclusion is needed on the router side.
//!
//! Its first run caught real drift (fixed alongside this gate, same
//! commit): `backup`, `backup_to_local`, `restore`, and `reindex_stream`
//! were served admin/index routes with no `#[utoipa::path]` annotation at
//! all, so they were silently absent from `lumen spec --format openapi`.

use std::collections::BTreeSet;

const API_RS: &str = include_str!("../src/api.rs");

/// Extract `(METHOD, path)` pairs from every `.route(` invocation in
/// `api.rs`. Handles multi-line calls by balanced-paren scanning and reads
/// method-router builder names (`get(`/`post(`/`put(`/`delete(`/`patch(`)
/// inside the call's argument text. `.options(`/`.head(`/`.fallback(` are
/// deliberately not scanned for: those combinators exist only for the
/// #1296 QUERY-method-emulation dispatch on `/collections` and
/// `/collections/{collection_id}`, not a second real HTTP verb.
fn router_routes_from_source() -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let mut rest = API_RS;
    while let Some(idx) = rest.find(".route(") {
        rest = &rest[idx + ".route(".len()..];
        // Balanced-paren scan for the call's argument text. `api.rs` route
        // args carry no string literals containing parens, so a plain
        // depth counter is sufficient here.
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
    // The one-port operational surface (probes + `/version` +
    // `/debug/cluster`) is merged from `service_http::standard_probe_routes*`
    // and the small `admin` sub-router, not `data_plane` `.route()`
    // literals. `/openapi.json` and `/docs` are excluded here too — see the
    // module docs on why those two never appear in the spec's own paths.
    for probe in [
        "/healthz",
        "/readyz",
        "/metrics",
        "/version",
        "/debug/cluster",
    ] {
        out.insert(("GET".to_string(), probe.to_string()));
    }
    out
}

/// The (method, path) set lumen's offline OpenAPI document actually
/// publishes. Only the five real HTTP methods this codebase routes on are
/// counted — this whitelist is what keeps the injected `QUERY` twin (see
/// module docs) out of the comparison without an explicit exclusion list.
fn spec_routes() -> BTreeSet<(String, String)> {
    let raw = lumen::spec::openapi_json();
    let value: serde_json::Value = serde_json::from_str(&raw).expect("openapi_json parses");
    let mut out = BTreeSet::new();
    for (path, item) in value["paths"].as_object().expect("openapi has paths") {
        let item = item.as_object().expect("path item is an object");
        for method in ["get", "post", "put", "delete", "patch"] {
            if item.contains_key(method) {
                out.insert((method.to_ascii_uppercase(), path.clone()));
            }
        }
    }
    out
}

/// #2482: every served data-plane/probe route is published in the spec
/// inventory, and the spec never lists a route the router does not serve.
#[test]
fn served_routes_and_spec_inventory_match_exactly() {
    let router = router_routes_from_source();
    let spec = spec_routes();

    let unpublished: Vec<_> = router.difference(&spec).collect();
    let phantom: Vec<_> = spec.difference(&router).collect();

    assert!(
        unpublished.is_empty() && phantom.is_empty(),
        "router\u{2194}spec drift.\nserved but missing from spec (add a #[utoipa::path] \
         annotation and register it in ApiDoc's paths(...) \u{2014} the #2482 class): \
         {unpublished:?}\nin spec but not served (phantom documentation): {phantom:?}"
    );
}

/// The extractor itself must keep seeing the full router surface: if a
/// refactor moves route registration out of `api.rs` string literals, this
/// floor forces the parity gate to be updated rather than silently
/// comparing an empty set.
#[test]
fn source_extractor_sees_the_data_plane() {
    let router = router_routes_from_source();
    assert!(
        router.len() >= 20,
        "route extractor found only {} routes — registration style changed; update the parity gate",
        router.len()
    );
}
