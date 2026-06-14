// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for self-closing intrinsic elements.
//!
//! The transpiler's `emit_jsx_self_closing` path emits these with
//! `vec![]` children. We verify they survive layout and appear in
//! `elementTree` as intrinsic nodes with an empty children array,
//! correctly positioned as siblings (before + after a button).

#[path = "../common/mod.rs"]
mod common;

use common::JetTestApp;

fn intrinsic_with_id<'a>(v: &'a serde_json::Value, id: &str) -> Option<&'a serde_json::Value> {
    let kind = v.get("kind").and_then(|k| k.as_str())?;
    if kind == "intrinsic" {
        let this_id = v
            .get("props")
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str());
        if this_id == Some(id) {
            return Some(v);
        }
    }
    if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
        for child in children {
            if let Some(hit) = intrinsic_with_id(child, id) {
                return Some(hit);
            }
        }
    }
    None
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn self_closing_demo_renders_void_elements_as_leaf_intrinsics() {
    common::require_env();
    let app = JetTestApp::launch("self-closing-demo")
        .await
        .expect("launch");

    let tree = app.element_tree().await.expect("elementTree");

    // Both self-closing <img>s should be present as intrinsic leaves.
    let icon =
        intrinsic_with_id(&tree, "icon").unwrap_or_else(|| panic!("#icon not found; tree={tree}"));
    let after = intrinsic_with_id(&tree, "after")
        .unwrap_or_else(|| panic!("#after not found; tree={tree}"));

    for (name, node) in [("#icon", icon), ("#after", after)] {
        assert_eq!(
            node.get("tag").and_then(|v| v.as_str()),
            Some("img"),
            "{name} should be an img tag; got {node}",
        );
        let children = node
            .get("children")
            .and_then(|c| c.as_array())
            .unwrap_or_else(|| panic!("{name} missing children array; got {node}"));
        assert!(
            children.is_empty(),
            "{name} is self-closing — no children allowed; got {children:?}",
        );
    }

    // Sanity: the button between them still works after the
    // self-closing siblings — proves layout/hit-test handles them.
    // Button is at y=24..48 given the <img> sits above it at y=0..24
    // with 24px default row height.
    app.click_canvas(20.0, 36.0).await.expect("click");
    let hv = app.hook_values(0).await.expect("hookValues");
    assert_eq!(
        hv.as_array()
            .and_then(|a| a.first())
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_i64()),
        Some(1),
        "click on bump button should still reach on_click despite img siblings",
    );

    app.shutdown().await;
}
// CODEGEN-END
