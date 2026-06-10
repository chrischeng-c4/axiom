// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for `items.map(x => <JSX/>)` over a prop
//! array. Companion to `list_render_debug` (which covers the
//! `[...Array(n)].map(...)` range shape); this one exercises the
//! general iter-map over an owned `Vec<T>` prop.
//!
//! **Exemplar for the snapshot-style conformance pattern** —
//! `conformance.md` §Snapshot strategy. The full element-tree
//! assertion goes through `snapshot_eq!`; hand-written equality
//! is reserved for the one targeted fiber-hook-count check, which
//! is a React-specific concern.

#[path = "../common/mod.rs"]
mod common;

use common::JetTestApp;
// `snapshot_eq!` is `#[macro_export]`ed from common::snapshot, so
// it's available at the crate root of this integration-test binary.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn items_list_demo_element_tree_matches_snapshot() {
    common::require_env();
    let app = JetTestApp::launch("items-list-demo").await.expect("launch");

    // Observable (framework-agnostic) assertion — the whole tree
    // shape is frozen. Any drift (attribute rename, intrinsic-tag
    // change, Fragment flatten regression, text padding change)
    // surfaces as a JSON diff against the committed snapshot.
    let tree = app.element_tree().await.expect("elementTree");
    snapshot_eq!("items_list_initial", &tree);

    // React-specific sanity: hookless component → hook_count=0.
    // When Vue / Angular adapters arrive, a parallel test file
    // would assert their framework-specific equivalent here (or
    // skip this block entirely).
    let fibers = app.fiber_tree().await.expect("fiberTree");
    let hook_count = fibers
        .as_array()
        .and_then(|a| a.first())
        .and_then(|f| f.get("hook_count"))
        .and_then(|v| v.as_u64());
    assert_eq!(hook_count, Some(0), "hookless list component");

    app.shutdown().await;
}
// CODEGEN-END
