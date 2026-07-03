// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-router-contract-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::collections::BTreeMap;

use preview::{resolve_route, RouteBinding, RouteRequest};

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
}

// </HANDWRITE>
