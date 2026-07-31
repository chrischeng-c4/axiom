// HANDWRITE-BEGIN gap="missing-generator:unit-test:9ceac845" tracker="pending-tracker" reason="Prove registry persistence, selection, future launch path, absence of child launch, UI contract, and the Jet E2E evidence gate."
use std::path::{Path, PathBuf};
use std::process::Command;

use workbench::folder_shell::ShellState;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("apps/workbench lives below the repository root")
        .to_path_buf()
}

/// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#unit-test
#[test]
fn registry_persists_identity_selection_and_future_launch_path() {
    let temp = tempfile::tempdir().unwrap();
    let alpha = temp.path().join("alpha");
    let beta = temp.path().join("beta");
    std::fs::create_dir_all(&alpha).unwrap();
    std::fs::create_dir_all(&beta).unwrap();

    let mut state = ShellState::default();
    let alpha_folder = state.register_path(&alpha).unwrap();
    let duplicate = state.register_path(&alpha).unwrap();
    assert_eq!(duplicate, alpha_folder);
    assert_eq!(state.folders.len(), 1, "canonical paths are de-duplicated");

    let beta_folder = state.register_path(&beta).unwrap();
    assert_eq!(state.selected_id.as_deref(), Some(beta_folder.id.as_str()));
    state.select(&alpha_folder.id).unwrap();
    assert_eq!(
        state.selected_launch_path(),
        alpha.canonicalize().unwrap().to_str()
    );

    let state_path = temp.path().join("config/folder-shell.json");
    state.save_to(&state_path).unwrap();
    let serialized = std::fs::read_to_string(&state_path).unwrap();
    for forbidden in ["collapsed", "cwd", "process", "pty", "renderer"] {
        assert!(
            !serialized.to_ascii_lowercase().contains(forbidden),
            "registry persisted forbidden field {forbidden}: {serialized}"
        );
    }

    let mut loaded = ShellState::load_from(&state_path).unwrap();
    assert_eq!(loaded, state);
    assert_eq!(loaded.selected_launch_path(), state.selected_launch_path());
    assert!(loaded.select("missing-folder").is_err());
    assert!(state.register_path(&state_path).is_err());
}

/// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#unit-test
#[test]
fn folder_shell_does_not_own_agent_process_or_terminal_cwd() {
    let rust = include_str!("../src/folder_shell.rs");
    let script = include_str!("../ui/shell.js");
    for forbidden in [
        "std::process::Command",
        "portable_pty",
        "TerminalSession",
        "set_current_dir",
        "current_dir()",
        "launch_agent",
        "aw::",
    ] {
        assert!(
            !rust.contains(forbidden) && !script.contains(forbidden),
            "folder shell owns forbidden runtime surface {forbidden}"
        );
    }

    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(config["app"]["withGlobalTauri"], true);
    assert!(config["app"]["windows"][0]["minWidth"].as_u64().unwrap() <= 860);

    let html = include_str!("../ui/index.html");
    let css = include_str!("../ui/shell.css");
    for contract in [
        "<nav",
        "<main",
        "<aside",
        "role=\"status\"",
        "Add launch folder",
        "Context",
    ] {
        assert!(html.contains(contract), "missing shell contract {contract}");
    }
    assert!(css.contains("@media (max-width: 900px)"));
    assert!(css.contains(":focus-visible"));
}

/// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#unit-test
#[test]
fn rendered_folder_shell_journey_passes() {
    let root = repository_root();
    let evidence = root.join("apps/workbench/evidence/folder-shell/2192");
    let jet_evidence = tempfile::tempdir().unwrap();
    let spec = root.join("apps/workbench/e2e/folder-shell.spec.js");

    let output = Command::new("jet")
        .current_dir(&root)
        .arg("e2e")
        .arg("run")
        .arg("--trace")
        .arg("on")
        .arg("--timeout")
        .arg("60000")
        .arg("--workers")
        .arg("1")
        .arg("--evidence-dir")
        .arg(jet_evidence.path())
        .arg(&spec)
        .output()
        .expect("run the installed Jet browser journey");
    assert!(
        output.status.success(),
        "Jet folder-shell journey failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_png_dimensions(&evidence.join("desktop.png"), 1440, 900);
    assert_png_dimensions(&evidence.join("constrained.png"), 860, 720);
    let journey: serde_json::Value =
        serde_json::from_slice(&std::fs::read(evidence.join("journey.json")).unwrap()).unwrap();
    assert_eq!(
        journey["schemaVersion"],
        "workbench.folder-shell.evidence.v1"
    );
    assert_eq!(journey["workItem"], 2192);
    assert_eq!(journey["viewports"]["desktop"]["width"], 1440);
    assert_eq!(journey["viewports"]["constrained"]["width"], 860);
    assert_eq!(journey["interactions"]["collapse"], true);
    assert_eq!(journey["interactions"]["arrowNavigation"], true);
    assert_eq!(journey["functionalStates"]["invalidPath"], true);
    assert_eq!(journey["accessibility"]["landmarks"], true);
    assert_eq!(journey["noChildProcess"], true);
    assert_eq!(
        journey["selectedLaunchPath"],
        "/Users/demo/axiom/app_workbench"
    );
}

fn assert_png_dimensions(path: &Path, expected_width: u32, expected_height: u32) {
    let bytes = std::fs::read(path).unwrap_or_else(|error| {
        panic!(
            "read retained viewport evidence {}: {error}",
            path.display()
        )
    });
    assert!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "{} is not PNG",
        path.display()
    );
    assert!(bytes.len() >= 24, "{} has no PNG IHDR", path.display());
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    assert_eq!((width, height), (expected_width, expected_height));
}
// HANDWRITE-END
