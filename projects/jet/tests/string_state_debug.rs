// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for `useState<string>`.
//!
//! Probes the transpiler's handling of a string-typed prop + hook
//! (`prop_field_types` maps `string` → `String`) and the runtime
//! primitive chain-downcast for `String` in `react::summarize_any`.
//!
//! Works end-to-end after the `emit_use_state_binding` + setter-arg
//! `.clone()` pair added in `tsx_to_rust::emit` — the transpiler
//! now emits `.clone()` on bare-identifier setter args (essential
//! for non-Copy captured state) and on non-Copy prop fields feeding
//! `use_state`.

mod common;

use common::JetTestApp;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn string_state_demo_surfaces_string_type_and_value() {
    common::require_env();
    let app = JetTestApp::launch("string-state-demo")
        .await
        .expect("launch");

    let hv = app.hook_values(0).await.expect("hookValues");
    let arr = hv.as_array().expect("array");
    assert_eq!(arr.len(), 1, "one useState<string>");
    let slot = &arr[0];

    assert_eq!(slot.get("kind").and_then(|v| v.as_str()), Some("State"),);
    assert_eq!(
        slot.get("type_name").and_then(|v| v.as_str()),
        Some("String"),
        "summarize_any should report String; got slot={slot}",
    );
    assert_eq!(
        slot.get("value").and_then(|v| v.as_str()),
        Some("hello"),
        "initial value from root_props=[\"hello\"]; got slot={slot}",
    );

    // The rendered tree should contain the initial value as a text
    // leaf.
    let tree = app.element_tree().await.expect("elementTree");
    let tree_str = serde_json::to_string(&tree).unwrap();
    assert!(
        tree_str.contains("hello"),
        "elementTree should render the string state; got {tree_str}",
    );

    app.shutdown().await;
}
// CODEGEN-END
