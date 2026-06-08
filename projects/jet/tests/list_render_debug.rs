// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for list rendering via `.map()`.
//!
//! Passes end-to-end after:
//! - `Element::Fragment(Vec<Element>)` added to `jet_wasm::Element`;
//!   renderer's layout + paint walks treat it as transparent.
//! - `tsx_to_rust::emit_jsx_interp_child` recognizes
//!   `[...Array(N)].map((_, idx) => body)` and emits
//!   `Element::Fragment((0..N).map(|idx| body).collect())`.

mod common;

use common::JetTestApp;

/// Count `<span>` intrinsics anywhere inside the tree — either as
/// direct children or inside any `Fragment` child. The debug
/// mirror surfaces `Fragment` as its own kind to avoid lying
/// about the tree shape, so consumers need to walk it explicitly.
fn count_spans(v: &serde_json::Value) -> usize {
    let kind = v.get("kind").and_then(|k| k.as_str());
    let empty: Vec<serde_json::Value> = Vec::new();
    let children = v
        .get("children")
        .and_then(|c| c.as_array())
        .unwrap_or(&empty);
    let self_count =
        if kind == Some("intrinsic") && v.get("tag").and_then(|t| t.as_str()) == Some("span") {
            1
        } else {
            0
        };
    self_count + children.iter().map(count_spans).sum::<usize>()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn list_render_demo_grows_children_on_click() {
    common::require_env();
    let app = JetTestApp::launch("list-render-demo")
        .await
        .expect("launch");

    // Initial render with n=3 should have 3 spans.
    let tree = app.element_tree().await.expect("elementTree");
    let spans = count_spans(&tree);
    assert_eq!(
        spans, 3,
        "initial render expected 3 spans; got {spans} from tree={tree}",
    );

    // Click "add" to grow to 4.
    app.click_canvas(10.0, 12.0).await.expect("click add");
    let tree2 = app.element_tree().await.expect("elementTree after click");
    let spans2 = count_spans(&tree2);
    assert_eq!(
        spans2, 4,
        "post-click expected 4 spans; got {spans2} from tree={tree2}",
    );

    app.shutdown().await;
}
// CODEGEN-END
