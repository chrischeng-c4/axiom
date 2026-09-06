//! The package-manager manifest is PEP 621 `pyproject.toml`.
//!
//! Every `mamba` project command (`add`, `remove`, `lock`, `sync`, `run`,
//! `tree`, `export`) reads the same file `uv`, `pip`, and every build
//! backend read, so one project carries one manifest:
//!
//! - `[project]` owns `name`, `version`, `requires-python`, and
//!   `dependencies` (PEP 621).
//! - `[dependency-groups]` owns the `dev` group (PEP 735) — the same table
//!   `uv add --dev` writes.
//! - `[tool.mamba.sources]` owns mamba's per-package source overrides; the
//!   compiler's own settings (`entry_point`, `[crates]`, `[expose]`,
//!   `[build]`, `[paths]`) live beside it under `[tool.mamba]`.
//!
//! `mamba.toml` is retired. `locate` refuses a directory that still carries
//! one and names `mamba migrate`, the one-shot converter, so a stale
//! manifest is never read as if it were empty. `ManifestState::parse`
//! still understands the legacy spellings (`python-requires`,
//! `dev-dependencies`) because `migrate` parses the old file through it.
//!
//! Writes go through `render_into`, which patches only the keys mamba owns
//! (`project.dependencies`, `dependency-groups.dev`, `tool.mamba.sources`)
//! inside the user's document and leaves every other table, comment, and
//! ordering byte for byte. `render` is the same writer over an empty
//! document, for a manifest that does not exist yet.

use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Key, Table, Value};

use crate::pkgmanage::add::{dep_name, normalize_name};

/// The manifest every project command reads and writes.
pub const PYPROJECT_FILE: &str = "pyproject.toml";
/// The retired manifest `mamba migrate` converts.
pub const LEGACY_MANIFEST_FILE: &str = "mamba.toml";
/// `requires-python` a fresh manifest starts with.
pub const DEFAULT_REQUIRES_PYTHON: &str = ">=3.12";

/// The manifest path for `project_dir`, or the reason there is none.
///
/// A directory that still carries a `mamba.toml` and no `pyproject.toml`
/// is refused with the `mamba migrate` hint rather than treated as
/// uninitialised, so the old file is converted, never silently ignored.
pub fn locate(project_dir: &Path) -> Result<PathBuf> {
    let path = project_dir.join(PYPROJECT_FILE);
    if path.is_file() {
        return Ok(path);
    }
    if project_dir.join(LEGACY_MANIFEST_FILE).is_file() {
        bail!(
            "{LEGACY_MANIFEST_FILE} in {} is no longer read — run `mamba migrate` to convert it to {PYPROJECT_FILE}",
            project_dir.display()
        );
    }
    bail!(
        "no {PYPROJECT_FILE} in {} — run `mamba init` first",
        project_dir.display()
    );
}

/// Whether `project_dir` holds an unconverted legacy manifest and nothing
/// else — the only case `mamba migrate` acts on.
pub fn has_legacy_manifest_only(project_dir: &Path) -> bool {
    project_dir.join(LEGACY_MANIFEST_FILE).is_file() && !project_dir.join(PYPROJECT_FILE).is_file()
}

pub(crate) struct ManifestState {
    pub(crate) project_name: String,
    pub(crate) project_version: String,
    pub(crate) python_requires: String,
    pub(crate) dependencies: Vec<String>,
    pub(crate) dev_dependencies: Vec<String>,
    pub(crate) source_overrides: BTreeMap<String, ManifestSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ManifestSource {
    MambaProvider { provider: String },
}

impl ManifestState {
    /// Read the keys mamba owns out of a `pyproject.toml` document.
    ///
    /// The legacy `python-requires` / `dev-dependencies` spellings are
    /// accepted so `mamba migrate` can read a `mamba.toml` through the same
    /// parser; `render_into` writes only the PEP 621 / PEP 735 names.
    pub(crate) fn parse(src: &str) -> Result<Self> {
        let doc: toml::Value = src.parse().context("parse pyproject.toml")?;
        let project = doc
            .get("project")
            .and_then(|v| v.as_table())
            .context("pyproject.toml missing [project] table")?;
        let project_name = project
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("mamba-project")
            .to_string();
        let project_version = project
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();
        let python_requires = project
            .get("requires-python")
            .or_else(|| project.get("python-requires"))
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_REQUIRES_PYTHON)
            .to_string();
        let dependencies = extract_string_list(project, "dependencies");
        let dev_dependencies = match doc.get("dependency-groups").and_then(|g| g.as_table()) {
            Some(groups) => extract_string_list(groups, "dev"),
            None => extract_string_list(project, "dev-dependencies"),
        };
        let source_overrides = extract_source_overrides(&doc)?;

        Ok(ManifestState {
            project_name,
            project_version,
            python_requires,
            dependencies,
            dev_dependencies,
            source_overrides,
        })
    }

    /// One package keeps one identity across every spelling it is written
    /// with: `AddApp`, `addapp`, `addapp==3.0` and `addapp>=2,<3` all
    /// normalize to the same PEP 503 name, so a later spelling replaces the
    /// earlier entry instead of standing beside it.
    pub(crate) fn upsert_dependency(&mut self, spec: &str) {
        let new_name = normalize_name(dep_name(spec));
        self.dependencies
            .retain(|d| normalize_name(dep_name(d)) != new_name);
        self.dependencies.push(spec.to_string());
        self.dependencies.sort();
        self.dependencies.dedup();
    }

    pub(crate) fn remove_dependency(&mut self, name: &str) {
        let name = name.trim();
        let normalized = normalize_name(name);
        self.dependencies
            .retain(|d| normalize_name(dep_name(d)) != normalized);
        self.dev_dependencies
            .retain(|d| normalize_name(dep_name(d)) != normalized);
        self.source_overrides.remove(name);
    }

    pub(crate) fn upsert_source(&mut self, name: &str, source: ManifestSource) {
        self.source_overrides.insert(name.to_string(), source);
    }

    pub(crate) fn remove_source(&mut self, name: &str) {
        self.source_overrides.remove(name);
    }

    /// A fresh `pyproject.toml` carrying exactly this state.
    #[cfg(test)]
    pub(crate) fn render(&self) -> String {
        self.render_into("")
            .expect("an empty document always parses")
    }

    /// `original` with the keys mamba owns replaced by this state.
    ///
    /// Everything mamba does not own — other `[project]` keys, other
    /// dependency groups, `[build-system]`, `[tool.*]`, comments, key
    /// order — survives byte for byte. The legacy `python-requires` and
    /// `dev-dependencies` keys are rewritten to their PEP 621 / PEP 735
    /// names on the way through, which is how `mamba migrate` converts.
    pub(crate) fn render_into(&self, original: &str) -> Result<String> {
        let mut doc: DocumentMut = original.parse().context("parse pyproject.toml")?;

        let project = ensure_table(doc.as_table_mut(), "project", false);
        set_if_absent(project, "name", &self.project_name);
        set_if_absent(project, "version", &self.project_version);
        if project.contains_key("python-requires") {
            project.remove("python-requires");
        }
        set_if_absent(project, "requires-python", &self.python_requires);
        project.insert(
            "dependencies",
            Item::Value(Value::Array(string_array(&self.dependencies, &[]))),
        );
        let had_legacy_dev = project.remove("dev-dependencies").is_some();

        let has_dev_group = doc
            .get("dependency-groups")
            .and_then(Item::as_table_like)
            .map(|g| g.contains_key("dev"))
            .unwrap_or(false);
        if has_dev_group || had_legacy_dev || !self.dev_dependencies.is_empty() {
            let groups = ensure_table(doc.as_table_mut(), "dependency-groups", false);
            let kept: Vec<Value> = groups
                .get("dev")
                .and_then(Item::as_array)
                .map(|arr| arr.iter().filter(|v| !v.is_str()).cloned().collect())
                .unwrap_or_default();
            groups.insert(
                "dev",
                Item::Value(Value::Array(string_array(&self.dev_dependencies, &kept))),
            );
        }

        if self.source_overrides.is_empty() {
            remove_sources(&mut doc);
        } else {
            let tool = ensure_table(doc.as_table_mut(), "tool", true);
            let mamba = ensure_table(tool, "mamba", true);
            let mut sources = Table::new();
            for (name, source) in &self.source_overrides {
                let ManifestSource::MambaProvider { provider } = source;
                let mut entry = InlineTable::new();
                entry.insert("provider", provider.as_str().into());
                let key: Key = format!("\"{}\"", escape_toml_string(name))
                    .parse()
                    .with_context(|| format!("[tool.mamba.sources] key `{name}`"))?;
                sources.insert_formatted(&key, Item::Value(Value::InlineTable(entry)));
            }
            mamba.insert("sources", Item::Table(sources));
        }

        Ok(doc.to_string())
    }
}

/// The table at `key` under `parent`, created when absent. `implicit`
/// tables emit no header of their own when they only hold sub-tables
/// (`[tool]`, `[tool.mamba]`), so the file reads `[tool.mamba.sources]`
/// the way `uv` writes `[tool.uv.sources]`.
fn ensure_table<'a>(parent: &'a mut Table, key: &str, implicit: bool) -> &'a mut Table {
    if !parent.get(key).map(Item::is_table).unwrap_or(false) {
        let mut table = Table::new();
        table.set_implicit(implicit);
        parent.insert(key, Item::Table(table));
    }
    parent
        .get_mut(key)
        .and_then(Item::as_table_mut)
        .expect("just inserted a table")
}

fn set_if_absent(table: &mut Table, key: &str, value: &str) {
    if !table.contains_key(key) {
        table.insert(key, Item::Value(value.into()));
    }
}

/// A multi-line string array in the shape `uv add` writes:
///
/// ```toml
/// dependencies = [
///     "foo==1.0",
/// ]
/// ```
///
/// `kept` values (non-string items such as PEP 735 `include-group` tables)
/// stay in front of the strings. An empty array renders as `[]`.
fn string_array(items: &[String], kept: &[Value]) -> Array {
    let mut arr = Array::new();
    for value in kept {
        arr.push_formatted(value.clone());
    }
    for item in items {
        arr.push_formatted(Value::from(item.as_str()));
    }
    if arr.is_empty() {
        return arr;
    }
    for value in arr.iter_mut() {
        value.decor_mut().set_prefix("\n    ");
        value.decor_mut().set_suffix("");
    }
    arr.set_trailing_comma(true);
    arr.set_trailing("\n");
    arr
}

fn remove_sources(doc: &mut DocumentMut) {
    let Some(tool) = doc.get_mut("tool").and_then(Item::as_table_mut) else {
        return;
    };
    let mamba_empty = match tool.get_mut("mamba").and_then(Item::as_table_mut) {
        Some(mamba) => {
            mamba.remove("sources");
            mamba.is_empty()
        }
        None => false,
    };
    if mamba_empty {
        tool.remove("mamba");
    }
    if tool.is_empty() {
        doc.remove("tool");
    }
}

fn extract_source_overrides(doc: &toml::Value) -> Result<BTreeMap<String, ManifestSource>> {
    let mut out = BTreeMap::new();
    let Some(sources) = doc
        .get("tool")
        .and_then(|t| t.get("mamba"))
        .and_then(|m| m.get("sources"))
    else {
        return Ok(out);
    };
    let sources = sources
        .as_table()
        .context("[tool.mamba.sources] must be a table")?;
    for (name, value) in sources {
        let entry = value
            .as_table()
            .with_context(|| format!("[tool.mamba.sources] `{name}` must be an inline table"))?;
        let provider = entry
            .get("provider")
            .and_then(|v| v.as_str())
            .with_context(|| format!("[tool.mamba.sources] `{name}` missing provider"))?;
        if provider != "mamba" {
            bail!("[tool.mamba.sources] `{name}` uses unsupported provider `{provider}`");
        }
        out.insert(
            name.clone(),
            ManifestSource::MambaProvider {
                provider: provider.to_string(),
            },
        );
    }
    Ok(out)
}

fn extract_string_list(tbl: &toml::value::Table, key: &str) -> Vec<String> {
    tbl.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(deps: &[&str], dev: &[&str]) -> ManifestState {
        ManifestState {
            project_name: "demo".into(),
            project_version: "0.1.0".into(),
            python_requires: ">=3.12".into(),
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            dev_dependencies: dev.iter().map(|s| s.to_string()).collect(),
            source_overrides: BTreeMap::new(),
        }
    }

    #[test]
    fn fresh_render_is_pep621_shaped() {
        let rendered = state(&[], &[]).render();
        assert_eq!(
            rendered,
            "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nrequires-python = \">=3.12\"\ndependencies = []\n"
        );
        let parsed = ManifestState::parse(&rendered).unwrap();
        assert_eq!(parsed.python_requires, ">=3.12");
        assert!(parsed.dependencies.is_empty());
    }

    #[test]
    fn dependencies_render_one_per_line() {
        let rendered = state(&["a==1.0", "b>=2,<3"], &["pytest"]).render();
        assert!(
            rendered.contains("dependencies = [\n    \"a==1.0\",\n    \"b>=2,<3\",\n]\n"),
            "{rendered}"
        );
        assert!(
            rendered.contains("[dependency-groups]\ndev = [\n    \"pytest\",\n]\n"),
            "{rendered}"
        );
        let parsed = ManifestState::parse(&rendered).unwrap();
        assert_eq!(parsed.dependencies, vec!["a==1.0", "b>=2,<3"]);
        assert_eq!(parsed.dev_dependencies, vec!["pytest"]);
    }

    #[test]
    fn render_into_keeps_foreign_tables_and_comments() {
        let original = "# my project\n[build-system]\nrequires = [\"hatchling\"]\nbuild-backend = \"hatchling.build\"\n\n[project]\nname = \"demo\"\nversion = \"0.1.0\"\ndescription = \"kept\"\nrequires-python = \">=3.11\"\ndependencies = []\n\n[tool.ruff]\nline-length = 100\n";
        let mut s = ManifestState::parse(original).unwrap();
        assert_eq!(s.python_requires, ">=3.11");
        s.upsert_dependency("httpx==0.27.0");
        let out = s.render_into(original).unwrap();
        assert!(out.starts_with("# my project\n[build-system]\n"), "{out}");
        assert!(out.contains("description = \"kept\""), "{out}");
        assert!(out.contains("requires-python = \">=3.11\""), "{out}");
        assert!(
            out.contains("dependencies = [\n    \"httpx==0.27.0\",\n]\n"),
            "{out}"
        );
        assert!(out.contains("[tool.ruff]\nline-length = 100\n"), "{out}");
        assert!(!out.contains("[dependency-groups]"), "{out}");
    }

    #[test]
    fn legacy_keys_are_read_and_rewritten() {
        let legacy = "[project]\nname = \"old\"\nversion = \"0.1.0\"\npython-requires = \">=3.10\"\ndependencies = [\n    \"foo==1.0\",\n]\ndev-dependencies = [\n    \"pytest\",\n]\n";
        let s = ManifestState::parse(legacy).unwrap();
        assert_eq!(s.python_requires, ">=3.10");
        assert_eq!(s.dev_dependencies, vec!["pytest"]);
        let out = s.render_into(legacy).unwrap();
        assert!(!out.contains("python-requires"), "{out}");
        assert!(!out.contains("dev-dependencies"), "{out}");
        assert!(out.contains("requires-python = \">=3.10\""), "{out}");
        assert!(
            out.contains("[dependency-groups]\ndev = [\n    \"pytest\",\n]\n"),
            "{out}"
        );
        let back = ManifestState::parse(&out).unwrap();
        assert_eq!(back.dependencies, vec!["foo==1.0"]);
        assert_eq!(back.dev_dependencies, vec!["pytest"]);
    }

    #[test]
    fn include_group_entries_survive_a_dev_rewrite() {
        let original = "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nrequires-python = \">=3.12\"\ndependencies = []\n\n[dependency-groups]\nlint = [\"ruff\"]\ndev = [\n    { include-group = \"lint\" },\n    \"pytest\",\n]\n";
        let mut s = ManifestState::parse(original).unwrap();
        assert_eq!(s.dev_dependencies, vec!["pytest"]);
        s.dev_dependencies.push("mypy".into());
        let out = s.render_into(original).unwrap();
        assert!(out.contains("lint = [\"ruff\"]"), "{out}");
        assert!(
            out.contains(
                "dev = [\n    { include-group = \"lint\" },\n    \"pytest\",\n    \"mypy\",\n]\n"
            ),
            "{out}"
        );
    }

    #[test]
    fn sources_table_round_trips_and_is_pruned_when_empty() {
        let mut s = state(&["mamba-httpx-compat==0.1.0"], &[]);
        s.upsert_source(
            "mamba-httpx-compat",
            ManifestSource::MambaProvider {
                provider: "mamba".into(),
            },
        );
        let rendered = s.render();
        assert!(
            rendered.contains(
                "[tool.mamba.sources]\n\"mamba-httpx-compat\" = { provider = \"mamba\" }\n"
            ),
            "{rendered}"
        );
        assert!(!rendered.contains("[tool]\n"), "{rendered}");
        assert!(!rendered.contains("[tool.mamba]\n"), "{rendered}");
        let mut parsed = ManifestState::parse(&rendered).unwrap();
        assert_eq!(
            parsed.source_overrides.get("mamba-httpx-compat"),
            Some(&ManifestSource::MambaProvider {
                provider: "mamba".into()
            })
        );
        parsed.remove_source("mamba-httpx-compat");
        let pruned = parsed.render_into(&rendered).unwrap();
        assert!(!pruned.contains("tool"), "{pruned}");
    }

    #[test]
    fn locate_names_migrate_for_a_legacy_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let err = locate(dir.path()).unwrap_err().to_string();
        assert!(err.contains("run `mamba init` first"), "{err}");
        std::fs::write(dir.path().join(LEGACY_MANIFEST_FILE), "[project]\n").unwrap();
        let err = locate(dir.path()).unwrap_err().to_string();
        assert!(err.contains("mamba migrate"), "{err}");
        assert!(has_legacy_manifest_only(dir.path()));
        std::fs::write(dir.path().join(PYPROJECT_FILE), "[project]\n").unwrap();
        assert_eq!(locate(dir.path()).unwrap(), dir.path().join(PYPROJECT_FILE));
        assert!(!has_legacy_manifest_only(dir.path()));
    }
}
