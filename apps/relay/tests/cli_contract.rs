// HANDWRITE-BEGIN gap="missing-generator:unit-test:relay-cli-contract" tracker="pending-tracker" reason="Prove the standard Relay CLI and shared structured-log configuration remain discoverable."
use std::process::Command;

fn relay(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_relay"))
        .args(args)
        .output()
        .expect("run relay CLI")
}

#[test]
fn help_exposes_standard_domain_and_observability_surfaces() {
    let output = relay(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for command in ["spec", "llm", "upgrade", "issue", "k8s", "dockerfile"] {
        assert!(
            stdout.contains(command),
            "missing {command} in relay --help"
        );
    }
    for flag in ["--bind", "--data-dir", "--log-format", "--otlp-endpoint"] {
        assert!(stdout.contains(flag), "missing {flag} in relay --help");
    }
    assert!(stdout.contains("pretty"));
    assert!(stdout.contains("json"));
}

#[test]
fn offline_spec_and_three_client_languages_share_one_contract() {
    let spec = relay(&["spec", "--format", "openapi"]);
    assert!(spec.status.success());
    let stdout = String::from_utf8(spec.stdout).unwrap();
    assert!(stdout.contains("/v1/{subject}/publish"));

    for (lang, entrypoint) in [
        ("ts", "index.ts"),
        ("py", "__init__.py"),
        ("rust", "mod.rs"),
    ] {
        let out = tempfile::tempdir().unwrap();
        let generated = relay(&[
            "spec",
            "gen",
            "--lang",
            lang,
            "--out",
            out.path().to_str().unwrap(),
        ]);
        assert!(
            generated.status.success(),
            "relay spec gen {lang} failed: {}",
            String::from_utf8_lossy(&generated.stderr)
        );
        assert!(out.path().join(entrypoint).is_file());
        assert!(String::from_utf8_lossy(&generated.stdout).contains("next: done"));
    }
}
// HANDWRITE-END
