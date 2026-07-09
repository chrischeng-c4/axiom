// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Boundary cell: `items.map()` over a 100-element `Vec<i64>`.
//!
//! Same component shape as `items-list-demo`, scaled up. Asserts:
//! - Every one of 100 spans lands in the Element tree.
//! - They appear in declaration order (0, 1, 2, ..., 99).
//! - Layout produces enough laid-out nodes — the full-repaint
//!   vertical stack handles 100 intrinsic children without
//!   collapsing or dropping.

#[path = "../common/mod.rs"]
mod common;

use common::JetTestApp;

fn span_texts(v: &serde_json::Value, out: &mut Vec<String>) {
    let kind = v.get("kind").and_then(|k| k.as_str());
    let children = v.get("children").and_then(|c| c.as_array());

    if kind == Some("intrinsic") && v.get("tag").and_then(|t| t.as_str()) == Some("span") {
        let mut acc = String::new();
        if let Some(kids) = children {
            for kid in kids {
                if kid.get("kind").and_then(|k| k.as_str()) == Some("text") {
                    if let Some(t) = kid.get("text").and_then(|t| t.as_str()) {
                        acc.push_str(t);
                    }
                }
            }
        }
        out.push(acc);
    }

    if let Some(kids) = children {
        for kid in kids {
            span_texts(kid, out);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn big_list_demo_renders_100_spans_in_order() {
    common::require_env();
    let app = JetTestApp::launch("big-list-demo").await.expect("launch");

    let tree = app.element_tree().await.expect("elementTree");
    let mut spans: Vec<String> = Vec::new();
    span_texts(&tree, &mut spans);

    assert_eq!(
        spans.len(),
        100,
        "expected 100 spans from 100-item prop; got {}",
        spans.len()
    );
    let expected: Vec<String> = (0..100).map(|i| format!("item {i}")).collect();
    assert_eq!(spans, expected, "spans must appear in declaration order");

    // Layout should have at least 100 intrinsic nodes + 1 root div
    // + 200 text leaves (2 per span: "item " + number). Don't pin
    // the exact count — just sanity-check "much more than 10".
    let layout = app.layout_tree().await.expect("layoutTree");
    let node_count = layout
        .get("nodes")
        .and_then(|n| n.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert!(
        node_count >= 100,
        "layout should produce ≥100 nodes for a 100-item list; got {node_count}",
    );

    app.shutdown().await;
}
// CODEGEN-END
