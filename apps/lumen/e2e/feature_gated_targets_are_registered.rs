//! Registry and verification of feature-gated test targets in `apps/lumen`.
//!
//! Asserts that every test target carrying a crate-level `#![cfg(...)]` gate
//! is recorded in the registry with its exact required features, and fails if
//! the registry and repository tree disagree in either direction or if an e2e
//! test file is missing its declaration in `Cargo.toml`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

struct GatedTarget {
    path: &'static str,
    gate: &'static str,
    required_features: &'static [&'static str],
}

impl GatedTarget {
    fn cargo_name(&self) -> &str {
        Path::new(self.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("file stem")
    }

    fn cargo_path(&self) -> &str {
        self.path.strip_prefix("apps/lumen/").unwrap_or(self.path)
    }
}

const REGISTRY: &[GatedTarget] = &[
    GatedTarget {
        path: "apps/lumen/e2e/access_render_cli.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/cli_client_ksa_token.rs",
        gate: r#"#![cfg(all(unix, feature = "delegated-auth", feature = "backup"))]"#,
        required_features: &["delegated-auth", "backup"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/operator_backup_kubernetes_wiring.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/operator_render.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/operator_retired_credential_projection.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/reshard_driver_e2e.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/routed_shard_e2e.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/capacity_catalog_contract.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/capacity_retire_hpa.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/body_limit_configurable.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/capacity_catalog_client.rs",
        gate: r#"#![cfg(feature = "operator")]"#,
        required_features: &["operator"],
    },
    GatedTarget {
        path: "apps/lumen/e2e/standalone_backup_restore_cli.rs",
        gate: r#"#![cfg(unix)]"#,
        required_features: &[],
    },
];

const MAINTAINED_CMD: &str = r#"`cargo test -p lumen --features "operator delegated-auth"`"#;
const DIRECT_FEATURES: &[&str] = &["operator", "delegated-auth"];

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

fn validate_manifest_and_gate(cargo_str: &str, contributing_str: &str) -> Result<(), String> {
    let manifest: Value =
        toml::from_str(cargo_str).map_err(|e| format!("invalid Cargo.toml: {e}"))?;

    let tests = manifest
        .get("test")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing [[test]] in Cargo.toml".to_string())?;

    let mut seen_names = HashSet::new();
    let mut seen_paths = HashSet::new();
    for t in tests {
        let name = t
            .get("name")
            .and_then(Value::as_str)
            .ok_or("test missing name")?;
        let path = t
            .get("path")
            .and_then(Value::as_str)
            .ok_or("test missing path")?;
        if !seen_names.insert(name) {
            return Err(format!("duplicate test name in Cargo.toml: {name}"));
        }
        if !seen_paths.insert(path) {
            return Err(format!("duplicate test path in Cargo.toml: {path}"));
        }
    }

    for entry in REGISTRY {
        let expected_name = entry.cargo_name();
        let expected_path = entry.cargo_path();

        let matching = tests.iter().find(|t| {
            t.get("name").and_then(Value::as_str) == Some(expected_name)
                && t.get("path").and_then(Value::as_str) == Some(expected_path)
        });

        let target = matching
            .ok_or_else(|| format!("missing [[test]] for registered target {}", entry.path))?;

        let mut actual_features: Vec<&str> = target
            .get("required-features")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("target {} missing required-features", expected_name))?
            .iter()
            .map(|v| v.as_str().ok_or("non-string feature"))
            .collect::<Result<_, _>>()?;
        actual_features.sort_unstable();

        let mut expected_features = entry.required_features.to_vec();
        expected_features.sort_unstable();

        if actual_features != expected_features {
            return Err(format!(
                "target {} required-features mismatch: got {:?}, expected {:?}",
                expected_name, actual_features, expected_features
            ));
        }
    }

    let cmd_count = contributing_str.matches(MAINTAINED_CMD).count();
    if cmd_count != 1 {
        return Err(format!(
            "expected exactly 1 occurrence of {MAINTAINED_CMD} in CONTRIBUTING.md, found {cmd_count}"
        ));
    }

    let features_table = manifest
        .get("features")
        .and_then(Value::as_table)
        .ok_or_else(|| "missing [features] in Cargo.toml".to_string())?;

    let mut active = HashSet::new();
    let mut to_visit: Vec<&str> = DIRECT_FEATURES.to_vec();

    while let Some(feat) = to_visit.pop() {
        let deps = features_table
            .get(feat)
            .and_then(Value::as_array)
            .ok_or_else(|| format!("feature '{feat}' missing or not an array in [features]"))?;
        active.insert(feat);
        for dep in deps {
            if let Some(dep_name) = dep.as_str() {
                if features_table.contains_key(dep_name) && active.insert(dep_name) {
                    to_visit.push(dep_name);
                }
            }
        }
    }

    for entry in REGISTRY {
        for &req in entry.required_features {
            if !active.contains(req) {
                return Err(format!(
                    "target {} required feature '{req}' not satisfied by active features {:?}",
                    entry.cargo_name(),
                    active
                ));
            }
        }
    }

    Ok(())
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

#[test]
fn manifest_and_gate_validation_passes_on_repo() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    let contributing =
        fs::read_to_string(manifest_dir.join("CONTRIBUTING.md")).expect("read CONTRIBUTING.md");
    validate_manifest_and_gate(&cargo_toml, &contributing)
        .expect("repository manifest and contributing gate must validate");
}

#[test]
fn negative_fixtures_reject_invalid_manifest_or_gate() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    let contributing =
        fs::read_to_string(manifest_dir.join("CONTRIBUTING.md")).expect("read CONTRIBUTING.md");

    enum MutationTarget {
        Cargo,
        Contributing,
    }

    let cases: &[(&str, MutationTarget, &str, &str)] = &[
        (
            "removed registered cargo target stanza",
            MutationTarget::Cargo,
            "[[test]]\nname = \"access_render_cli\"\npath = \"e2e/access_render_cli.rs\"\nrequired-features = [\"operator\"]\n",
            "",
        ),
        (
            "removed target required feature",
            MutationTarget::Cargo,
            "name = \"access_render_cli\"\npath = \"e2e/access_render_cli.rs\"\nrequired-features = [\"operator\"]",
            "name = \"access_render_cli\"\npath = \"e2e/access_render_cli.rs\"",
        ),
        (
            "maintained command without operator",
            MutationTarget::Contributing,
            r#"`cargo test -p lumen --features "operator delegated-auth"`"#,
            r#"`cargo test -p lumen --features "delegated-auth"`"#,
        ),
        (
            "maintained command without delegated-auth",
            MutationTarget::Contributing,
            r#"`cargo test -p lumen --features "operator delegated-auth"`"#,
            r#"`cargo test -p lumen --features "operator"`"#,
        ),
    ];

    for (name, target, from, to) in cases {
        let (mut_cargo, mut_contributing) = match target {
            MutationTarget::Cargo => {
                let mutated = cargo_toml.replacen(from, to, 1);
                assert_ne!(
                    &mutated, &cargo_toml,
                    "negative fixture '{name}' failed to mutate Cargo.toml"
                );
                (mutated, contributing.clone())
            }
            MutationTarget::Contributing => {
                let mutated = contributing.replacen(from, to, 1);
                assert_ne!(
                    &mutated, &contributing,
                    "negative fixture '{name}' failed to mutate CONTRIBUTING.md"
                );
                (cargo_toml.clone(), mutated)
            }
        };

        let result = validate_manifest_and_gate(&mut_cargo, &mut_contributing);
        assert!(
            result.is_err(),
            "negative fixture '{name}' was expected to fail validation, but passed"
        );
    }
}
