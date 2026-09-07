//! The package-manager manifest is PEP 621 `pyproject.toml`.
//!
//! Every `mamba` project command (`add`, `remove`, `lock`, `sync`, `run`,
//! `tree`, `export`) reads the same file `uv`, `pip`, and every build
//! backend read, so one project carries one manifest:
//!
//! - `[project]` owns `name`, `version`, `requires-python`, and
//!   `dependencies` (PEP 621).
//! - `[project.optional-dependencies]` owns the extras (PEP 621) —
//!   `mamba add --optional <extra>` writes there, `mamba sync --extra`
//!   selects from there.
//! - `[dependency-groups]` owns every group (PEP 735), `dev` included —
//!   the same table `uv add --dev` / `uv add --group` write. A
//!   `{ include-group = "…" }` entry is honoured when a group's members are
//!   expanded for the lock and preserved untouched on the way back out.
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
//! (`project.dependencies`, the groups and extras whose members changed,
//! `tool.mamba.sources`) inside the user's document and leaves every other
//! table, comment, and ordering byte for byte. `render` is the same writer
//! over an empty document, for a manifest that does not exist yet.
//!
//! The lock's inputs are the union of every list above: `lock_roots` is
//! what `mamba lock` resolves and `lock_inputs` is what its `input_hash`
//! digests, so a `dev` or extra requirement that changes invalidates the
//! lock the way a `[project] dependencies` change always did. A manifest
//! with no group or extra members hashes exactly as it did before groups
//! were locked, so an existing `mamba.lock` stays byte for byte.

use anyhow::{bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Key, Table, Value};

use crate::pkgmanage::add::{dep_name, normalize_name};

/// The manifest every project command reads and writes.
pub const PYPROJECT_FILE: &str = "pyproject.toml";
/// The retired manifest `mamba migrate` converts.
pub const LEGACY_MANIFEST_FILE: &str = "mamba.toml";
/// `requires-python` a fresh manifest starts with.
pub const DEFAULT_REQUIRES_PYTHON: &str = ">=3.12";
/// The PEP 735 group `--dev` names, and the one `mamba sync` installs by
/// default.
pub const DEV_GROUP: &str = "dev";

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

/// Which manifest list a dependency edit addresses: `[project] dependencies`,
/// one `[dependency-groups]` group, or one `[project.optional-dependencies]`
/// extra. `mamba add --dev` is `Group("dev")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DepTarget {
    Project,
    Group(String),
    Extra(String),
}

impl DepTarget {
    /// The target `--dev`, `--group NAME`, and `--optional EXTRA` select;
    /// none of them is `Project`.
    pub(crate) fn from_flags(dev: bool, group: Option<&str>, extra: Option<&str>) -> Self {
        if let Some(g) = group {
            DepTarget::Group(g.to_string())
        } else if let Some(e) = extra {
            DepTarget::Extra(e.to_string())
        } else if dev {
            DepTarget::Group(DEV_GROUP.to_string())
        } else {
            DepTarget::Project
        }
    }
}

/// One requirement the lock resolves from, with every manifest list that
/// declares it. A requirement declared by `[project] dependencies` has
/// `project = true`; one declared only by groups or extras does not, and
/// `mamba sync` installs its closure only when one of those is selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LockRoot {
    pub(crate) spec: String,
    pub(crate) project: bool,
    pub(crate) groups: Vec<String>,
    pub(crate) extras: Vec<String>,
}

pub(crate) struct ManifestState {
    pub(crate) project_name: String,
    pub(crate) project_version: String,
    pub(crate) python_requires: String,
    pub(crate) dependencies: Vec<String>,
    /// Every `[dependency-groups]` group's string members, `dev` included.
    pub(crate) groups: BTreeMap<String, Vec<String>>,
    /// The `{ include-group = "…" }` references each group carries, kept
    /// apart from the strings so `render_into` can leave them in place.
    pub(crate) group_includes: BTreeMap<String, Vec<String>>,
    /// Every `[project.optional-dependencies]` extra's members.
    pub(crate) extras: BTreeMap<String, Vec<String>>,
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

        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut group_includes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        match doc.get("dependency-groups").and_then(|g| g.as_table()) {
            Some(table) => {
                for (name, value) in table {
                    let Some(items) = value.as_array() else {
                        bail!("[dependency-groups] `{name}` must be an array");
                    };
                    let mut members = Vec::new();
                    let mut includes = Vec::new();
                    for item in items {
                        if let Some(s) = item.as_str() {
                            members.push(s.to_string());
                        } else if let Some(inc) = item.get("include-group").and_then(|v| v.as_str())
                        {
                            includes.push(inc.to_string());
                        }
                    }
                    groups.insert(name.clone(), members);
                    if !includes.is_empty() {
                        group_includes.insert(name.clone(), includes);
                    }
                }
            }
            None => {
                if project.contains_key("dev-dependencies") {
                    groups.insert(
                        DEV_GROUP.to_string(),
                        extract_string_list(project, "dev-dependencies"),
                    );
                }
            }
        }

        let mut extras: BTreeMap<String, Vec<String>> = BTreeMap::new();
        if let Some(table) = project
            .get("optional-dependencies")
            .and_then(|v| v.as_table())
        {
            for (name, _) in table {
                extras.insert(name.clone(), extract_string_list(table, name));
            }
        }

        let source_overrides = extract_source_overrides(&doc)?;

        Ok(ManifestState {
            project_name,
            project_version,
            python_requires,
            dependencies,
            groups,
            group_includes,
            extras,
            source_overrides,
        })
    }

    /// The `dev` group's members — the list `mamba add --dev` writes.
    #[cfg(test)]
    pub(crate) fn dev_dependencies(&self) -> &[String] {
        self.groups.get(DEV_GROUP).map(Vec::as_slice).unwrap_or(&[])
    }

    fn list_mut(&mut self, target: &DepTarget) -> &mut Vec<String> {
        match target {
            DepTarget::Project => &mut self.dependencies,
            DepTarget::Group(g) => self.groups.entry(g.clone()).or_default(),
            DepTarget::Extra(e) => self.extras.entry(e.clone()).or_default(),
        }
    }

    /// One package keeps one identity across every spelling it is written
    /// with: `AddApp`, `addapp`, `addapp==3.0` and `addapp>=2,<3` all
    /// normalize to the same PEP 503 name, so a later spelling replaces the
    /// earlier entry instead of standing beside it.
    #[cfg(test)]
    pub(crate) fn upsert_dependency(&mut self, spec: &str) {
        self.upsert_dependency_in(spec, &DepTarget::Project);
    }

    /// `upsert_dependency` against the list `target` names; the other lists
    /// are left alone, so a package can be both a project dependency and a
    /// member of a group the way PEP 735 allows.
    pub(crate) fn upsert_dependency_in(&mut self, spec: &str, target: &DepTarget) {
        let new_name = normalize_name(dep_name(spec));
        let list = self.list_mut(target);
        list.retain(|d| normalize_name(dep_name(d)) != new_name);
        list.push(spec.to_string());
        list.sort();
        list.dedup();
    }

    /// Drop `name` from every list — project dependencies, every group,
    /// every extra — and its source override.
    pub(crate) fn remove_dependency(&mut self, name: &str) {
        let name = name.trim();
        let normalized = normalize_name(name);
        self.dependencies
            .retain(|d| normalize_name(dep_name(d)) != normalized);
        for members in self.groups.values_mut() {
            members.retain(|d| normalize_name(dep_name(d)) != normalized);
        }
        for members in self.extras.values_mut() {
            members.retain(|d| normalize_name(dep_name(d)) != normalized);
        }
        self.source_overrides.remove(name);
    }

    /// Drop `name` from the one list `target` names. Returns whether an
    /// entry was removed. The source override goes only when the package
    /// is no longer declared anywhere.
    pub(crate) fn remove_dependency_in(&mut self, name: &str, target: &DepTarget) -> bool {
        let name = name.trim();
        let normalized = normalize_name(name);
        let list = self.list_mut(target);
        let before = list.len();
        list.retain(|d| normalize_name(dep_name(d)) != normalized);
        let removed = list.len() != before;
        if !self.is_declared(&normalized) {
            self.source_overrides.remove(name);
        }
        removed
    }

    fn is_declared(&self, normalized: &str) -> bool {
        self.dependencies
            .iter()
            .chain(self.groups.values().flatten())
            .chain(self.extras.values().flatten())
            .any(|d| normalize_name(dep_name(d)) == normalized)
    }

    pub(crate) fn upsert_source(&mut self, name: &str, source: ManifestSource) {
        self.source_overrides.insert(name.to_string(), source);
    }

    pub(crate) fn remove_source(&mut self, name: &str) {
        self.source_overrides.remove(name);
    }

    /// The string members of `group` plus, recursively, of every group it
    /// `include-group`s. A cycle is walked once.
    pub(crate) fn group_members(&self, group: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut visited = BTreeSet::new();
        self.collect_group(group, &mut visited, &mut out);
        out.sort();
        out.dedup();
        out
    }

    fn collect_group(&self, group: &str, visited: &mut BTreeSet<String>, out: &mut Vec<String>) {
        if !visited.insert(group.to_string()) {
            return;
        }
        if let Some(members) = self.groups.get(group) {
            out.extend(members.iter().cloned());
        }
        if let Some(includes) = self.group_includes.get(group) {
            for inc in includes {
                self.collect_group(inc, visited, out);
            }
        }
    }

    /// Every group name the manifest declares, `include-group`-only groups
    /// included.
    pub(crate) fn group_names(&self) -> Vec<String> {
        self.groups
            .keys()
            .chain(self.group_includes.keys())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Every requirement the lock resolves from, tagged with the lists that
    /// declare it and ordered by spec. Two lists naming the same spec share
    /// one root; two lists naming the same package with different specs
    /// yield two roots, which the resolver reconciles or refuses.
    pub(crate) fn lock_roots(&self) -> Vec<LockRoot> {
        fn root<'a>(by_spec: &'a mut BTreeMap<String, LockRoot>, spec: &str) -> &'a mut LockRoot {
            by_spec.entry(spec.to_string()).or_insert_with(|| LockRoot {
                spec: spec.to_string(),
                project: false,
                groups: Vec::new(),
                extras: Vec::new(),
            })
        }
        let mut by_spec: BTreeMap<String, LockRoot> = BTreeMap::new();
        for dep in &self.dependencies {
            root(&mut by_spec, dep).project = true;
        }
        for group in self.group_names() {
            for dep in self.group_members(&group) {
                let r = root(&mut by_spec, &dep);
                if !r.groups.contains(&group) {
                    r.groups.push(group.clone());
                }
            }
        }
        for (extra, members) in &self.extras {
            for dep in members {
                let r = root(&mut by_spec, dep);
                if !r.extras.contains(extra) {
                    r.extras.push(extra.clone());
                }
            }
        }
        by_spec.into_values().collect()
    }

    /// Every distinct requirement string `lock_roots` carries.
    pub(crate) fn all_dependency_specs(&self) -> Vec<String> {
        self.lock_roots().into_iter().map(|r| r.spec).collect()
    }

    /// What `input_hash` digests: the project dependencies as they always
    /// were, plus one `group:<name>:<spec>` line per group member and one
    /// `extra:<name>:<spec>` line per extra member. A manifest with no
    /// group or extra members hashes exactly as before.
    pub(crate) fn lock_inputs(&self) -> Vec<String> {
        let mut inputs: Vec<String> = self.dependencies.clone();
        for (group, members) in &self.groups {
            for dep in members {
                inputs.push(format!("group:{group}:{dep}"));
            }
        }
        for (group, includes) in &self.group_includes {
            for inc in includes {
                inputs.push(format!("group:{group}:include-group:{inc}"));
            }
        }
        for (extra, members) in &self.extras {
            for dep in members {
                inputs.push(format!("extra:{extra}:{dep}"));
            }
        }
        inputs.sort();
        inputs.dedup();
        inputs
    }

    /// A fresh `pyproject.toml` carrying exactly this state.
    #[cfg(test)]
    pub(crate) fn render(&self) -> String {
        self.render_into("")
            .expect("an empty document always parses")
    }

    /// `original` with the keys mamba owns replaced by this state.
    ///
    /// Everything mamba does not own — other `[project]` keys, groups and
    /// extras whose members did not change, `[build-system]`, `[tool.*]`,
    /// comments, key order — survives byte for byte. The legacy
    /// `python-requires` and `dev-dependencies` keys are rewritten to their
    /// PEP 621 / PEP 735 names on the way through, which is how
    /// `mamba migrate` converts.
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

        // Extras: rewrite only the ones whose members changed, create the
        // table only when a non-empty extra has nowhere to go.
        let extras_to_write: Vec<(&String, &Vec<String>)> = self
            .extras
            .iter()
            .filter(|(name, members)| {
                let current = project
                    .get("optional-dependencies")
                    .and_then(Item::as_table_like)
                    .and_then(|t| t.get(name))
                    .and_then(Item::as_array)
                    .map(array_strings);
                match current {
                    Some(existing) => &existing != *members,
                    None => !members.is_empty(),
                }
            })
            .collect();
        if !extras_to_write.is_empty() {
            let extras = ensure_table(project, "optional-dependencies", false);
            for (name, members) in extras_to_write {
                let kept = extras
                    .get(name)
                    .and_then(Item::as_array)
                    .map(non_string_values)
                    .unwrap_or_default();
                extras.insert(
                    name,
                    Item::Value(Value::Array(string_array(members, &kept))),
                );
            }
        }

        // Groups: same rule, plus the legacy `dev-dependencies` key always
        // lands as `dev` so the rename is visible even when it was empty.
        let groups_to_write: Vec<(&String, &Vec<String>)> = self
            .groups
            .iter()
            .filter(|(name, members)| {
                if name.as_str() == DEV_GROUP && had_legacy_dev {
                    return true;
                }
                let current = doc
                    .get("dependency-groups")
                    .and_then(Item::as_table_like)
                    .and_then(|t| t.get(name))
                    .and_then(Item::as_array)
                    .map(array_strings);
                match current {
                    Some(existing) => &existing != *members,
                    None => !members.is_empty(),
                }
            })
            .collect();
        if !groups_to_write.is_empty() {
            let groups = ensure_table(doc.as_table_mut(), "dependency-groups", false);
            for (name, members) in groups_to_write {
                let kept = groups
                    .get(name)
                    .and_then(Item::as_array)
                    .map(non_string_values)
                    .unwrap_or_default();
                groups.insert(
                    name,
                    Item::Value(Value::Array(string_array(members, &kept))),
                );
            }
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

fn array_strings(arr: &Array) -> Vec<String> {
    arr.iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

fn non_string_values(arr: &Array) -> Vec<Value> {
    arr.iter().filter(|v| !v.is_str()).cloned().collect()
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
        let mut groups = BTreeMap::new();
        if !dev.is_empty() {
            groups.insert(
                DEV_GROUP.to_string(),
                dev.iter().map(|s| s.to_string()).collect(),
            );
        }
        ManifestState {
            project_name: "demo".into(),
            project_version: "0.1.0".into(),
            python_requires: ">=3.12".into(),
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            groups,
            group_includes: BTreeMap::new(),
            extras: BTreeMap::new(),
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
        assert!(parsed.groups.is_empty());
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
        assert_eq!(parsed.dev_dependencies(), &["pytest".to_string()]);
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
        assert_eq!(s.dev_dependencies(), &["pytest".to_string()]);
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
        assert_eq!(back.dev_dependencies(), &["pytest".to_string()]);
    }

    #[test]
    fn include_group_entries_survive_a_dev_rewrite() {
        let original = "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nrequires-python = \">=3.12\"\ndependencies = []\n\n[dependency-groups]\nlint = [\"ruff\"]\ndev = [\n    { include-group = \"lint\" },\n    \"pytest\",\n]\n";
        let mut s = ManifestState::parse(original).unwrap();
        assert_eq!(s.dev_dependencies(), &["pytest".to_string()]);
        assert_eq!(
            s.group_includes.get("dev").unwrap(),
            &vec!["lint".to_string()]
        );
        assert_eq!(s.group_members("dev"), vec!["pytest", "ruff"]);
        s.upsert_dependency_in("mypy", &DepTarget::Group("dev".into()));
        let out = s.render_into(original).unwrap();
        assert!(out.contains("lint = [\"ruff\"]"), "{out}");
        assert!(
            out.contains(
                "dev = [\n    { include-group = \"lint\" },\n    \"mypy\",\n    \"pytest\",\n]\n"
            ),
            "{out}"
        );
    }

    #[test]
    fn groups_and_extras_are_targets_and_lock_roots() {
        let original = "[project]\nname = \"demo\"\nversion = \"0.1.0\"\nrequires-python = \">=3.12\"\ndependencies = [\n    \"core==1.0\",\n]\n\n[dependency-groups]\ndev = []\n";
        let mut s = ManifestState::parse(original).unwrap();
        s.upsert_dependency_in("pytest==8.0", &DepTarget::from_flags(true, None, None));
        s.upsert_dependency_in(
            "ruff==0.5",
            &DepTarget::from_flags(false, Some("lint"), None),
        );
        s.upsert_dependency_in(
            "core==1.0",
            &DepTarget::from_flags(false, None, Some("fast")),
        );
        s.upsert_dependency_in("orjson==3.0", &DepTarget::Extra("fast".into()));
        let out = s.render_into(original).unwrap();
        assert!(
            out.contains("[project.optional-dependencies]\nfast = [\n    \"core==1.0\",\n    \"orjson==3.0\",\n]\n"),
            "{out}"
        );
        assert!(
            out.contains("[dependency-groups]\ndev = [\n    \"pytest==8.0\",\n]\nlint = [\n    \"ruff==0.5\",\n]\n"),
            "{out}"
        );
        // The extras table lands inside [project], ahead of the groups.
        assert!(
            out.find("[project.optional-dependencies]").unwrap()
                < out.find("[dependency-groups]").unwrap(),
            "{out}"
        );

        let back = ManifestState::parse(&out).unwrap();
        let roots = back.lock_roots();
        let core = roots.iter().find(|r| r.spec == "core==1.0").unwrap();
        assert!(core.project);
        assert_eq!(core.extras, vec!["fast"]);
        assert!(core.groups.is_empty());
        let pytest = roots.iter().find(|r| r.spec == "pytest==8.0").unwrap();
        assert!(!pytest.project);
        assert_eq!(pytest.groups, vec!["dev"]);
        assert_eq!(
            back.all_dependency_specs(),
            vec!["core==1.0", "orjson==3.0", "pytest==8.0", "ruff==0.5"]
        );
        assert_eq!(
            back.lock_inputs(),
            vec![
                "core==1.0",
                "extra:fast:core==1.0",
                "extra:fast:orjson==3.0",
                "group:dev:pytest==8.0",
                "group:lint:ruff==0.5",
            ]
        );

        // Removing scoped to one list leaves the others alone; unscoped
        // removal clears every list.
        let mut scoped = ManifestState::parse(&out).unwrap();
        assert!(scoped.remove_dependency_in("core", &DepTarget::Extra("fast".into())));
        assert_eq!(scoped.dependencies, vec!["core==1.0"]);
        assert_eq!(scoped.extras["fast"], vec!["orjson==3.0"]);
        assert!(!scoped.remove_dependency_in("core", &DepTarget::Group("dev".into())));
        scoped.remove_dependency("pytest");
        assert!(scoped.groups["dev"].is_empty());
        let pruned = scoped.render_into(&out).unwrap();
        assert!(pruned.contains("dev = []\n"), "{pruned}");
        assert!(
            pruned.contains("fast = [\n    \"orjson==3.0\",\n]\n"),
            "{pruned}"
        );
    }

    #[test]
    fn a_manifest_without_groups_hashes_as_its_project_dependencies() {
        let s = state(&["b==2", "a==1"], &[]);
        assert_eq!(s.lock_inputs(), vec!["a==1", "b==2"]);
        let with_empty_dev = ManifestState::parse(
            "[project]\nname = \"d\"\nversion = \"0\"\ndependencies = [\"b==2\", \"a==1\"]\n\n[dependency-groups]\ndev = []\n",
        )
        .unwrap();
        assert_eq!(with_empty_dev.lock_inputs(), vec!["a==1", "b==2"]);
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
