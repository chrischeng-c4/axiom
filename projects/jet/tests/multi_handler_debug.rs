// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for multiple `onClick` handlers within one
//! component — each handler must only mutate its own state slot.

mod common;

use common::JetTestApp;

fn hook_i64(hv: &serde_json::Value, slot: usize) -> Option<i64> {
    hv.as_array()?.get(slot)?.get("value")?.as_i64()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn multi_handler_demo_isolates_state_per_button() {
    common::require_env();
    let app = JetTestApp::launch("multi-handler-demo")
        .await
        .expect("launch");

    // Initial: two State hooks, both 0.
    let hv0 = app.hook_values(0).await.expect("hookValues");
    let arr = hv0.as_array().expect("array");
    assert_eq!(arr.len(), 2, "expected 2 State hooks; got {arr:?}");
    assert_eq!(hook_i64(&hv0, 0), Some(0));
    assert_eq!(hook_i64(&hv0, 1), Some(0));

    // Buttons stack vertically at the default 24-px height — button
    // A sits at y=0..24, button B at y=24..48. Click each and
    // verify only the corresponding hook slot moved.
    app.click_canvas(10.0, 12.0).await.expect("click A");
    let hv1 = app.hook_values(0).await.expect("hv after A");
    assert_eq!(hook_i64(&hv1, 0), Some(1), "a should bump by 1");
    assert_eq!(hook_i64(&hv1, 1), Some(0), "b should NOT change");

    app.click_canvas(10.0, 36.0).await.expect("click B");
    let hv2 = app.hook_values(0).await.expect("hv after B");
    assert_eq!(hook_i64(&hv2, 0), Some(1), "a should still be 1");
    assert_eq!(hook_i64(&hv2, 1), Some(10), "b should jump by 10");

    // Click B again to prove the increment is +10 and deterministic.
    app.click_canvas(10.0, 36.0).await.expect("click B 2");
    let hv3 = app.hook_values(0).await.expect("hv after B×2");
    assert_eq!(hook_i64(&hv3, 0), Some(1));
    assert_eq!(hook_i64(&hv3, 1), Some(20));

    app.shutdown().await;
}
// CODEGEN-END
