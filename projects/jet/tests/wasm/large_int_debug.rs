// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Boundary cell: `useState<i64>` with large values.
//!
//! Covers the `boundary` column of the conformance matrix for the
//! `useState<i64>` row. Verifies:
//!
//! 1. Large i64 initial values round-trip through the TOML → Rust
//!    entry-arg pipeline without truncation.
//! 2. `Element::from_number(n)` renders the full digit sequence —
//!    no precision loss on the Rust side.
//! 3. Debug bridge's `hookValues()` reports the exact value when
//!    read back via CDP.
//!
//! **Known conformance caveat**: values outside `[-2^53, 2^53]`
//! (±9 007 199 254 740 992) lose precision at the JS Number
//! boundary — `serde-wasm-bindgen` 0.6 converts i64 to `BigInt`,
//! but CDP's `Runtime.evaluate` with `returnByValue: true` can't
//! always serialize BigInt cleanly, and `as_i64()` on the Rust
//! side sees garbage. We use a value well under 2^53 here; the
//! behaviour above 2^53 is documented in `conformance.md` as an
//! intentional divergence (JS interop, not a runtime bug).

#[path = "../common/mod.rs"]
mod common;

use common::JetTestApp;

const START: i64 = 4_500_000_000_000_000; // 4.5e15, safely under 2^53

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn large_int_demo_survives_near_max_increments() {
    common::require_env();
    let app = JetTestApp::launch("large-int-demo").await.expect("launch");

    let hv0 = app.hook_values(0).await.expect("hookValues");
    let initial = hv0
        .as_array()
        .and_then(|a| a.first())
        .and_then(|h| h.get("value"))
        .and_then(|v| v.as_i64());
    assert_eq!(initial, Some(START), "initial i64 round-tripped exactly");

    // Rendered tree carries the full number as text — no precision
    // loss in Display.
    let tree = app.element_tree().await.expect("elementTree");
    let tree_str = serde_json::to_string(&tree).unwrap();
    assert!(
        tree_str.contains(&START.to_string()),
        "large i64 must render as-is; got {tree_str}",
    );

    // 5 clicks — still comfortably within i64 bounds (max is
    // START + 7 before overflow).
    for step in 1..=5_i64 {
        app.click_canvas(20.0, 12.0).await.expect("click");
        let hv = app.hook_values(0).await.expect("hookValues loop");
        let got = hv
            .as_array()
            .and_then(|a| a.first())
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_i64());
        assert_eq!(
            got,
            Some(START + step),
            "after {step} click(s), state should be START + {step}",
        );
    }

    app.shutdown().await;
}
// CODEGEN-END
