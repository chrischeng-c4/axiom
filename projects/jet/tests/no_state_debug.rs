// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for a component with zero hooks.
//!
//! Exercises the transpiler's "no `useState` binding" path and the
//! runtime's handling of a Fiber with `hooks: Vec::new()`. If
//! anything in the commit loop assumes at least one hook, this
//! test surfaces it.

mod common;

use common::JetTestApp;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn no_state_demo_runs_with_zero_hooks() {
    common::require_env();
    let app = JetTestApp::launch("no-state-demo").await.expect("launch");

    let fibers = app.fiber_tree().await.expect("fiberTree");
    let arr = fibers.as_array().expect("fiberTree is array");
    assert_eq!(arr.len(), 1, "single component = single fiber");
    assert_eq!(
        arr[0].get("hook_count").and_then(|v| v.as_u64()),
        Some(0),
        "zero hooks expected; fiber={:?}",
        arr[0],
    );
    assert_eq!(
        arr[0].get("dirty").and_then(|v| v.as_bool()),
        Some(false),
        "fresh fiber shouldn't be dirty",
    );

    // hookValues on a hookless fiber should be the empty array,
    // not a panic.
    let hv = app
        .hook_values(0)
        .await
        .expect("hookValues on hookless fiber");
    assert_eq!(
        hv.as_array().map(|a| a.len()),
        Some(0),
        "hookless fiber yields empty hook array; got {hv}",
    );

    // The prop value must reach the rendered text.
    let tree = app.element_tree().await.expect("elementTree");
    let tree_str = serde_json::to_string(&tree).unwrap();
    assert!(
        tree_str.contains("\"42\""),
        "root_props=[42] should appear as text in the tree; got {tree_str}",
    );
    assert!(
        tree_str.contains("\"value: \""),
        "static text prefix should also appear; got {tree_str}",
    );

    // Layout should still work — at minimum two laid-out nodes
    // (the div wrapper + span child).
    let layout = app.layout_tree().await.expect("layoutTree");
    let nodes = layout
        .get("nodes")
        .and_then(|v| v.as_array())
        .expect("layoutTree.nodes array");
    assert!(
        nodes.len() >= 2,
        "layout should include wrapper + inner span; got {}",
        nodes.len(),
    );

    app.shutdown().await;
}
// CODEGEN-END
