// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Boundary cell: `useState<String>` with non-ASCII content.
//!
//! Exercises the full UTF-8 round-trip:
//!   TOML parse → Rust String literal in emitted code → wasm-bindgen
//!   → JS heap → `serde-wasm-bindgen` → CDP `Runtime.evaluate` →
//!   our serde_json::Value → assertion.
//!
//! Chinese + emoji (4-byte UTF-8) + Cyrillic covers 3-, 4-, and
//! 2-byte sequences in one test.

#[path = "../common/mod.rs"]
mod common;

use common::JetTestApp;

const GREETING: &str = "你好 👋 мир";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn unicode_demo_roundtrips_cjk_emoji_and_cyrillic() {
    common::require_env();
    let app = JetTestApp::launch("unicode-demo").await.expect("launch");

    // hookValues sees the exact string — every byte of UTF-8
    // survived the TOML → Rust → wasm → JS → CDP chain.
    let hv = app.hook_values(0).await.expect("hookValues");
    let got = hv
        .as_array()
        .and_then(|a| a.first())
        .and_then(|h| h.get("value"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    assert_eq!(got.as_deref(), Some(GREETING), "UTF-8 round-trip");

    // Same string appears as a text leaf in the rendered tree.
    // Walk the parsed value instead of string-containing — serde_json
    // escapes non-ASCII by default (`\u{...}`), which wouldn't
    // round-trip a `.contains(GREETING)` check.
    let tree = app.element_tree().await.expect("elementTree");
    fn find_text_leaf(v: &serde_json::Value) -> Option<String> {
        if v.get("kind").and_then(|k| k.as_str()) == Some("text") {
            return v
                .get("text")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());
        }
        if let Some(kids) = v.get("children").and_then(|c| c.as_array()) {
            for kid in kids {
                if let Some(hit) = find_text_leaf(kid) {
                    return Some(hit);
                }
            }
        }
        None
    }
    assert_eq!(
        find_text_leaf(&tree).as_deref(),
        Some(GREETING),
        "rendered text leaf carries the full UTF-8 sequence",
    );

    app.shutdown().await;
}
// CODEGEN-END
