// `mamba migrate` — the one-shot conversion of a retired `mamba.toml` into
// PEP 621 `pyproject.toml`.
//
// A project written by an older `mamba init` carries its dependencies in
// `mamba.toml`; every project command now reads only `pyproject.toml`
// and refuses the old file by name (`manifest::pyproject::locate`). This
// command is the bridge: it reads the old manifest once, writes the new
// one, and deletes the old file so the refusal never fires again.
//
//   - `[project] name / version / dependencies` carry over unchanged;
//     `python-requires` becomes `requires-python`; `dev-dependencies`
//     becomes `[dependency-groups] dev` (PEP 735).
//   - `[tool.mamba.sources]` carries over unchanged.
//   - The compiler's own settings (`entry_point`, `[crates]`, `[expose]`,
//     `[build]`, `[paths]`) move under `[tool.mamba]`, where
//     `MambaConfig::discover` reads them from a `pyproject.toml`.
//   - Every other top-level key is copied under `[tool.mamba]` too, so
//     nothing the old file said is dropped on the floor.
//   - A directory with no `mamba.toml`, or one that already has a
//     `pyproject.toml`, is refused: there is nothing to convert, or the
//     conversion would overwrite a file the user owns.

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table};

use crate::pkgmanage::add::{atomic_write, ManifestState};
use crate::pkgmanage::manifest::pyproject::{LEGACY_MANIFEST_FILE, PYPROJECT_FILE};

/// Compiler-side tables and keys that live under `[tool.mamba]` in a
/// `pyproject.toml`. Everything else at the top level of the old file
/// that is not `project` or `tool` moves there as well.
const COMPILER_KEYS: &[&str] = &["entry_point", "crates", "expose", "build", "paths"];

pub fn cmd_migrate(sub: &ArgMatches) -> Result<()> {
    let project_dir: PathBuf = match sub.get_one::<String>("path") {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().context("read current directory")?,
    };
    let (new_path, removed) = migrate_dir(&project_dir)?;
    eprintln!(
        "mamba: wrote {} and removed {}",
        new_path.display(),
        removed.display()
    );
    Ok(())
}

/// Convert `<project_dir>/mamba.toml` into `<project_dir>/pyproject.toml`
/// and delete the old file. Returns the two paths.
pub fn migrate_dir(project_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let legacy_path = project_dir.join(LEGACY_MANIFEST_FILE);
    let new_path = project_dir.join(PYPROJECT_FILE);
    if new_path.exists() {
        bail!(
            "{} already exists — nothing to migrate (delete {LEGACY_MANIFEST_FILE} by hand once its contents are in {PYPROJECT_FILE})",
            new_path.display()
        );
    }
    if !legacy_path.is_file() {
        bail!(
            "no {LEGACY_MANIFEST_FILE} in {} — nothing to migrate",
            project_dir.display()
        );
    }
    let legacy_src = fs::read_to_string(&legacy_path)
        .with_context(|| format!("read {}", legacy_path.display()))?;
    let rendered = convert(&legacy_src)?;
    atomic_write(&new_path, rendered.as_bytes())?;
    fs::remove_file(&legacy_path).with_context(|| format!("remove {}", legacy_path.display()))?;
    Ok((new_path, legacy_path))
}

/// The `pyproject.toml` text for a legacy `mamba.toml` document.
pub fn convert(legacy_src: &str) -> Result<String> {
    let mut doc: DocumentMut = legacy_src
        .parse()
        .with_context(|| format!("parse {LEGACY_MANIFEST_FILE}"))?;

    // A compiler-only manifest had no [project]; PEP 621 requires one, and
    // `render_into` fills the defaults a fresh `mamba init` would write.
    // It is inserted before anything moves so it serializes first.
    if !doc.get("project").map(Item::is_table).unwrap_or(false) {
        let mut project = Table::new();
        project.set_implicit(false);
        doc.insert("project", Item::Table(project));
    }

    // Everything the package manager does not own moves under
    // [tool.mamba]; `[project]` stays, and an existing `[tool]` merges.
    let moved: Vec<(String, Item)> = doc
        .as_table()
        .iter()
        .filter(|(key, _)| *key != "project" && *key != "tool")
        .map(|(key, item)| (key.to_string(), item.clone()))
        .collect();
    for (key, _) in &moved {
        doc.remove(key);
    }
    if let Some(project) = doc.get_mut("project").and_then(Item::as_table_mut) {
        if let Some(entry_point) = project.remove("entry_point") {
            let tool_mamba = tool_mamba(&mut doc);
            if !tool_mamba.contains_key("entry_point") {
                tool_mamba.insert("entry_point", entry_point);
            }
        }
    }
    if !moved.is_empty() {
        let tool_mamba = tool_mamba(&mut doc);
        for (key, item) in moved {
            if tool_mamba.contains_key(&key) {
                continue;
            }
            let item = match item {
                Item::Table(mut t) => {
                    if !COMPILER_KEYS.contains(&key.as_str()) {
                        t.set_implicit(false);
                    }
                    // A header that followed the file's first line has no
                    // blank line above it; every moved table gets one.
                    if t.decor().prefix().map_or(true, |p| p.as_str() == Some("")) {
                        t.decor_mut().set_prefix("\n");
                    }
                    Item::Table(t)
                }
                other => other,
            };
            tool_mamba.insert(&key, item);
        }
    }
    let intermediate = doc.to_string();

    // The package-manager keys are rewritten by the same writer every
    // other command uses, which is what renames `python-requires` and
    // `dev-dependencies` to their PEP 621 / PEP 735 spellings.
    let state = ManifestState::parse(&intermediate)?;
    state.render_into(&intermediate)
}

fn tool_mamba(doc: &mut DocumentMut) -> &mut Table {
    let root = doc.as_table_mut();
    if !root.get("tool").map(Item::is_table).unwrap_or(false) {
        let mut tool = Table::new();
        tool.set_implicit(true);
        root.insert("tool", Item::Table(tool));
    }
    let tool = root
        .get_mut("tool")
        .and_then(Item::as_table_mut)
        .expect("tool is a table");
    if !tool.get("mamba").map(Item::is_table).unwrap_or(false) {
        let mut mamba = Table::new();
        mamba.set_implicit(true);
        tool.insert("mamba", Item::Table(mamba));
    }
    tool.get_mut("mamba")
        .and_then(Item::as_table_mut)
        .expect("tool.mamba is a table")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY: &str = "[project]\nname = \"old\"\nversion = \"0.2.0\"\npython-requires = \">=3.11\"\ndependencies = [\n    \"foo==1.0\",\n]\ndev-dependencies = [\n    \"pytest\",\n]\n\n[tool.mamba.sources]\n\"mamba-httpx-compat\" = { provider = \"mamba\" }\n";

    #[test]
    fn package_manager_keys_convert_to_pep621() {
        let out = convert(LEGACY).unwrap();
        assert!(
            out.contains("[project]\nname = \"old\"\nversion = \"0.2.0\"\n"),
            "{out}"
        );
        assert!(out.contains("requires-python = \">=3.11\""), "{out}");
        assert!(!out.contains("python-requires"), "{out}");
        assert!(!out.contains("dev-dependencies"), "{out}");
        assert!(
            out.contains("dependencies = [\n    \"foo==1.0\",\n]\n"),
            "{out}"
        );
        assert!(
            out.contains("[dependency-groups]\ndev = [\n    \"pytest\",\n]\n"),
            "{out}"
        );
        assert!(
            out.contains(
                "[tool.mamba.sources]\n\"mamba-httpx-compat\" = { provider = \"mamba\" }\n"
            ),
            "{out}"
        );
        let state = ManifestState::parse(&out).unwrap();
        assert_eq!(state.dependencies, vec!["foo==1.0"]);
        assert_eq!(state.dev_dependencies(), &["pytest".to_string()]);
        assert_eq!(state.python_requires, ">=3.11");
    }

    #[test]
    fn compiler_settings_move_under_tool_mamba() {
        let legacy = "[project]\nname = \"app\"\nversion = \"0.1.0\"\nentry_point = \"src/main.py\"\n\n[crates.cclab-schema-mamba]\nversion = \"0.1.0\"\nexpose = [\"BaseModel\"]\n\n[build]\ntarget = \"x\"\n";
        let out = convert(legacy).unwrap();
        assert!(!out.contains("\n[crates."), "{out}");
        assert!(
            out.contains("[tool.mamba]\nentry_point = \"src/main.py\"\n"),
            "{out}"
        );
        assert!(out.contains("[tool.mamba.crates.cclab-schema-mamba]\nversion = \"0.1.0\"\nexpose = [\"BaseModel\"]\n"), "{out}");
        assert!(
            out.contains("[tool.mamba.build]\ntarget = \"x\"\n"),
            "{out}"
        );
        let cfg = crate::pkgmanage::manifest::MambaConfig::from_pyproject_str(&out)
            .unwrap()
            .expect("[tool.mamba] present");
        assert_eq!(cfg.entry_point(), Some("src/main.py"));
        assert!(cfg.crates.contains_key("cclab-schema-mamba"));
        assert_eq!(cfg.project.name, "app");
    }

    #[test]
    fn flat_compiler_config_converts_without_a_project_table() {
        let legacy = "entry_point = \"app.py\"\n[crates]\ncclab-schema-mamba = \"0.1.0\"\n";
        let out = convert(legacy).unwrap();
        assert!(
            out.contains("[tool.mamba]\nentry_point = \"app.py\"\n"),
            "{out}"
        );
        assert!(
            out.contains("[tool.mamba.crates]\ncclab-schema-mamba = \"0.1.0\"\n"),
            "{out}"
        );
        assert!(
            out.contains("[project]\nname = \"mamba-project\"\n"),
            "{out}"
        );
    }

    #[test]
    fn migrate_dir_writes_the_new_file_and_removes_the_old() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(LEGACY_MANIFEST_FILE), LEGACY).unwrap();
        let (new_path, old_path) = migrate_dir(tmp.path()).unwrap();
        assert!(new_path.is_file());
        assert!(!old_path.exists());
        let second = migrate_dir(tmp.path()).unwrap_err().to_string();
        assert!(second.contains("already exists"), "{second}");
    }

    #[test]
    fn migrate_dir_refuses_when_there_is_nothing_to_convert() {
        let tmp = tempfile::tempdir().unwrap();
        let err = migrate_dir(tmp.path()).unwrap_err().to_string();
        assert!(err.contains("nothing to migrate"), "{err}");
    }
}
