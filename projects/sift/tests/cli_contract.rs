// HANDWRITE-BEGIN gap="sift-agent-cli-contract-tests" tracker="1604" reason="Verify JSON CLI output parses and spec generation emits a typed-client entrypoint."
use std::process::Command;

#[test]
fn machine_readable_cli_output_is_valid_json_and_terminal() {
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args(["llm", "--format", "json"])
        .output()
        .expect("run sift llm");
    assert!(output.status.success(), "{output:?}");

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("machine-readable output must remain JSON");
    assert_eq!(value["next"], "done");
    assert!(value["topics"].is_array());
}

#[test]
fn spec_gen_writes_a_typed_client_entrypoint() {
    let output_dir = tempfile::tempdir().expect("temporary generated-client directory");
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "spec",
            "gen",
            "--lang",
            "rust",
            "--out",
            output_dir.path().to_str().expect("utf-8 temporary path"),
        ])
        .output()
        .expect("run sift spec gen");
    assert!(output.status.success(), "{output:?}");
    assert!(output_dir.path().join("mod.rs").is_file());
    assert!(String::from_utf8_lossy(&output.stdout).contains("next:"));
}

// HANDWRITE-END
