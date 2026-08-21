// Dependency groups (PEP 735) + extras (PEP 631) — Tick 23.
//
// Two related but distinct mechanisms:
//
// 1. EXTRAS (PEP 631) — opt-in dependencies that ship with the package.
//    Declared under `[project.optional-dependencies]`. Selected at install
//    time via `pkg[extra1,extra2]` (PEP 508).
//
//        [project.optional-dependencies]
//        test = ["pytest", "pytest-cov"]
//        dev  = ["black", "ruff"]
//
// 2. DEPENDENCY GROUPS (PEP 735) — non-installable dev/test environments,
//    *not* bundled with the package on a build. Declared under the
//    top-level `[dependency-groups]` table. Activated by tooling
//    (`uv sync --group dev`).
//
//        [dependency-groups]
//        dev  = ["mypy", "ruff"]
//        test = ["pytest", {include-group = "dev"}]
//
// Each group entry is either a PEP 508 requirement string OR an
// `{include-group = "<name>"}` inline table that pulls in another
// group's entries — that's where the cycle hazard lives.
//
// This module owns the pure parsing + group-graph expansion. Wire-up
// into the resolver (treating the union of expanded entries as the input
// requirement set for `mamba sync --group <name>` and `mamba add --group`)
// is a follow-up tick.
//
// References:
//   - https://peps.python.org/pep-0631/ (extras)
//   - https://peps.python.org/pep-0735/ (dependency groups)

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::pkgmanage::pkgmgr::types::IndexError;

const GROUPS_URL: &str = "<pyproject.toml [dependency-groups] / [project.optional-dependencies]>";

/// Parsed `[project.optional-dependencies]`. Keys are extra names; values
/// are PEP 508 requirement strings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectExtras {
    /// Insertion order is not significant; using BTreeMap to give stable
    /// iteration for diffs/lockfile output.
    pub by_name: BTreeMap<String, Vec<String>>,
}

impl ProjectExtras {
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    pub fn requirements_for(&self, name: &str) -> Option<&[String]> {
        self.by_name.get(name).map(|v| v.as_slice())
    }
}

/// One entry inside a dependency group — either a PEP 508 requirement
/// string or a reference to another group by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupEntry {
    Requirement(String),
    IncludeGroup(String),
}

/// Parsed `[dependency-groups]` — group name → ordered entry list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyGroups {
    pub by_name: BTreeMap<String, Vec<GroupEntry>>,
}

impl DependencyGroups {
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Expand `requested` group names into a flat, deduplicated list of
    /// PEP 508 requirement strings, transitively resolving
    /// `include-group` references.
    ///
    /// Errors on:
    /// - unknown group name (typo at authoring time)
    /// - cyclic `include-group` (would loop forever)
    ///
    /// Output order is stable: depth-first by request order, with
    /// duplicates after the first occurrence dropped. This mirrors how
    /// uv emits the resolver input set so lockfile diffs stay stable.
    pub fn expand(&self, requested: &[&str]) -> Result<Vec<String>, IndexError> {
        let mut out: Vec<String> = Vec::new();
        let mut seen_req: HashSet<String> = HashSet::new();
        let mut visiting: BTreeSet<String> = BTreeSet::new();
        let mut completed: HashSet<String> = HashSet::new();

        for name in requested {
            self.expand_into(name, &mut out, &mut seen_req, &mut visiting, &mut completed)?;
        }
        Ok(out)
    }

    fn expand_into(
        &self,
        name: &str,
        out: &mut Vec<String>,
        seen_req: &mut HashSet<String>,
        visiting: &mut BTreeSet<String>,
        completed: &mut HashSet<String>,
    ) -> Result<(), IndexError> {
        if completed.contains(name) {
            return Ok(());
        }
        if visiting.contains(name) {
            return Err(IndexError::ParseError {
                url: GROUPS_URL.into(),
                detail: format!(
                    "cyclic include-group: {} (chain: {})",
                    name,
                    visiting.iter().cloned().collect::<Vec<_>>().join(" -> ")
                ),
            });
        }
        let entries = self
            .by_name
            .get(name)
            .ok_or_else(|| IndexError::ParseError {
                url: GROUPS_URL.into(),
                detail: format!("unknown dependency group: {name:?}"),
            })?;

        visiting.insert(name.to_string());
        for entry in entries {
            match entry {
                GroupEntry::Requirement(req) => {
                    if seen_req.insert(req.clone()) {
                        out.push(req.clone());
                    }
                }
                GroupEntry::IncludeGroup(other) => {
                    self.expand_into(other, out, seen_req, visiting, completed)?;
                }
            }
        }
        visiting.remove(name);
        completed.insert(name.to_string());
        Ok(())
    }
}

/// Parse `[project.optional-dependencies]` out of a pyproject TOML source.
/// Returns `Ok(ProjectExtras::default())` (empty) when the table is absent.
pub fn parse_extras(toml_src: &str) -> Result<ProjectExtras, IndexError> {
    let doc: toml::Value = toml_src.parse().map_err(|err| IndexError::ParseError {
        url: GROUPS_URL.into(),
        detail: format!("malformed TOML: {err}"),
    })?;

    let Some(table) = doc
        .get("project")
        .and_then(|p| p.get("optional-dependencies"))
    else {
        return Ok(ProjectExtras::default());
    };

    let table = table.as_table().ok_or_else(|| IndexError::ParseError {
        url: GROUPS_URL.into(),
        detail: "[project.optional-dependencies] must be a table".into(),
    })?;

    let mut by_name = BTreeMap::new();
    for (name, value) in table {
        let arr = value.as_array().ok_or_else(|| IndexError::ParseError {
            url: GROUPS_URL.into(),
            detail: format!(
                "[project.optional-dependencies].{name} must be an array of PEP 508 strings"
            ),
        })?;
        let mut entries = Vec::with_capacity(arr.len());
        for v in arr {
            let s = v.as_str().ok_or_else(|| IndexError::ParseError {
                url: GROUPS_URL.into(),
                detail: format!(
                    "[project.optional-dependencies].{name} entries must be strings, got {v:?}"
                ),
            })?;
            entries.push(s.to_string());
        }
        by_name.insert(name.clone(), entries);
    }
    Ok(ProjectExtras { by_name })
}

/// Parse `[dependency-groups]` out of a pyproject TOML source. Returns
/// `Ok(DependencyGroups::default())` (empty) when the table is absent.
pub fn parse_dependency_groups(toml_src: &str) -> Result<DependencyGroups, IndexError> {
    let doc: toml::Value = toml_src.parse().map_err(|err| IndexError::ParseError {
        url: GROUPS_URL.into(),
        detail: format!("malformed TOML: {err}"),
    })?;

    let Some(table) = doc.get("dependency-groups") else {
        return Ok(DependencyGroups::default());
    };
    let table = table.as_table().ok_or_else(|| IndexError::ParseError {
        url: GROUPS_URL.into(),
        detail: "[dependency-groups] must be a table".into(),
    })?;

    let mut by_name = BTreeMap::new();
    for (name, value) in table {
        let arr = value.as_array().ok_or_else(|| IndexError::ParseError {
            url: GROUPS_URL.into(),
            detail: format!("[dependency-groups].{name} must be an array"),
        })?;
        let mut entries = Vec::with_capacity(arr.len());
        for v in arr {
            entries.push(parse_group_entry(name, v)?);
        }
        by_name.insert(name.clone(), entries);
    }
    Ok(DependencyGroups { by_name })
}

fn parse_group_entry(group_name: &str, v: &toml::Value) -> Result<GroupEntry, IndexError> {
    if let Some(s) = v.as_str() {
        return Ok(GroupEntry::Requirement(s.to_string()));
    }
    if let Some(t) = v.as_table() {
        if let Some(include) = t.get("include-group") {
            let included = include.as_str().ok_or_else(|| IndexError::ParseError {
                url: GROUPS_URL.into(),
                detail: format!(
                    "[dependency-groups].{group_name} include-group value must be a string, got {include:?}"
                ),
            })?;
            // Reject co-mingled fields — `{include-group = "x", foo = "y"}` is
            // not a shape PEP 735 defines.
            if t.len() != 1 {
                return Err(IndexError::ParseError {
                    url: GROUPS_URL.into(),
                    detail: format!(
                        "[dependency-groups].{group_name}: include-group entry must have only the `include-group` key"
                    ),
                });
            }
            return Ok(GroupEntry::IncludeGroup(included.to_string()));
        }
        return Err(IndexError::ParseError {
            url: GROUPS_URL.into(),
            detail: format!(
                "[dependency-groups].{group_name}: inline table must have an `include-group` key"
            ),
        });
    }
    Err(IndexError::ParseError {
        url: GROUPS_URL.into(),
        detail: format!(
            "[dependency-groups].{group_name} entries must be PEP 508 strings or include-group tables, got {v:?}"
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extras_absent_yields_empty() {
        let src = r#"
[project]
name = "demo"
version = "0.1.0"
"#;
        let extras = parse_extras(src).unwrap();
        assert!(extras.is_empty());
    }

    #[test]
    fn extras_picks_up_each_named_group() {
        let src = r#"
[project]
name = "demo"
version = "0.1.0"
[project.optional-dependencies]
test = ["pytest", "pytest-cov"]
dev = ["black", "ruff"]
"#;
        let extras = parse_extras(src).unwrap();
        assert_eq!(
            extras.requirements_for("test"),
            Some(&["pytest".to_string(), "pytest-cov".to_string()][..])
        );
        assert_eq!(
            extras.requirements_for("dev"),
            Some(&["black".to_string(), "ruff".to_string()][..])
        );
    }

    #[test]
    fn extras_rejects_non_string_entries() {
        let src = r#"
[project.optional-dependencies]
test = ["pytest", 42]
"#;
        let err = parse_extras(src).unwrap_err();
        assert!(
            format!("{err}").contains("entries must be strings"),
            "got: {err}"
        );
    }

    #[test]
    fn groups_absent_yields_empty() {
        let src = r#"
[project]
name = "demo"
version = "0.1.0"
"#;
        let groups = parse_dependency_groups(src).unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn groups_picks_up_requirements_and_include_group() {
        let src = r#"
[dependency-groups]
dev  = ["mypy", "ruff"]
test = ["pytest", {include-group = "dev"}]
"#;
        let groups = parse_dependency_groups(src).unwrap();
        assert_eq!(
            groups.by_name["dev"],
            vec![
                GroupEntry::Requirement("mypy".into()),
                GroupEntry::Requirement("ruff".into()),
            ]
        );
        assert_eq!(
            groups.by_name["test"],
            vec![
                GroupEntry::Requirement("pytest".into()),
                GroupEntry::IncludeGroup("dev".into()),
            ]
        );
    }

    #[test]
    fn groups_rejects_inline_table_without_include_group() {
        let src = r#"
[dependency-groups]
test = [{some-other-key = "x"}]
"#;
        let err = parse_dependency_groups(src).unwrap_err();
        assert!(
            format!("{err}").contains("must have an `include-group` key"),
            "got: {err}"
        );
    }

    #[test]
    fn groups_rejects_include_group_with_extra_keys() {
        let src = r#"
[dependency-groups]
test = [{include-group = "dev", marker = "python_version<'3.13'"}]
"#;
        let err = parse_dependency_groups(src).unwrap_err();
        assert!(
            format!("{err}").contains("only the `include-group` key"),
            "got: {err}"
        );
    }

    #[test]
    fn expand_flattens_includes_with_dedup() {
        let groups = parse_dependency_groups(
            r#"
[dependency-groups]
base = ["a", "b"]
dev  = ["c", {include-group = "base"}, "d"]
test = [{include-group = "base"}, {include-group = "dev"}, "e"]
"#,
        )
        .unwrap();
        let expanded = groups.expand(&["test"]).unwrap();
        // Depth-first: test -> base(a, b) -> dev(c -> base[already done] -> d) -> e
        assert_eq!(expanded, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn expand_errors_on_unknown_group() {
        let groups = parse_dependency_groups(
            r#"
[dependency-groups]
dev = ["mypy"]
"#,
        )
        .unwrap();
        let err = groups.expand(&["doesnt-exist"]).unwrap_err();
        assert!(
            format!("{err}").contains("unknown dependency group"),
            "got: {err}"
        );
    }

    #[test]
    fn expand_errors_on_direct_cycle() {
        let groups = parse_dependency_groups(
            r#"
[dependency-groups]
a = [{include-group = "b"}]
b = [{include-group = "a"}]
"#,
        )
        .unwrap();
        let err = groups.expand(&["a"]).unwrap_err();
        assert!(
            format!("{err}").contains("cyclic include-group"),
            "got: {err}"
        );
    }

    #[test]
    fn expand_errors_on_self_cycle() {
        let groups = parse_dependency_groups(
            r#"
[dependency-groups]
loopy = [{include-group = "loopy"}]
"#,
        )
        .unwrap();
        let err = groups.expand(&["loopy"]).unwrap_err();
        assert!(
            format!("{err}").contains("cyclic include-group"),
            "got: {err}"
        );
    }

    #[test]
    fn expand_handles_multiple_requested_groups_with_shared_base() {
        let groups = parse_dependency_groups(
            r#"
[dependency-groups]
base = ["common"]
dev  = [{include-group = "base"}, "dev-only"]
test = [{include-group = "base"}, "test-only"]
"#,
        )
        .unwrap();
        let expanded = groups.expand(&["dev", "test"]).unwrap();
        // base only emitted once even though included from both dev and test.
        assert_eq!(expanded, vec!["common", "dev-only", "test-only"]);
    }
}
