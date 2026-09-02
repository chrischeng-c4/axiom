use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use syn::visit::{self, Visit};

const TRAIT: &str = "RaftStateMachine";
const FOR: &str = " for";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Gate {
    Defer,
    KeepRaft,
    Loom,
    LumenRaftWal,
    Relay,
    Tape,
    Sift,
    RaftRuntimeTests,
}

impl Gate {
    const ALL: [Self; 8] = [
        Self::Defer,
        Self::KeepRaft,
        Self::Loom,
        Self::LumenRaftWal,
        Self::Relay,
        Self::Tape,
        Self::Sift,
        Self::RaftRuntimeTests,
    ];

    const fn command(self) -> &'static str {
        match self {
            Self::Defer => "cargo build -p defer",
            Self::KeepRaft => "cargo build -p keep --features raft",
            Self::Loom => "cargo build -p loom",
            Self::LumenRaftWal => "cargo build -p lumen --features raft-wal",
            Self::Relay => "cargo build -p relay",
            Self::Tape => "cargo build -p tape",
            Self::Sift => "cargo build -p sift",
            Self::RaftRuntimeTests => "cargo test -p raft-runtime --no-run",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Registration {
    path: &'static str,
    line: String,
    gate: Gate,
}

#[derive(Clone, Copy)]
struct Site {
    path: &'static str,
    implementor: &'static str,
    gate: Gate,
}

const SITES: [Site; 14] = [
    Site {
        path: "apps/defer/src/raft.rs",
        implementor: "DeferStateMachine",
        gate: Gate::Defer,
    },
    Site {
        path: "apps/keep/src/raft.rs",
        implementor: "KvStateMachine",
        gate: Gate::KeepRaft,
    },
    Site {
        path: "apps/loom/src/raft.rs",
        implementor: "LoomSm",
        gate: Gate::Loom,
    },
    Site {
        path: "apps/lumen/src/raft_sm.rs",
        implementor: "EngineSm",
        gate: Gate::LumenRaftWal,
    },
    Site {
        path: "apps/relay/src/raft.rs",
        implementor: "RelayStateMachine",
        gate: Gate::Relay,
    },
    Site {
        path: "apps/tape/src/raft.rs",
        implementor: "TapeStateMachine",
        gate: Gate::Tape,
    },
    Site {
        path: "projects/sift/src/durability.rs",
        implementor: "SiftStateMachine",
        gate: Gate::Sift,
    },
    Site {
        path: "libs/raft-runtime/src/lib.rs",
        implementor: "CounterSm",
        gate: Gate::RaftRuntimeTests,
    },
    Site {
        path: "libs/raft-runtime/src/conformance.rs",
        implementor: "CountingSm",
        gate: Gate::RaftRuntimeTests,
    },
    Site {
        path: "libs/raft-runtime/e2e/adversarial_recovery.rs",
        implementor: "Sm",
        gate: Gate::RaftRuntimeTests,
    },
    Site {
        path: "libs/raft-runtime/e2e/group_registry.rs",
        implementor: "SequenceSm",
        gate: Gate::RaftRuntimeTests,
    },
    Site {
        path: "libs/raft-runtime/e2e/support/cluster.rs",
        implementor: "TestSm",
        gate: Gate::RaftRuntimeTests,
    },
    Site {
        path: "libs/raft-runtime/e2e/group_membership_isolation.rs",
        implementor: "NullSm",
        gate: Gate::RaftRuntimeTests,
    },
    Site {
        path: "libs/raft-runtime/e2e/snapshot_peak_memory.rs",
        implementor: "MemoryTestSm",
        gate: Gate::RaftRuntimeTests,
    },
];

type Location = (String, String);

fn registrations() -> Vec<Registration> {
    SITES
        .iter()
        .map(|site| Registration {
            path: site.path,
            line: format!("{TRAIT}{FOR} {}", site.implementor),
            gate: site.gate,
        })
        .collect()
}

fn required_gate(path: &str) -> Result<Gate, String> {
    match path {
        "apps/defer/src/raft.rs" => Ok(Gate::Defer),
        "apps/keep/src/raft.rs" => Ok(Gate::KeepRaft),
        "apps/loom/src/raft.rs" => Ok(Gate::Loom),
        "apps/lumen/src/raft_sm.rs" => Ok(Gate::LumenRaftWal),
        "apps/relay/src/raft.rs" => Ok(Gate::Relay),
        "apps/tape/src/raft.rs" => Ok(Gate::Tape),
        "projects/sift/src/durability.rs" => Ok(Gate::Sift),
        "libs/raft-runtime/src/lib.rs"
        | "libs/raft-runtime/src/conformance.rs"
        | "libs/raft-runtime/e2e/adversarial_recovery.rs"
        | "libs/raft-runtime/e2e/group_registry.rs"
        | "libs/raft-runtime/e2e/support/cluster.rs"
        | "libs/raft-runtime/e2e/group_membership_isolation.rs"
        | "libs/raft-runtime/e2e/snapshot_peak_memory.rs" => Ok(Gate::RaftRuntimeTests),
        _ => Err(format!("unknown implementor path: {path}")),
    }
}

fn validate_registry(entries: &[Registration]) -> Result<(), String> {
    if entries.len() != SITES.len() {
        return Err(format!(
            "expected {} registry rows, found {}",
            SITES.len(),
            entries.len()
        ));
    }
    let mut paths = BTreeSet::new();
    let mut gates = BTreeSet::new();
    for entry in entries {
        if !paths.insert(entry.path) {
            return Err(format!("duplicate registry path: {}", entry.path));
        }
        let expected = required_gate(entry.path)?;
        if entry.gate != expected {
            return Err(format!(
                "wrong command for {}: expected '{}', found '{}'",
                entry.path,
                expected.command(),
                entry.gate.command()
            ));
        }
        gates.insert(entry.gate);
    }
    let expected: BTreeSet<_> = Gate::ALL.into_iter().collect();
    if gates != expected {
        return Err("registry does not use every declared command".into());
    }
    Ok(())
}

fn workspace_layout() -> Result<(PathBuf, Vec<PathBuf>), String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
            "--offline",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .map_err(|error| format!("run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("parse cargo metadata: {error}"))?;
    let root = PathBuf::from(
        metadata["workspace_root"]
            .as_str()
            .ok_or("cargo metadata omitted workspace_root")?,
    );
    let members: BTreeSet<_> = metadata["workspace_members"]
        .as_array()
        .ok_or("cargo metadata omitted workspace_members")?
        .iter()
        .map(|member| {
            member
                .as_str()
                .map(str::to_owned)
                .ok_or("workspace member ID is not a string")
        })
        .collect::<Result<_, _>>()?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("cargo metadata omitted packages")?;
    let mut roots = Vec::new();
    for package in packages {
        let id = package["id"].as_str().ok_or("package ID is not a string")?;
        if !members.contains(id) {
            continue;
        }
        let manifest = package["manifest_path"]
            .as_str()
            .ok_or("package manifest_path is not a string")?;
        let directory = Path::new(manifest)
            .parent()
            .ok_or_else(|| format!("manifest has no parent: {manifest}"))?
            .to_path_buf();
        directory
            .strip_prefix(&root)
            .map_err(|error| format!("workspace package outside root {manifest}: {error}"))?;
        roots.push(directory);
    }
    roots.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then_with(|| left.cmp(right))
    });
    roots.dedup();
    let mut effective = Vec::new();
    for path in roots {
        if !effective
            .iter()
            .any(|parent: &PathBuf| path.starts_with(parent))
        {
            effective.push(path);
        }
    }
    Ok((root, effective))
}

struct AliasCollector {
    renamed: Vec<String>,
}

impl<'ast> Visit<'ast> for AliasCollector {
    fn visit_use_tree(&mut self, tree: &'ast syn::UseTree) {
        if let syn::UseTree::Rename(rename) = tree {
            if rename.ident == TRAIT && rename.rename != "_" {
                self.renamed.push(rename.rename.to_string());
            }
        }
        visit::visit_use_tree(self, tree);
    }
}

struct ImplCollector<'a> {
    path: &'a str,
    trait_names: &'a BTreeSet<String>,
    found: &'a mut Vec<Location>,
    errors: Vec<String>,
}

impl<'ast> Visit<'ast> for ImplCollector<'_> {
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let Some((_, trait_path, _)) = &item.trait_ else {
            visit::visit_item_impl(self, item);
            return;
        };
        let Some(trait_name) = trait_path.segments.last() else {
            self.errors.push("trait path has no segment".into());
            return;
        };
        if !self.trait_names.contains(&trait_name.ident.to_string()) {
            visit::visit_item_impl(self, item);
            return;
        }
        let implementor = match item.self_ty.as_ref() {
            syn::Type::Path(path)
                if path.qself.is_none()
                    && path
                        .path
                        .segments
                        .last()
                        .is_some_and(|segment| segment.arguments.is_empty()) =>
            {
                path.path
                    .segments
                    .last()
                    .expect("checked above")
                    .ident
                    .to_string()
            }
            _ => {
                self.errors.push(format!(
                    "{} has an unsupported {TRAIT} implementor type",
                    self.path
                ));
                return;
            }
        };
        self.found
            .push((self.path.to_owned(), format!("{TRAIT}{FOR} {implementor}")));
        visit::visit_item_impl(self, item);
    }
}

fn scan_source(path: &str, source: &str, found: &mut Vec<Location>) -> Result<(), String> {
    if !source.contains(TRAIT) {
        return Ok(());
    }
    let file = syn::parse_file(source).map_err(|error| format!("parse {path}: {error}"))?;
    let mut aliases = AliasCollector {
        renamed: Vec::new(),
    };
    aliases.visit_file(&file);
    if !aliases.renamed.is_empty() {
        return Err(format!(
            "{path} renames {TRAIT}; renamed imports are not inventory-safe: {}",
            aliases.renamed.join(", ")
        ));
    }
    let trait_names = BTreeSet::from([TRAIT.to_owned()]);
    let mut collector = ImplCollector {
        path,
        trait_names: &trait_names,
        found,
        errors: Vec::new(),
    };
    collector.visit_file(&file);
    if collector.errors.is_empty() {
        Ok(())
    } else {
        Err(collector.errors.join("; "))
    }
}

fn scan_rs(directory: &Path, root: &Path, found: &mut Vec<Location>) -> Result<(), String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("read {}: {error}", directory.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("read entry in {}: {error}", directory.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("read type for {}: {error}", path.display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "symlinked workspace path requires explicit inventory handling: {}",
                path.display()
            ));
        } else if file_type.is_dir() {
            let name = entry.file_name();
            if !matches!(
                name.to_str(),
                Some(".git" | "target" | "node_modules" | ".venv")
            ) {
                scan_rs(&path, root, found)?;
            }
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("rs")
        {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| format!("make {} relative: {error}", path.display()))?
                .to_str()
                .ok_or_else(|| format!("non-UTF-8 source path: {}", path.display()))?
                .to_owned();
            let source = std::fs::read_to_string(&path)
                .map_err(|error| format!("read {}: {error}", path.display()))?;
            scan_source(&relative, &source, found)?;
        }
    }
    Ok(())
}

fn observed_workspace() -> Result<Vec<Location>, String> {
    static OBSERVED: OnceLock<Result<Vec<Location>, String>> = OnceLock::new();
    OBSERVED
        .get_or_init(|| {
            let (root, directories) = workspace_layout()?;
            let mut found = Vec::new();
            for directory in directories {
                scan_rs(&directory, &root, &mut found)?;
            }
            found.sort();
            Ok(found)
        })
        .clone()
}

fn multiset(rows: impl IntoIterator<Item = Location>) -> BTreeMap<Location, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(row).or_default() += 1;
    }
    counts
}

fn expected_locations(entries: &[Registration]) -> Vec<Location> {
    entries
        .iter()
        .map(|entry| (entry.path.to_owned(), entry.line.clone()))
        .collect()
}

fn tree_is_registered(actual: &[Location], expected: &[Location]) -> Result<(), String> {
    let actual = multiset(actual.to_vec());
    let expected = multiset(expected.to_vec());
    for (site, count) in &actual {
        let registered = expected.get(site).copied().unwrap_or(0);
        if registered == 0 {
            return Err(format!("unregistered tree site: {} | {}", site.0, site.1));
        }
        if *count > registered {
            return Err(format!(
                "duplicate physical tree site: {} | {}",
                site.0, site.1
            ));
        }
    }
    Ok(())
}

fn registry_still_exists(actual: &[Location], expected: &[Location]) -> Result<(), String> {
    let actual = multiset(actual.to_vec());
    let expected = multiset(expected.to_vec());
    for (site, count) in &expected {
        let present = actual.get(site).copied().unwrap_or(0);
        if present == 0 {
            return Err(format!("stale registry row: {} | {}", site.0, site.1));
        }
        if *count > present {
            return Err(format!("duplicate registry row: {} | {}", site.0, site.1));
        }
    }
    Ok(())
}

fn expected_script() -> String {
    let mut script = concat!(
        "#!/usr/bin/env bash\n",
        "# Slow migration gate: compiles every registered RaftStateMachine implementor.\n",
        "# Each application uses its bounded package and feature gate.\n",
        "set -euo pipefail\n",
        "ROOT_DIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n",
        "cd \"$ROOT_DIR\"\n"
    )
    .to_owned();
    for gate in Gate::ALL {
        let command = gate.command();
        script.push_str(&format!("\necho \"{command}\"\n{command}\n"));
    }
    let runner = "cargo test -p raft-runtime --test implementor_build_coverage";
    script.push_str(&format!("\necho \"{runner}\"\n{runner}\n"));
    script
}

fn validate_script(raw: &str) -> Result<Vec<String>, String> {
    let expected = expected_script();
    if raw != expected {
        let expected_lines: Vec<_> = expected.lines().collect();
        let actual_lines: Vec<_> = raw.lines().collect();
        let line = (0..expected_lines.len().max(actual_lines.len()))
            .find(|index| expected_lines.get(*index) != actual_lines.get(*index))
            .unwrap_or(0);
        return Err(format!(
            "script line {} differs: expected {:?}, found {:?}",
            line + 1,
            expected_lines.get(line),
            actual_lines.get(line)
        ));
    }
    Ok(Gate::ALL
        .into_iter()
        .map(|gate| gate.command().to_owned())
        .collect())
}

fn command_sets_match(entries: &[Registration], commands: &[String]) -> Result<(), String> {
    let registered: BTreeSet<_> = entries
        .iter()
        .map(|entry| entry.gate.command().to_owned())
        .collect();
    let executed: BTreeSet<_> = commands.iter().cloned().collect();
    if executed.len() != commands.len() {
        return Err("script contains a duplicate command".into());
    }
    if let Some(command) = registered.difference(&executed).next() {
        return Err(format!("script is missing registry command: {command}"));
    }
    if let Some(command) = executed.difference(&registered).next() {
        return Err(format!("script has unregistered command: {command}"));
    }
    let expected_order: Vec<_> = Gate::ALL
        .into_iter()
        .map(|gate| gate.command().to_owned())
        .collect();
    if commands != expected_order {
        return Err("script command order or command text differs".into());
    }
    Ok(())
}

fn changed<T: Debug + PartialEq>(valid: &T, mutation: &T) {
    assert_ne!(valid, mutation, "negative fixture did not change its input");
}

fn expect_error<T: Debug>(result: Result<T, String>, needle: &str) {
    let error = result.expect_err("negative fixture must fail");
    assert!(error.contains(needle), "expected '{needle}', got '{error}'");
}

#[test]
fn all_workspace_implementors_are_registered() -> Result<(), String> {
    let registry = registrations();
    validate_registry(&registry)?;
    tree_is_registered(&observed_workspace()?, &expected_locations(&registry))
}

#[test]
fn all_registered_implementors_still_exist() -> Result<(), String> {
    let registry = registrations();
    registry_still_exists(&observed_workspace()?, &expected_locations(&registry))
}

#[test]
fn registry_and_script_commands_match() -> Result<(), String> {
    let registry = registrations();
    validate_registry(&registry)?;
    let commands = validate_script(include_str!("../../../scripts/raft-implementor-build.sh"))?;
    command_sets_match(&registry, &commands)?;

    let cargo = include_str!("../Cargo.toml");
    let stanza = concat!(
        "[[test]]\n",
        "name = \"implementor_build_coverage\"\n",
        "path = \"e2e/implementor_build_coverage.rs\""
    );
    if cargo.match_indices(stanza).count() != 1 {
        return Err("Cargo target stanza must appear exactly once".into());
    }
    let row = "| implementor compile migration | `scripts/raft-implementor-build.sh` |";
    if include_str!("../CONTRIBUTING.md")
        .lines()
        .filter(|line| *line == row)
        .count()
        != 1
    {
        return Err("CONTRIBUTING verification row must appear exactly once".into());
    }
    Ok(())
}

#[test]
fn negative_fixtures_reject_bidirectional_drift() {
    let registry = registrations();
    let valid_tree = expected_locations(&registry);

    let mut added = valid_tree.clone();
    let source =
        format!("struct UncompiledProbe;\nimpl crate::{TRAIT}\n    for UncompiledProbe {{}}\n");
    scan_source("apps/new/src/raft.rs", &source, &mut added).unwrap();
    changed(&valid_tree, &added);
    expect_error(
        tree_is_registered(&added, &valid_tree),
        "unregistered tree site",
    );

    let mut missing = valid_tree.clone();
    missing.pop();
    changed(&valid_tree, &missing);
    expect_error(
        registry_still_exists(&missing, &valid_tree),
        "stale registry row",
    );

    let mut repeated = valid_tree.clone();
    repeated.push(valid_tree[0].clone());
    changed(&valid_tree, &repeated);
    expect_error(
        tree_is_registered(&repeated, &valid_tree),
        "duplicate physical tree site",
    );

    let mut wrong_gate = registry.clone();
    wrong_gate[0].gate = Gate::LumenRaftWal;
    wrong_gate[3].gate = Gate::KeepRaft;
    changed(&registry, &wrong_gate);
    expect_error(validate_registry(&wrong_gate), "wrong command");

    let mut unknown = registry.clone();
    unknown[0].path = "apps/unknown/src/raft.rs";
    unknown[0].line = format!("{TRAIT}{FOR} UnknownSm");
    changed(&registry, &unknown);
    expect_error(validate_registry(&unknown), "unknown implementor path");

    let valid_commands: Vec<_> = Gate::ALL
        .into_iter()
        .map(|gate| gate.command().to_owned())
        .collect();
    let mut missing_command = valid_commands.clone();
    missing_command.remove(1);
    changed(&valid_commands, &missing_command);
    expect_error(
        command_sets_match(&registry, &missing_command),
        "missing registry command",
    );

    for index in 0..Gate::ALL.len() {
        let mut missing = valid_commands.clone();
        missing.remove(index);
        changed(&valid_commands, &missing);
        expect_error(command_sets_match(&registry, &missing), "command");

        let mut replaced = valid_commands.clone();
        replaced[index] = "cargo build --workspace".into();
        changed(&valid_commands, &replaced);
        expect_error(command_sets_match(&registry, &replaced), "command");

        let mut duplicated = valid_commands.clone();
        duplicated[index] = valid_commands[(index + 1) % Gate::ALL.len()].clone();
        changed(&valid_commands, &duplicated);
        expect_error(command_sets_match(&registry, &duplicated), "command");
    }

    let mut reordered = valid_commands.clone();
    reordered.swap(0, 1);
    changed(&valid_commands, &reordered);
    expect_error(command_sets_match(&registry, &reordered), "order");

    let mut workspace = valid_commands.clone();
    workspace[0] = "cargo build --workspace".into();
    changed(&valid_commands, &workspace);
    expect_error(command_sets_match(&registry, &workspace), "command");

    let mut extra_command = valid_commands.clone();
    extra_command.push("cargo check -p unrelated".into());
    changed(&valid_commands, &extra_command);
    expect_error(
        command_sets_match(&registry, &extra_command),
        "unregistered command",
    );

    let valid_script = expected_script();
    let feature_drift =
        valid_script.replace("cargo build -p keep --features raft", "cargo build -p keep");
    changed(&valid_script, &feature_drift);
    expect_error(validate_script(&feature_drift), "script line");
    let lumen_feature_drift = valid_script.replace(
        "cargo build -p lumen --features raft-wal",
        "cargo build -p lumen",
    );
    changed(&valid_script, &lumen_feature_drift);
    expect_error(validate_script(&lumen_feature_drift), "script line");

    let early_exit = valid_script.replace("cd \"$ROOT_DIR\"", "cd \"$ROOT_DIR\"\nexit 0");
    changed(&valid_script, &early_exit);
    expect_error(validate_script(&early_exit), "script line");

    let header_drift = valid_script.replace(
        "# Each application uses its bounded package and feature gate.",
        "# Each application uses a broad workspace gate.",
    );
    changed(&valid_script, &header_drift);
    expect_error(validate_script(&header_drift), "script line");

    let alias = format!("use crate::{TRAIT} as Machine;\n");
    let mut alias_sites = Vec::new();
    expect_error(
        scan_source("apps/new/src/alias.rs", &alias, &mut alias_sites),
        "renamed imports are not inventory-safe",
    );
}
