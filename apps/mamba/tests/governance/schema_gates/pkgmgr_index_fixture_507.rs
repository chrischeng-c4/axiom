//! Schema gate for the `mamba index build` frozen-local-index fixture.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("index")
        .join("manifest.toml")
}

#[test]
fn pkgmgr_index_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_index")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(507));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("index"));
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager")
    );
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn pkgmgr_index_action_pins_generated_layout_and_replay() {
    let doc = crate::common::load_toml(&manifest_path());
    let action = doc
        .get("action")
        .and_then(|v| v.as_table())
        .expect("manifest missing [action]");
    let command: Vec<&str> = action
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command,
        vec!["index", "build", "--out", "index", "wheels"],
        "index fixture must exercise the public CLI shape"
    );
    assert_eq!(
        action
            .get("must_create_index_layout")
            .and_then(|v| v.as_str()),
        Some("index/frozen-index-demo/0.2.0/frozen_index_demo-0.2.0-py3-none-any.whl")
    );
    assert_eq!(
        action
            .get("byte_identical_on_replay")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn pkgmgr_index_consumer_assertion_uses_add_and_lock_offline() {
    let doc = crate::common::load_toml(&manifest_path());
    let consumer = doc
        .get("consumer_assertion")
        .and_then(|v| v.as_table())
        .expect("manifest missing [consumer_assertion]");
    let add_command: Vec<&str> = consumer
        .get("add_command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let lock_command: Vec<&str> = consumer
        .get("lock_command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        add_command,
        vec!["add", "frozen-index-demo", "--index", "index"]
    );
    assert_eq!(lock_command, vec!["lock", "--index", "index"]);
    assert_eq!(
        consumer.get("must_run_offline").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn pkgmgr_index_malformed_wheel_case_fails_before_mutation() {
    let doc = crate::common::load_toml(&manifest_path());
    let malformed = doc
        .get("malformed_wheel_case")
        .and_then(|v| v.as_table())
        .expect("manifest missing [malformed_wheel_case]");
    assert_eq!(
        malformed.get("filename").and_then(|v| v.as_str()),
        Some("not-a-valid-wheel.whl")
    );
    assert_eq!(
        malformed
            .get("must_fail_before_output_mutation")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        malformed
            .get("diagnostic_message_substring")
            .and_then(|v| v.as_str()),
        Some("parse wheel filename")
    );
}

#[test]
fn pkgmgr_index_runner_contract_declares_required_keys() {
    let doc = crate::common::load_toml(&manifest_path());
    let contract = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .expect("manifest missing [runner_contract]");
    let keys: BTreeSet<&str> = contract
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in ["outcome", "index_path", "exit_code", "diagnostic_stream"] {
        assert!(
            keys.contains(required),
            "runner_contract keys must contain {required:?}; got {keys:?}"
        );
    }
}
