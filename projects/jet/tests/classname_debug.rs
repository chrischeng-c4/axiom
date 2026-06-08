// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Runtime verification for the `className` attribute.
//!
//! The transpiler maps JSX `className="primary"` → Rust
//! `class_name: Some("primary".to_string())` on Props. We confirm
//! that value flows all the way to the serialized element tree the
//! debug bridge exposes.

mod common;

use common::JetTestApp;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn classname_demo_surfaces_class_name_in_element_tree() {
    common::require_env();
    let app = JetTestApp::launch("classname-demo").await.expect("launch");

    let tree = app.element_tree().await.expect("elementTree");

    // The root is <button id="cta" className="primary">. The debug
    // mirror renames to snake_case (`class_name`), matching the
    // Rust Props field.
    assert_eq!(tree.get("kind").and_then(|v| v.as_str()), Some("intrinsic"),);
    assert_eq!(
        tree.get("tag").and_then(|v| v.as_str()),
        Some("button"),
        "root should be a button; got {tree}",
    );

    let props = tree
        .get("props")
        .unwrap_or_else(|| panic!("no props on root; got {tree}"));
    assert_eq!(
        props.get("class_name").and_then(|v| v.as_str()),
        Some("primary"),
        "className should surface as class_name=\"primary\"; got props={props}",
    );
    assert_eq!(
        props.get("id").and_then(|v| v.as_str()),
        Some("cta"),
        "id should coexist with className; got props={props}",
    );

    app.shutdown().await;
}
// CODEGEN-END
