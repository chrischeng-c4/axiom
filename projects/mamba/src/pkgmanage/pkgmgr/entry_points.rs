// PEP 621 entry-point parser + wheel `entry_points.txt` renderer (Tick 36).
//
// Bridges the gap between `uv init` (writes pyproject.toml) and
// `installer/scripts.rs` (consumes `entry_points.txt` to lay down
// `bin/<name>` launchers). The wheel build pipeline reads
// pyproject.toml's
//
//     [project.scripts]
//     httpie = "httpie.core:main"
//
//     [project.gui-scripts]
//     httpie-gui = "httpie.gui:main"
//
//     [project.entry-points."some.group"]
//     myplugin = "myplugin:hook"
//
// and projects them into the wheel's `dist-info/entry_points.txt`:
//
//     [console_scripts]
//     httpie = httpie.core:main
//
//     [gui_scripts]
//     httpie-gui = httpie.gui:main
//
//     [some.group]
//     myplugin = myplugin:hook
//
// (Note: pyproject.toml uses `gui-scripts` with a hyphen, the wheel uses
// `gui_scripts` with an underscore. Same for the canonical group rename
// `scripts` -> `console_scripts`. This module owns that translation.)
//
// Scope is intentionally narrow:
//   * pure data layer — no filesystem, no subprocesses;
//   * round-trip through `installer::scripts::parse_console_scripts` is
//     tested in-module so the contract stays honest as either side moves;
//   * PEP 503 normalization for entry-point *names* is left to the
//     caller (entry-point names are case-sensitive per PEP 621, only the
//     project name is normalized);
//   * within a group, entries come out alphabetically by name (the
//     `toml` crate's default key order) — we don't promise insertion
//     order, only determinism for reproducible wheels.

use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One parsed entry-point line. `module` and `attr` follow PEP 621:
/// the value is `module:attr` or just `module`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryPoint {
    /// Verbatim name from pyproject.toml (case-sensitive).
    pub name: String,
    /// Dotted Python module path (e.g. `httpie.core`).
    pub module: String,
    /// Optional callable inside the module (e.g. `main`). When absent,
    /// installers default to `runpy.run_module(module)`.
    pub attr: Option<String>,
}

/// Collection of entry points keyed by *wheel* group name (i.e. after
/// the `scripts` -> `console_scripts`, `gui-scripts` -> `gui_scripts`
/// rename). BTreeMap so the rendered `entry_points.txt` is
/// deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryPointSet {
    /// Group -> entry points. Per-group order matches the underlying
    /// `toml` crate's table iteration (alphabetical by key in the
    /// default build). The exact order doesn't matter for installation
    /// correctness — `entry_points.txt` is consumed as a set — but we
    /// guarantee it's deterministic so wheels are reproducible.
    pub groups: BTreeMap<String, Vec<EntryPoint>>,
}

impl EntryPointSet {
    pub fn is_empty(&self) -> bool {
        self.groups.values().all(|v| v.is_empty())
    }
    /// Total entry-point count across all groups.
    pub fn len(&self) -> usize {
        self.groups.values().map(|v| v.len()).sum()
    }
    /// Lookup a single group by its wheel-side name.
    pub fn group(&self, name: &str) -> Option<&[EntryPoint]> {
        self.groups.get(name).map(|v| v.as_slice())
    }
}

/// Parse the entry-point related sections of a pyproject.toml.
///
/// Reads (all optional):
///   * `[project.scripts]`       -> wheel group `console_scripts`
///   * `[project.gui-scripts]`   -> wheel group `gui_scripts`
///   * `[project.entry-points.GROUP]` for arbitrary `GROUP` (verbatim,
///     no rename — group name comes through as authored).
///
/// All values must be strings of the form `module` or `module:attr`.
/// Empty strings, non-string values, and entries with an empty `module`
/// segment are hard errors so authors notice typos before they ship a
/// broken wheel.
pub fn parse_pyproject_entry_points(src: &str) -> Result<EntryPointSet, IndexError> {
    let doc: toml::Value = toml::from_str(src).map_err(|e| IndexError::ParseError {
        url: "<pyproject.toml>".into(),
        detail: format!("pyproject.toml: {e}"),
    })?;

    let mut out = EntryPointSet::default();
    let Some(project) = doc.get("project") else {
        return Ok(out);
    };
    let project = project.as_table().ok_or_else(|| IndexError::ParseError {
        url: "<pyproject.toml>".into(),
        detail: "pyproject.toml: [project] must be a table".into(),
    })?;

    if let Some(scripts) = project.get("scripts") {
        let eps = parse_group_table("project.scripts", scripts)?;
        if !eps.is_empty() {
            out.groups.insert("console_scripts".to_string(), eps);
        }
    }
    if let Some(gui) = project.get("gui-scripts") {
        let eps = parse_group_table("project.gui-scripts", gui)?;
        if !eps.is_empty() {
            out.groups.insert("gui_scripts".to_string(), eps);
        }
    }
    if let Some(custom) = project.get("entry-points") {
        let table = custom.as_table().ok_or_else(|| IndexError::ParseError {
            url: "<pyproject.toml>".into(),
            detail: "pyproject.toml: [project.entry-points] must be a table".into(),
        })?;
        for (group_name, group_value) in table {
            if group_name == "console_scripts" || group_name == "gui_scripts" {
                return Err(IndexError::ParseError {
                    url: "<pyproject.toml>".into(),
                    detail: format!(
                        "pyproject.toml: [project.entry-points.{group_name}] conflicts \
                         with the reserved group; use [project.scripts] / \
                         [project.gui-scripts] instead"
                    ),
                });
            }
            let eps = parse_group_table(
                &format!("project.entry-points.{group_name}"),
                group_value,
            )?;
            if !eps.is_empty() {
                out.groups.insert(group_name.clone(), eps);
            }
        }
    }
    Ok(out)
}

fn parse_group_table(context: &str, value: &toml::Value) -> Result<Vec<EntryPoint>, IndexError> {
    let table = value.as_table().ok_or_else(|| IndexError::ParseError {
        url: "<pyproject.toml>".into(),
        detail: format!("pyproject.toml: [{context}] must be a table of name = \"module:attr\""),
    })?;
    let mut out = Vec::with_capacity(table.len());
    for (name, raw) in table {
        let target = raw.as_str().ok_or_else(|| IndexError::ParseError {
            url: "<pyproject.toml>".into(),
            detail: format!(
                "pyproject.toml: [{context}] entry {name:?} must be a string of the \
                 form \"module\" or \"module:attr\""
            ),
        })?;
        let target = target.trim();
        if target.is_empty() {
            return Err(IndexError::ParseError {
                url: "<pyproject.toml>".into(),
                detail: format!("pyproject.toml: [{context}] entry {name:?} is empty"),
            });
        }
        let ep = parse_target(name, target, context)?;
        out.push(ep);
    }
    Ok(out)
}

fn parse_target(name: &str, target: &str, context: &str) -> Result<EntryPoint, IndexError> {
    let (module, attr) = match target.split_once(':') {
        Some((m, a)) => {
            let m = m.trim();
            let a = a.trim();
            if m.is_empty() {
                return Err(IndexError::ParseError {
                    url: "<pyproject.toml>".into(),
                    detail: format!(
                        "pyproject.toml: [{context}] entry {name:?} has empty module \
                         before ':' (got {target:?})"
                    ),
                });
            }
            if a.is_empty() {
                return Err(IndexError::ParseError {
                    url: "<pyproject.toml>".into(),
                    detail: format!(
                        "pyproject.toml: [{context}] entry {name:?} has empty attr \
                         after ':' (got {target:?})"
                    ),
                });
            }
            (m.to_string(), Some(a.to_string()))
        }
        None => (target.to_string(), None),
    };
    Ok(EntryPoint {
        name: name.to_string(),
        module,
        attr,
    })
}

/// Parse a wheel-side `.dist-info/entry_points.txt` (Tick 64).
///
/// Round-trips with `render_entry_points_txt`: feeding the rendered
/// output back into this parser yields the same `EntryPointSet`.
///
/// Syntax (PEP 753 / setuptools INI dialect):
///   ```
///   [group_name]
///   name = module
///   other = module:callable
///   ```
///
/// Rules:
///   * blank lines and `# ...` / `; ...` comments are skipped
///   * `name`, `module`, and `attr` are trimmed
///   * empty `name`, empty `module`, or empty `attr` after `:` reject
///   * a duplicate group name appends entries to the existing group so
///     authors who hand-edit the file aren't surprised by silent loss
///   * a duplicate `name` within one group is a hard error — installers
///     would otherwise overwrite each other's launchers.
pub fn parse_entry_points_txt(src: &str) -> Result<EntryPointSet, IndexError> {
    let mut out = EntryPointSet::default();
    let mut current_group: Option<String> = None;

    for (lineno0, raw) in src.lines().enumerate() {
        let lineno = lineno0 + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let name = rest.strip_suffix(']').ok_or_else(|| IndexError::ParseError {
                url: "<entry_points.txt>".into(),
                detail: format!("entry_points.txt: line {lineno}: missing ']' on section header"),
            })?;
            let name = name.trim();
            if name.is_empty() {
                return Err(IndexError::ParseError {
                    url: "<entry_points.txt>".into(),
                    detail: format!("entry_points.txt: line {lineno}: empty section name"),
                });
            }
            current_group = Some(name.to_string());
            out.groups.entry(name.to_string()).or_default();
            continue;
        }
        let group = current_group.as_ref().ok_or_else(|| IndexError::ParseError {
            url: "<entry_points.txt>".into(),
            detail: format!(
                "entry_points.txt: line {lineno}: entry {line:?} appears before any [group] header"
            ),
        })?;
        let (name, target) = line.split_once('=').ok_or_else(|| IndexError::ParseError {
            url: "<entry_points.txt>".into(),
            detail: format!("entry_points.txt: line {lineno}: missing '=' in {line:?}"),
        })?;
        let name = name.trim();
        let target = target.trim();
        if name.is_empty() {
            return Err(IndexError::ParseError {
                url: "<entry_points.txt>".into(),
                detail: format!("entry_points.txt: line {lineno}: empty entry name"),
            });
        }
        if target.is_empty() {
            return Err(IndexError::ParseError {
                url: "<entry_points.txt>".into(),
                detail: format!("entry_points.txt: line {lineno}: empty target after '='"),
            });
        }
        let ep = parse_target(name, target, &format!("entry_points.txt line {lineno}"))?;
        let bucket = out.groups.get_mut(group).expect("group inserted above");
        if bucket.iter().any(|existing| existing.name == ep.name) {
            return Err(IndexError::ParseError {
                url: "<entry_points.txt>".into(),
                detail: format!(
                    "entry_points.txt: line {lineno}: duplicate entry name {:?} in [{group}]",
                    ep.name
                ),
            });
        }
        bucket.push(ep);
    }
    out.groups.retain(|_, eps| !eps.is_empty());
    Ok(out)
}

/// Render a wheel-side `dist-info/entry_points.txt`. Groups appear in
/// BTreeMap order; entries within a group keep insertion order.
///
/// Output is byte-identical to what `setuptools` / `flit_core` emit:
/// `[group]\nname = module:attr\n` blocks separated by a single blank
/// line, with a trailing newline at end of file.
pub fn render_entry_points_txt(set: &EntryPointSet) -> String {
    let mut out = String::new();
    let mut first = true;
    for (group, eps) in &set.groups {
        if eps.is_empty() {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        out.push('[');
        out.push_str(group);
        out.push(']');
        out.push('\n');
        for ep in eps {
            out.push_str(&ep.name);
            out.push_str(" = ");
            out.push_str(&ep.module);
            if let Some(attr) = &ep.attr {
                out.push(':');
                out.push_str(attr);
            }
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::installer;

    #[test]
    fn parse_empty_pyproject_yields_empty_set() {
        let set = parse_pyproject_entry_points("").unwrap();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn parse_pyproject_without_project_table_is_ok() {
        let set = parse_pyproject_entry_points("[build-system]\nrequires = []\n").unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn parse_project_scripts_maps_to_console_scripts() {
        let set = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.scripts]\nhttpie = \"httpie.core:main\"\n",
        )
        .unwrap();
        let eps = set.group("console_scripts").unwrap();
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].name, "httpie");
        assert_eq!(eps[0].module, "httpie.core");
        assert_eq!(eps[0].attr.as_deref(), Some("main"));
    }

    #[test]
    fn parse_gui_scripts_maps_to_gui_scripts() {
        let set = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.gui-scripts]\nx-gui = \"x.gui:run\"\n",
        )
        .unwrap();
        let eps = set.group("gui_scripts").unwrap();
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].name, "x-gui");
        assert_eq!(eps[0].module, "x.gui");
        assert_eq!(eps[0].attr.as_deref(), Some("run"));
    }

    #[test]
    fn parse_module_only_target_has_no_attr() {
        let set = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.scripts]\nrun = \"some.module\"\n",
        )
        .unwrap();
        let eps = set.group("console_scripts").unwrap();
        assert_eq!(eps[0].module, "some.module");
        assert!(eps[0].attr.is_none());
    }

    #[test]
    fn parse_custom_entry_points_group() {
        let set = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n\
             [project.entry-points.\"pytest11\"]\n\
             myplugin = \"my.plugin:fixture\"\n",
        )
        .unwrap();
        let eps = set.group("pytest11").unwrap();
        assert_eq!(eps[0].name, "myplugin");
        assert_eq!(eps[0].module, "my.plugin");
        assert_eq!(eps[0].attr.as_deref(), Some("fixture"));
    }

    #[test]
    fn parse_rejects_reserved_group_in_custom_table() {
        let err = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n\
             [project.entry-points.console_scripts]\n\
             foo = \"foo:main\"\n",
        )
        .unwrap_err();
        assert!(format!("{err}").contains("conflicts with the reserved group"));
    }

    #[test]
    fn parse_rejects_non_string_value() {
        let err = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.scripts]\nfoo = 42\n",
        )
        .unwrap_err();
        assert!(format!("{err}").contains("must be a string"));
    }

    #[test]
    fn parse_rejects_empty_module() {
        let err = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.scripts]\nfoo = \":main\"\n",
        )
        .unwrap_err();
        assert!(format!("{err}").contains("empty module"));
    }

    #[test]
    fn parse_rejects_empty_attr() {
        let err = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.scripts]\nfoo = \"module:\"\n",
        )
        .unwrap_err();
        assert!(format!("{err}").contains("empty attr"));
    }

    #[test]
    fn parse_rejects_empty_target() {
        let err = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.scripts]\nfoo = \"\"\n",
        )
        .unwrap_err();
        assert!(format!("{err}").contains("is empty"));
    }

    #[test]
    fn parse_rejects_non_table_scripts() {
        let err = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\nscripts = \"not a table\"\n",
        )
        .unwrap_err();
        assert!(format!("{err}").contains("must be a table"));
    }

    #[test]
    fn render_emits_canonical_layout() {
        let mut set = EntryPointSet::default();
        set.groups.insert(
            "console_scripts".into(),
            vec![EntryPoint {
                name: "httpie".into(),
                module: "httpie.core".into(),
                attr: Some("main".into()),
            }],
        );
        set.groups.insert(
            "gui_scripts".into(),
            vec![EntryPoint {
                name: "x-gui".into(),
                module: "x.gui".into(),
                attr: Some("run".into()),
            }],
        );
        let rendered = render_entry_points_txt(&set);
        assert_eq!(
            rendered,
            "[console_scripts]\nhttpie = httpie.core:main\n\n[gui_scripts]\nx-gui = x.gui:run\n"
        );
    }

    #[test]
    fn render_module_only_omits_colon() {
        let mut set = EntryPointSet::default();
        set.groups.insert(
            "console_scripts".into(),
            vec![EntryPoint {
                name: "run".into(),
                module: "some.module".into(),
                attr: None,
            }],
        );
        assert_eq!(
            render_entry_points_txt(&set),
            "[console_scripts]\nrun = some.module\n"
        );
    }

    #[test]
    fn render_skips_empty_groups() {
        let mut set = EntryPointSet::default();
        set.groups.insert("console_scripts".into(), vec![]);
        set.groups.insert(
            "gui_scripts".into(),
            vec![EntryPoint {
                name: "g".into(),
                module: "m".into(),
                attr: None,
            }],
        );
        assert_eq!(
            render_entry_points_txt(&set),
            "[gui_scripts]\ng = m\n"
        );
    }

    /// The whole point of this module: the renderer output must be
    /// accepted by the existing installer parser without round-trip
    /// drift.
    #[test]
    fn roundtrip_through_installer_scripts_parser() {
        // Synthesize a small pyproject.toml, render, then ensure the
        // installer-side parser walks back to the same console-scripts
        // we put in.
        let set = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n\
             [project.scripts]\n\
             one = \"pkg.one:run\"\n\
             two = \"pkg.two\"\n\
             \n\
             [project.gui-scripts]\n\
             gui = \"pkg.gui:main\"\n",
        )
        .unwrap();
        let rendered = render_entry_points_txt(&set);

        // Use the installer's public surface — write to a temp bin dir
        // and confirm the console_scripts list it sees matches our two
        // [project.scripts] entries (gui_scripts intentionally skipped
        // by the installer in Phase 1.3, mirroring its behavior).
        let tmp = tempfile::tempdir().unwrap();
        let names = installer::scripts::write_console_scripts(
            &rendered,
            tmp.path(),
            std::path::Path::new("/usr/bin/python3"),
        )
        .expect("write_console_scripts");
        let mut names_sorted = names;
        names_sorted.sort();
        assert_eq!(names_sorted, vec!["one".to_string(), "two".to_string()]);

        // And the file bodies the installer wrote should reflect both
        // forms (attr vs. module-only) — guarding against a future
        // regression where the renderer drops the attr.
        let one = std::fs::read_to_string(tmp.path().join("one")).unwrap();
        assert!(one.contains("from pkg.one import run"));
        let two = std::fs::read_to_string(tmp.path().join("two")).unwrap();
        assert!(two.contains("runpy.run_module('pkg.two'"));
    }

    // ---------- parse_entry_points_txt (Tick 64) ----------

    #[test]
    fn parse_entry_points_txt_empty_input_is_empty_set() {
        let set = parse_entry_points_txt("").unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn parse_entry_points_txt_blank_and_comment_lines_skipped() {
        let src = "\n# header comment\n; semicolon comment\n\n";
        let set = parse_entry_points_txt(src).unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn parse_entry_points_txt_console_scripts_with_attr() {
        let src = "[console_scripts]\nhttpie = httpie.core:main\n";
        let set = parse_entry_points_txt(src).unwrap();
        let eps = set.group("console_scripts").unwrap();
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].name, "httpie");
        assert_eq!(eps[0].module, "httpie.core");
        assert_eq!(eps[0].attr.as_deref(), Some("main"));
    }

    #[test]
    fn parse_entry_points_txt_module_only_target_has_no_attr() {
        let src = "[console_scripts]\nrun = some.module\n";
        let set = parse_entry_points_txt(src).unwrap();
        let eps = set.group("console_scripts").unwrap();
        assert_eq!(eps[0].module, "some.module");
        assert!(eps[0].attr.is_none());
    }

    #[test]
    fn parse_entry_points_txt_multiple_groups() {
        let src = "[console_scripts]\na = pkg.a:main\n\n[gui_scripts]\nb = pkg.b:main\n";
        let set = parse_entry_points_txt(src).unwrap();
        assert_eq!(set.group("console_scripts").unwrap().len(), 1);
        assert_eq!(set.group("gui_scripts").unwrap().len(), 1);
    }

    #[test]
    fn parse_entry_points_txt_round_trips_via_render() {
        // Build a set, render to text, parse back, compare.
        let mut original = EntryPointSet::default();
        original.groups.insert(
            "console_scripts".to_string(),
            vec![
                EntryPoint {
                    name: "alpha".into(),
                    module: "pkg.alpha".into(),
                    attr: Some("main".into()),
                },
                EntryPoint {
                    name: "beta".into(),
                    module: "pkg.beta".into(),
                    attr: None,
                },
            ],
        );
        original.groups.insert(
            "pytest11".to_string(),
            vec![EntryPoint {
                name: "plugin".into(),
                module: "my.plugin".into(),
                attr: Some("fixture".into()),
            }],
        );
        let rendered = render_entry_points_txt(&original);
        let parsed = parse_entry_points_txt(&rendered).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn parse_entry_points_txt_handles_whitespace_around_equals() {
        let src = "[console_scripts]\n  httpie  =   httpie.core:main  \n";
        let set = parse_entry_points_txt(src).unwrap();
        let eps = set.group("console_scripts").unwrap();
        assert_eq!(eps[0].name, "httpie");
        assert_eq!(eps[0].module, "httpie.core");
        assert_eq!(eps[0].attr.as_deref(), Some("main"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_entry_before_group_header() {
        let err = parse_entry_points_txt("foo = bar:baz\n").unwrap_err();
        assert!(format!("{err}").contains("before any [group] header"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_missing_close_bracket() {
        let err = parse_entry_points_txt("[console_scripts\n").unwrap_err();
        assert!(format!("{err}").contains("missing ']'"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_empty_section_name() {
        let err = parse_entry_points_txt("[]\nfoo = bar\n").unwrap_err();
        assert!(format!("{err}").contains("empty section name"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_missing_equals() {
        let err = parse_entry_points_txt("[console_scripts]\nhttpie httpie.core:main\n")
            .unwrap_err();
        assert!(format!("{err}").contains("missing '='"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_empty_name() {
        let err = parse_entry_points_txt("[console_scripts]\n  = mod\n").unwrap_err();
        assert!(format!("{err}").contains("empty entry name"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_empty_target() {
        let err = parse_entry_points_txt("[console_scripts]\nfoo =\n").unwrap_err();
        assert!(format!("{err}").contains("empty target"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_empty_attr_after_colon() {
        let err = parse_entry_points_txt("[console_scripts]\nfoo = mod:\n").unwrap_err();
        assert!(format!("{err}").contains("empty attr"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_empty_module_before_colon() {
        let err = parse_entry_points_txt("[console_scripts]\nfoo = :main\n").unwrap_err();
        assert!(format!("{err}").contains("empty module"));
    }

    #[test]
    fn parse_entry_points_txt_rejects_duplicate_entry_name_in_group() {
        let src = "[console_scripts]\nfoo = a:run\nfoo = b:run\n";
        let err = parse_entry_points_txt(src).unwrap_err();
        assert!(format!("{err}").contains("duplicate entry name"));
    }

    #[test]
    fn parse_entry_points_txt_duplicate_group_header_appends_entries() {
        let src = "[console_scripts]\na = pkg.a:run\n\n[console_scripts]\nb = pkg.b:run\n";
        let set = parse_entry_points_txt(src).unwrap();
        let eps = set.group("console_scripts").unwrap();
        assert_eq!(eps.len(), 2);
        assert_eq!(eps[0].name, "a");
        assert_eq!(eps[1].name, "b");
    }

    #[test]
    fn parse_entry_points_txt_empty_group_drops_from_set() {
        // A header with no entries should not appear in the result.
        let set = parse_entry_points_txt("[console_scripts]\n").unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn parse_within_group_is_deterministic_and_alphabetical() {
        // We don't promise insertion order, but the iteration order
        // must be deterministic so wheels are reproducible. The
        // underlying `toml` crate sorts table keys alphabetically.
        let set = parse_pyproject_entry_points(
            "[project]\nname = \"x\"\n\n[project.scripts]\n\
             zeta  = \"a:run\"\n\
             alpha = \"b:run\"\n\
             mid   = \"c:run\"\n",
        )
        .unwrap();
        let eps = set.group("console_scripts").unwrap();
        let names: Vec<&str> = eps.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "mid", "zeta"]);
    }
}
