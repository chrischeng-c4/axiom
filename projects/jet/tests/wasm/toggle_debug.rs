// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for `useState<bool>` + `{cond && <span>}`.
//!
//! Known-working at the transpiler-unit level (see
//! `tests/tsx_to_rust_toggle.rs`); this test verifies the Rust that
//! comes out actually behaves when mounted + clicked + flushed in a
//! real browser.

#[path = "../common/mod.rs"]
mod common;

use common::JetTestApp;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn toggle_demo_flips_bool_and_conditionally_renders_span() {
    common::require_env();
    let app = JetTestApp::launch("toggle-demo").await.expect("launch");

    // Initial state: hook is bool=false, no conditional span.
    let hv = app.hook_values(0).await.expect("hookValues");
    let arr = hv.as_array().expect("array");
    assert_eq!(arr.len(), 1, "Toggle has exactly one useState");
    assert_eq!(
        arr[0].get("kind").and_then(|v| v.as_str()),
        Some("State"),
        "slot 0 is a State hook",
    );
    assert_eq!(
        arr[0].get("type_name").and_then(|v| v.as_str()),
        Some("bool"),
        "state is bool-typed",
    );
    assert_eq!(
        arr[0].get("value").and_then(|v| v.as_bool()),
        Some(false),
        "initial value false",
    );

    let before = app.element_tree().await.expect("elementTree");
    let before_str = serde_json::to_string(&before).unwrap();
    assert!(
        !before_str.contains("\"indicator\""),
        "conditional span should NOT be rendered when on=false; got {before_str}"
    );

    // Click the button — (20, 12) lands on the #flip button which is
    // the first child of the full-width <div>. The button stretches
    // to viewport width and is 24 px tall by default.
    app.click_canvas(20.0, 12.0).await.expect("click");

    // After click: bool=true, conditional span present.
    let hv2 = app.hook_values(0).await.expect("hookValues after click");
    assert_eq!(
        hv2.as_array()
            .and_then(|a| a.first())
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_bool()),
        Some(true),
        "click should flip bool to true",
    );

    let after = app.element_tree().await.expect("elementTree after click");
    let after_str = serde_json::to_string(&after).unwrap();
    assert!(
        after_str.contains("\"indicator\""),
        "conditional span SHOULD now render; got {after_str}"
    );

    app.shutdown().await;
}
// CODEGEN-END
