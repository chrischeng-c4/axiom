// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-router-contract-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::collections::BTreeMap;

use preview::{
    load_route_table_from_rendered_dir, render_files, resolve_route, resolve_route_with_base,
    BaseRoute, RenderInput, RouteBinding, RouteOutcome, RouteRequest,
};

fn bindings() -> BTreeMap<String, RouteBinding> {
    BTreeMap::from([(
        "mr-123".to_string(),
        RouteBinding {
            target: "mr-123".to_string(),
            host: "uat.example.com".to_string(),
            cookie: "uat_target".to_string(),
            header: "X-UAT-Target".to_string(),
            namespace: "uat-mr-123".to_string(),
            service: "checkout".to_string(),
            service_port: 80,
            sha: "abc123".to_string(),
        },
    )])
}

fn base() -> BaseRoute {
    BaseRoute {
        host: "uat.example.com".to_string(),
        namespace: "uat-base".to_string(),
        service: "checkout".to_string(),
        service_port: 80,
    }
}

fn render_input() -> RenderInput {
    RenderInput {
        mr: 123,
        sha: "abc123".to_string(),
        image: "registry.local/checkout:abc123".to_string(),
        app: "checkout".to_string(),
        host: "uat.example.com".to_string(),
        base_namespace: "uat-base".to_string(),
        owner: "payments-sre".to_string(),
        ttl_hours: 48,
        control_namespace: "preview-system".to_string(),
        workload_identity: "preview-runner".to_string(),
        base_contract: None,
    }
}

#[test]
fn cookie_target_resolves_to_route_binding() {
    let request = RouteRequest {
        host: "uat.example.com".to_string(),
        cookies: BTreeMap::from([("uat_target".to_string(), "mr-123".to_string())]),
        headers: BTreeMap::new(),
    };

    let route = resolve_route(&bindings(), &request).expect("route");

    assert_eq!(route.target, "mr-123");
    assert_eq!(route.namespace, "uat-mr-123");
    assert_eq!(route.service, "checkout");
}

#[test]
fn no_target_uses_base_route() {
    let request = RouteRequest {
        host: "uat.example.com".to_string(),
        cookies: BTreeMap::new(),
        headers: BTreeMap::new(),
    };

    let decision = resolve_route_with_base(&bindings(), &base(), &request);

    assert_eq!(decision.outcome, RouteOutcome::Base);
    assert_eq!(decision.target, None);
    assert_eq!(decision.namespace.as_deref(), Some("uat-base"));
    assert_eq!(decision.service.as_deref(), Some("checkout"));
    assert!(decision.reason.contains("base route"));
}

#[test]
fn header_target_overrides_cookie_target() {
    let mut bindings = bindings();
    bindings.insert(
        "mr-456".to_string(),
        RouteBinding {
            target: "mr-456".to_string(),
            host: "uat.example.com".to_string(),
            cookie: "uat_target".to_string(),
            header: "X-UAT-Target".to_string(),
            namespace: "uat-mr-456".to_string(),
            service: "checkout".to_string(),
            service_port: 80,
            sha: "def456".to_string(),
        },
    );
    let request = RouteRequest {
        host: "uat.example.com".to_string(),
        cookies: BTreeMap::from([("uat_target".to_string(), "mr-123".to_string())]),
        headers: BTreeMap::from([("X-UAT-Target".to_string(), "mr-456".to_string())]),
    };

    let route = resolve_route(&bindings, &request).expect("route");

    assert_eq!(route.target, "mr-456");
    assert_eq!(route.namespace, "uat-mr-456");
}

#[test]
fn unknown_target_does_not_guess_namespace() {
    let request = RouteRequest {
        host: "uat.example.com".to_string(),
        cookies: BTreeMap::from([("uat_target".to_string(), "uat-mr-999".to_string())]),
        headers: BTreeMap::new(),
    };

    assert!(resolve_route(&bindings(), &request).is_none());

    let decision = resolve_route_with_base(&bindings(), &base(), &request);
    assert_eq!(decision.outcome, RouteOutcome::NotFound);
    assert_eq!(decision.target.as_deref(), Some("uat-mr-999"));
    assert_eq!(decision.namespace, None);
    assert!(decision.reason.contains("unknown route target"));
}

#[test]
fn host_mismatch_fails_closed_instead_of_falling_back_to_base() {
    let request = RouteRequest {
        host: "other.example.com".to_string(),
        cookies: BTreeMap::from([("uat_target".to_string(), "mr-123".to_string())]),
        headers: BTreeMap::new(),
    };

    let decision = resolve_route_with_base(&bindings(), &base(), &request);

    assert_eq!(decision.outcome, RouteOutcome::NotFound);
    assert_eq!(decision.namespace, None);
    assert!(decision.reason.contains("host does not match"));
}

#[test]
fn rendered_route_binding_file_loads_adapter_route_table() {
    let dir = tempfile::tempdir().expect("tempdir");
    for file in render_files(&render_input()).expect("render files") {
        if file.path != "router/route-binding.yaml" {
            continue;
        }
        let path = dir.path().join(&file.path);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        std::fs::write(path, file.contents).expect("write route binding");
    }

    let loaded = load_route_table_from_rendered_dir(dir.path()).expect("load route table");
    let request = RouteRequest {
        host: "uat.example.com".to_string(),
        cookies: BTreeMap::new(),
        headers: BTreeMap::from([("X-UAT-Target".to_string(), "mr-123".to_string())]),
    };
    let decision = resolve_route_with_base(&loaded, &base(), &request);

    assert_eq!(decision.outcome, RouteOutcome::Preview);
    assert_eq!(decision.namespace.as_deref(), Some("uat-mr-123"));
    assert_eq!(decision.service.as_deref(), Some("checkout"));
    assert_eq!(decision.reason, "matched X-UAT-Target header");
}

// </HANDWRITE>
