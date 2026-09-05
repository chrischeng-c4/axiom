// HANDWRITE-BEGIN gap="sift-cli-operational-contract" tracker="1607" reason="Verify CLI standard surfaces, connect help, and parseable terminal output contracts."
use std::process::Command;

fn sift(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sift"))
        .args(args)
        .output()
        .expect("run sift CLI")
}

#[test]
fn standard_and_operational_commands_are_discoverable() {
    let output = sift(&["--help"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).expect("help is UTF-8");
    for command in [
        "serve",
        "collect",
        "query",
        "mcp",
        "snapshot",
        "restore",
        "backup",
        "dockerfile",
        "k8s",
        "connect",
        "spec",
        "llm",
        "upgrade",
        "issue",
    ] {
        assert!(
            help.contains(command),
            "missing `{command}` from sift --help"
        );
    }
    assert!(
        !help.contains("  event"),
        "legacy event command must be removed"
    );
    assert!(
        !help.contains("  replay"),
        "legacy replay command must be removed"
    );

    let query = sift(&["query", "--help"]);
    assert!(query.status.success());
    let help = String::from_utf8(query.stdout).expect("query help is UTF-8");
    for value in ["--endpoint", "--token", "<REQUEST>"] {
        assert!(help.contains(value), "missing `{value}` from query help");
    }

    let connect = sift(&["connect", "--help"]);
    assert!(connect.status.success());
    let help = String::from_utf8(connect.stdout).expect("connect help is UTF-8");
    for flag in [
        "--namespace",
        "--service",
        "--cr",
        "--local-port",
        "--secret",
        "--token",
    ] {
        assert!(
            help.contains(flag),
            "missing `{flag}` from sift connect --help"
        );
    }
}

#[test]
fn offline_json_commands_have_terminal_machine_output() {
    for args in [["llm", "--format", "json"].as_slice(), ["spec"].as_slice()] {
        let output = sift(args);
        assert!(
            output.status.success(),
            "sift {args:?}: {:?}",
            output.stderr
        );
        let value: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("offline command output must be JSON");
        assert_eq!(value["next"], "done");
    }
}
// HANDWRITE-END
