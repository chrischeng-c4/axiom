//! Black-box contract for `lumen standalone compose patch`.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_yaml::Value;

const IMAGE: &str = "ghcr.io/chrischeng-c4/lumen:0.4.31";
const MANAGED_LABEL: &str = "com.axiom.lumen.managed";

fn run(current_dir: &Path, file: &OsStr, name: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lumen"));
    command
        .current_dir(current_dir)
        .args(["standalone", "compose", "patch", "--file"])
        .arg(file);
    if let Some(name) = name {
        command.args(["--name", name]);
    }
    command.output().expect("run standalone compose patch")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed: status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn read_yaml(path: &Path) -> Value {
    serde_yaml::from_slice(&fs::read(path).expect("read Compose file"))
        .expect("parse Compose output")
}

fn assert_managed_service(root: &Value, name: &str) {
    let service = &root["services"][name];
    assert_eq!(service["image"], IMAGE);
    assert_eq!(service["ports"].as_sequence().unwrap().len(), 1);
    assert_eq!(service["ports"][0], "127.0.0.1:7373:7373");
    assert_eq!(service["volumes"].as_sequence().unwrap().len(), 1);
    assert_eq!(
        service["volumes"][0],
        format!("{name}-data:/var/lib/lumen/data")
    );
    assert_eq!(service["environment"]["LUMEN_AUTH"], "off");
    assert_eq!(service["labels"][MANAGED_LABEL], "true");
    assert!(root["volumes"][format!("{name}-data")]
        .as_mapping()
        .is_some());
}

fn assert_refused_without_write(source: &str, name: Option<&str>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("compose.yaml");
    fs::write(&path, source).unwrap();
    let before = fs::read(&path).unwrap();
    let output = run(dir.path(), OsStr::new("compose.yaml"), name);
    assert!(
        !output.status.success(),
        "invalid input unexpectedly succeeded: {source:?}"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "refusal changed input bytes for {source:?}"
    );
}

#[test]
fn absent_relative_file_gets_the_exact_default_and_is_byte_idempotent() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.4.31");
    let dir = tempfile::tempdir().unwrap();
    let relative = OsStr::new("compose.yaml");

    assert_success(&run(dir.path(), relative, None));
    let path = dir.path().join(relative);
    let first = fs::read(&path).unwrap();
    assert_managed_service(&read_yaml(&path), "lumen");

    assert_success(&run(dir.path(), relative, None));
    assert_eq!(fs::read(&path).unwrap(), first);
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name())
        .filter(|name| name.to_string_lossy().contains(".lumen.tmp-"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temporary files remain: {leftovers:?}"
    );
}

#[test]
fn custom_name_preserves_other_services_and_every_top_level_setting() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("compose.yaml");
    let source = r#"version: "3.9"
name: keep-project
services:
  api:
    image: example/api:1
    profiles: [dev]
networks:
  backend:
    driver: bridge
volumes:
  keep-volume:
    external: true
configs:
  api-config:
    file: ./api.conf
secrets:
  api-secret:
    file: ./api.secret
profiles: [top-level-control]
x-lumen-test:
  nested: [one, two]
"#;
    fs::write(&path, source).unwrap();
    let before: Value = serde_yaml::from_str(source).unwrap();

    assert_success(&run(
        dir.path(),
        OsStr::new("compose.yaml"),
        Some("search.v1_test"),
    ));
    let after = read_yaml(&path);
    for key in [
        "version",
        "name",
        "networks",
        "configs",
        "secrets",
        "profiles",
        "x-lumen-test",
    ] {
        assert_eq!(after[key], before[key], "top-level {key} changed");
    }
    assert_eq!(after["services"]["api"], before["services"]["api"]);
    assert_eq!(
        after["volumes"]["keep-volume"],
        before["volumes"]["keep-volume"]
    );
    assert_managed_service(&after, "search.v1_test");
}

#[test]
fn both_standard_managed_label_forms_authorize_replacement() {
    for labels in [
        "labels:\n      com.axiom.lumen.managed: 'true'",
        "labels:\n      com.axiom.lumen.managed: true",
        "labels:\n      - com.axiom.lumen.managed=true",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("compose.yaml");
        fs::write(
            &path,
            format!("services:\n  lumen:\n    image: old\n    {labels}\n"),
        )
        .unwrap();
        assert_success(&run(dir.path(), OsStr::new("compose.yaml"), None));
        assert_managed_service(&read_yaml(&path), "lumen");
    }
}

#[test]
fn unmanaged_service_is_refused_with_exact_input_bytes() {
    for source in [
        "services:\n  lumen:\n    image: custom\n",
        "services:\n  lumen:\n    labels:\n      com.axiom.lumen.managed: 'false'\n",
        "services:\n  lumen:\n    labels:\n      - com.axiom.lumen.managed=false\n",
        "services:\n  lumen:\n    labels: com.axiom.lumen.managed=true\n",
        "services:\n  lumen: scalar\n",
    ] {
        assert_refused_without_write(source, None);
    }
}

#[test]
fn malformed_or_incompatible_yaml_is_refused_with_exact_input_bytes() {
    for source in [
        "services: [\n",
        "- not\n- a\n- mapping\n",
        "services: []\n",
        "services: {}\nvolumes: []\n",
    ] {
        assert_refused_without_write(source, None);
    }
}

#[test]
fn invalid_names_are_rejected_without_creating_a_file() {
    let too_long = "a".repeat(64);
    for name in [
        "",
        "-leading",
        ".leading",
        "_leading",
        "bad/name",
        "bad name",
        "ümlaut",
        too_long.as_str(),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let output = run(dir.path(), OsStr::new("compose.yaml"), Some(name));
        assert!(!output.status.success(), "invalid name accepted: {name:?}");
        assert!(!dir.path().join("compose.yaml").exists());
    }
}

#[test]
fn an_existing_empty_file_is_treated_as_an_empty_compose_document() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("compose.yaml");
    fs::write(&path, "\n  \n").unwrap();
    assert_success(&run(dir.path(), OsStr::new("compose.yaml"), None));
    assert_managed_service(&read_yaml(&path), "lumen");
}
