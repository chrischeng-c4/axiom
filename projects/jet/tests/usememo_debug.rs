// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for `useMemo` — the second hook kind the
//! transpiler emits (after `useState`).
//!
//! Basic-cell coverage per `conformance.md` §Process: verifies the
//! hook count, both slot kinds, and that the derived value tracks
//! the state value across a click.

mod common;

use common::JetTestApp;

/// Find the first text leaf whose content contains `needle`.
fn tree_contains_text(v: &serde_json::Value, needle: &str) -> bool {
    if v.get("kind").and_then(|k| k.as_str()) == Some("text") {
        if let Some(t) = v.get("text").and_then(|t| t.as_str()) {
            if t.contains(needle) {
                return true;
            }
        }
    }
    if let Some(kids) = v.get("children").and_then(|c| c.as_array()) {
        for kid in kids {
            if tree_contains_text(kid, needle) {
                return true;
            }
        }
    }
    false
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn usememo_demo_tracks_state_across_click() {
    common::require_env();
    let app = JetTestApp::launch("usememo-demo").await.expect("launch");

    // React-tier sanity: fiber has exactly 2 hooks (State + Memo).
    let hv0 = app.hook_values(0).await.expect("hookValues");
    let arr = hv0.as_array().expect("array");
    assert_eq!(arr.len(), 2, "two hooks expected (useState + useMemo)");
    assert_eq!(
        arr[0].get("kind").and_then(|v| v.as_str()),
        Some("State"),
        "slot 0 is the useState",
    );
    assert_eq!(
        arr[1].get("kind").and_then(|v| v.as_str()),
        Some("Memo"),
        "slot 1 is the useMemo",
    );
    // Initial values: State = 0, Memo = 0 * 2 = 0.
    assert_eq!(
        arr[0].get("value").and_then(|v| v.as_i64()),
        Some(0),
        "initial state value",
    );
    assert_eq!(
        arr[1].get("value").and_then(|v| v.as_i64()),
        Some(0),
        "initial memo value = 0 * 2 = 0",
    );

    // Observable: the rendered tree shows both numbers.
    let tree_before = app.element_tree().await.expect("elementTree");
    assert!(
        tree_contains_text(&tree_before, "0"),
        "tree should show the state value; got {tree_before}"
    );

    // Click → n becomes 1, memo recomputes to 2.
    app.click_canvas(30.0, 12.0).await.expect("click");

    let hv1 = app.hook_values(0).await.expect("hookValues after click");
    let arr1 = hv1.as_array().expect("array");
    assert_eq!(
        arr1[0].get("value").and_then(|v| v.as_i64()),
        Some(1),
        "state bumped to 1",
    );
    assert_eq!(
        arr1[1].get("value").and_then(|v| v.as_i64()),
        Some(2),
        "memo recomputed to 2 (= 1 * 2) because dep [n] changed",
    );

    // Observable confirmation — the text leaf with "2" is present
    // somewhere in the tree.
    let tree_after = app.element_tree().await.expect("elementTree after");
    assert!(
        tree_contains_text(&tree_after, "2"),
        "tree should show the recomputed memo value; got {tree_after}"
    );

    app.shutdown().await;
}
// CODEGEN-END
