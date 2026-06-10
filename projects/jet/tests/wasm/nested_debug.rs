// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for deeply nested JSX.
//!
//! Confirms the transpiler's recursive element emission + the
//! renderer's layout over multiple tag layers.

#[path = "../common/mod.rs"]
mod common;

use common::JetTestApp;

/// Count intrinsic-node depth in a serialized element tree.
/// Text / component / empty nodes count as leaves (don't recurse).
fn intrinsic_depth(v: &serde_json::Value) -> usize {
    match v.get("kind").and_then(|k| k.as_str()) {
        Some("intrinsic") => {
            let child_max = v
                .get("children")
                .and_then(|c| c.as_array())
                .map(|cs| cs.iter().map(intrinsic_depth).max().unwrap_or(0))
                .unwrap_or(0);
            1 + child_max
        }
        _ => 0,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn nested_demo_preserves_tree_depth_and_hit_tests_through_layers() {
    common::require_env();
    let app = JetTestApp::launch("nested-demo").await.expect("launch");

    let tree = app.element_tree().await.expect("elementTree");
    assert_eq!(
        intrinsic_depth(&tree),
        4,
        "expected div>div>button>span (depth 4); got tree={tree}"
    );

    // layoutTree should have laid out at least three intrinsic
    // nodes (outer div, middle div, button); spans contribute text
    // which shows up as a Text node too.
    let layout = app.layout_tree().await.expect("layoutTree");
    let nodes = layout
        .get("nodes")
        .and_then(|v| v.as_array())
        .expect("layoutTree.nodes is an array");
    assert!(
        nodes.len() >= 3,
        "expected ≥3 layout nodes for nested tree, got {}",
        nodes.len()
    );

    // pickAt inside the rendered button area: (30, 12) is inside
    // the default full-width 24px-tall button. Hit-test walks layout
    // nodes in reverse z-order, so a text leaf painted over its
    // parent button returns first — that's expected behaviour and
    // proves the inner-most layout position matched. Assertion: we
    // hit SOMETHING (not null) and it's an inner-most node (text
    // with button/span content, or a span/button intrinsic).
    let pick = app.pick_at(30.0, 12.0).await.expect("pickAt");
    assert!(!pick.is_null(), "pick should hit something at (30, 12)");
    let kind = pick
        .get("node")
        .and_then(|n| n.get("kind"))
        .and_then(|k| k.get("kind"))
        .and_then(|k| k.as_str());
    let text_content = pick
        .get("node")
        .and_then(|n| n.get("kind"))
        .and_then(|k| k.get("text"))
        .and_then(|t| t.as_str());
    let tag = pick
        .get("node")
        .and_then(|n| n.get("kind"))
        .and_then(|k| k.get("tag"))
        .and_then(|t| t.as_str());
    assert!(
        matches!(kind, Some("text") | Some("intrinsic")),
        "pick at (30,12) should be a text or intrinsic node; got {pick}",
    );
    if kind == Some("text") {
        // Text "count: " or the number leaf is expected for our
        // inner-span layout.
        assert!(
            text_content
                .map(|t| t.contains("count") || t.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false),
            "text leaf should be part of the counter label; got {pick}",
        );
    } else {
        assert!(
            matches!(tag, Some("button") | Some("span")),
            "intrinsic hit should be innermost (button/span); got {pick}",
        );
    }

    // Click path intactly reaches the innermost button's on_click
    // despite the two outer div wrappers — exercises hit-test
    // walking back up the tree.
    app.click_canvas(30.0, 12.0).await.expect("click");
    let hv = app.hook_values(0).await.expect("hookValues");
    assert_eq!(
        hv.as_array()
            .and_then(|a| a.first())
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_i64()),
        Some(1),
        "click on nested button should bump useState from 0 to 1",
    );

    app.shutdown().await;
}
// CODEGEN-END
