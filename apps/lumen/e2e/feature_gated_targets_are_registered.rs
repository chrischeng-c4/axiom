//! Registry and verification of feature-gated test targets in `apps/lumen`.
//!
//! Asserts that every test target carrying a crate-level `#![cfg(...)]` gate
//! is recorded in the registry with its exact required features, and fails if
//! the registry and repository tree disagree in either direction or if an e2e
//! test file is missing its declaration in `Cargo.toml`.

use std::fs;
use std::path::{Path, PathBuf};

struct GatedTarget {
    path: &'static str,
    gate: &'static str,
}

const REGISTRY: &[GatedTarget] = &[
    GatedTarget {
        path: "apps/lumen/e2e/access_render_cli.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/cli_client_ksa_token.rs",
        gate: r#"#![cfg(all(unix, feature = "delegated-auth", feature = "backup"))]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/operator_backup_kubernetes_wiring.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/operator_render.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/operator_retired_credential_projection.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/reshard_driver_e2e.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/routed_shard_e2e.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/capacity_catalog_contract.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/capacity_retire_hpa.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/body_limit_configurable.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
    GatedTarget {
        path: "apps/lumen/e2e/capacity_catalog_client.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
    },
];

fn repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("manifest parent dir")
        .parent()
        .expect("repo root dir")
        .to_path_buf()
}

fn find_cfg_gate(content: &str) -> Option<&str> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#![cfg(") {
            return Some(trimmed);
        }
    }
    None
}

fn count_test_rows(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("#[test]") || trimmed.starts_with("#[tokio::test]")
        })
        .count()
}

#[test]
fn all_gated_files_in_tree_are_registered() {
    let root = repo_root();
    let scan_roots = ["apps/lumen/e2e"];
    let mut missing_or_mismatched = Vec::new();

    for scan_root in &scan_roots {
        let dir = root.join(scan_root);
        if !dir.is_dir() {
            continue;
        }
        let entries =
            fs::read_dir(&dir).unwrap_or_else(|e| panic!("failed to read dir {:?}: {}", dir, e));

        for entry in entries {
            let entry = entry.expect("valid dir entry");
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
                if let Some(gate) = find_cfg_gate(&content) {
                    let rel_path = format!(
                        "{}/{}",
                        scan_root,
                        path.file_name().unwrap().to_str().unwrap()
                    );
                    if let Some(registered) = REGISTRY.iter().find(|r| r.path == rel_path) {
                        if registered.gate != gate {
                            missing_or_mismatched.push(format!(
                                "{} gate mismatch: tree has {:?}, registry has {:?}",
                                rel_path, gate, registered.gate
                            ));
                        }
                    } else {
                        missing_or_mismatched.push(format!(
                            "{} carries a `#![cfg(` gate that the registry does not list",
                            rel_path
                        ));
                    }
                }
            }
        }
    }

    assert!(
        missing_or_mismatched.is_empty(),
        "Gated files in tree not registered or mismatched:\n{}",
        missing_or_mismatched.join("\n")
    );
}

#[test]
fn all_registered_targets_exist_and_match_gate_in_tree() {
    let root = repo_root();
    for entry in REGISTRY {
        let file_path = root.join(entry.path);
        assert!(
            file_path.is_file(),
            "Registered target file does not exist: {}",
            entry.path
        );
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", file_path, e));
        let gate = find_cfg_gate(&content);
        assert_eq!(
            gate,
            Some(entry.gate),
            "Registry claims gate {:?} for {}, but file does not carry it",
            entry.gate,
            entry.path
        );
    }
}

#[test]
fn all_registered_targets_declare_non_zero_test_rows() {
    let root = repo_root();
    for entry in REGISTRY {
        let file_path = root.join(entry.path);
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", file_path, e));
        let row_count = count_test_rows(&content);
        assert!(
            row_count > 0,
            "Registered target {} declared 0 test rows",
            entry.path
        );
    }
}

#[test]
fn all_e2e_test_files_are_declared_in_cargo_toml() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|e| panic!("failed to read {:?}: {}", cargo_toml_path, e));

    let e2e_dir = manifest_dir.join("e2e");
    let entries = fs::read_dir(&e2e_dir)
        .unwrap_or_else(|e| panic!("failed to read dir {:?}: {}", e2e_dir, e));

    let mut undeclared = Vec::new();
    for entry in entries {
        let entry = entry.expect("valid dir entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let expected_decl = format!("path = \"e2e/{}\"", file_name);
            if !cargo_toml_content.contains(&expected_decl) {
                undeclared.push(file_name.to_string());
            }
        }
    }

    assert!(
        undeclared.is_empty(),
        "e2e source files missing [[test]] declaration in Cargo.toml: {:?}",
        undeclared
    );
}
