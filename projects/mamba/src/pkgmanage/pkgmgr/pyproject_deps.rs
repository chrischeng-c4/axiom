// pyproject.toml dependency editor (Tick 54).
//
// Foundation for `uv add <pkg>` and `uv remove <pkg>`. Pure-text edits
// against a pyproject.toml body — preserves comments, indentation, and
// surrounding TOML structure where possible.
//
// What this module covers:
//   * Three dependency groups: main (`[project].dependencies`),
//     optional-extra (`[project.optional-dependencies].<extra>`), dev
//     (`[tool.uv.dev-dependencies]`).
//   * `list_dependencies` — read a snapshot of the group as
//     `Vec<String>` of PEP 508 lines.
//   * `add_dependency` — insert idempotently, sorted by PEP 503-
//     normalized name. Replaces an existing entry that matches by name
//     so updating a version pin is a single call.
//   * `remove_dependency` — delete by PEP 503-normalized name. No-op
//     when the entry doesn't exist (returns the source unchanged).
//
// What this module does NOT cover (deferred):
//   * Comment preservation on the array entry being replaced — uv keeps
//     trailing `# comment` text; we drop it on update.
//   * Inline-array (`dependencies = [...]` on one line) — we always
//     normalize to the multi-line form on first edit.
//   * Non-`[project]` PEP 621 alternatives (Poetry's `[tool.poetry]`).

use toml::Value;

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::requirements_parse::parse_one_line;
use crate::pkgmanage::pkgmgr::requirements_parse::RequirementLine;

/// Which array to act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyGroup {
    /// `[project].dependencies`.
    Main,
    /// `[project.optional-dependencies].<extra>`.
    Optional(String),
    /// `[tool.uv.dev-dependencies]`.
    Dev,
}

impl DependencyGroup {
    /// Section/header path for the inline array.
    fn path(&self) -> ArrayPath {
        match self {
            DependencyGroup::Main => ArrayPath {
                table_header: "[project]".to_string(),
                key: "dependencies".to_string(),
            },
            DependencyGroup::Optional(extra) => ArrayPath {
                table_header: "[project.optional-dependencies]".to_string(),
                key: extra.clone(),
            },
            DependencyGroup::Dev => ArrayPath {
                table_header: "[tool.uv]".to_string(),
                key: "dev-dependencies".to_string(),
            },
        }
    }
}

struct ArrayPath {
    /// TOML header introducing the table that owns the array.
    table_header: String,
    /// Key inside that table.
    key: String,
}

/// Read every entry of the named dependency group, in source order.
pub fn list_dependencies(
    toml_src: &str,
    group: &DependencyGroup,
) -> Result<Vec<String>, &'static str> {
    let doc: Value = toml::from_str(toml_src).map_err(|_| "pyproject.toml: parse failed")?;
    let arr = match group {
        DependencyGroup::Main => doc
            .get("project")
            .and_then(|p| p.get("dependencies"))
            .and_then(|v| v.as_array()),
        DependencyGroup::Optional(extra) => doc
            .get("project")
            .and_then(|p| p.get("optional-dependencies"))
            .and_then(|o| o.get(extra))
            .and_then(|v| v.as_array()),
        DependencyGroup::Dev => doc
            .get("tool")
            .and_then(|t| t.get("uv"))
            .and_then(|u| u.get("dev-dependencies"))
            .and_then(|v| v.as_array()),
    };
    Ok(arr
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default())
}

/// Add or replace the entry whose normalized name matches `requirement`'s
/// project name. Entries are kept sorted by PEP 503-normalized name.
pub fn add_dependency(
    toml_src: &str,
    group: &DependencyGroup,
    requirement: &str,
) -> Result<String, &'static str> {
    let req_name = parse_name_from_requirement(requirement)?;
    let req_key = pep503_normalize(&req_name);

    let mut current = list_dependencies(toml_src, group)?;
    current.retain(|line| {
        let name = parse_name_from_requirement(line).unwrap_or_default();
        pep503_normalize(&name) != req_key
    });
    current.push(requirement.trim().to_string());
    current.sort_by_key(|s| {
        let n = parse_name_from_requirement(s).unwrap_or_default();
        pep503_normalize(&n)
    });

    rewrite_group(toml_src, group, &current)
}

/// Remove the entry whose normalized name matches `name`. Returns the
/// source unchanged when the entry isn't present (no-op).
pub fn remove_dependency(
    toml_src: &str,
    group: &DependencyGroup,
    name: &str,
) -> Result<String, &'static str> {
    let key = pep503_normalize(name);
    let current = list_dependencies(toml_src, group)?;
    let filtered: Vec<String> = current
        .into_iter()
        .filter(|line| {
            let n = parse_name_from_requirement(line).unwrap_or_default();
            pep503_normalize(&n) != key
        })
        .collect();
    rewrite_group(toml_src, group, &filtered)
}

// ---------------------------------------------------------------------------
// Rewriter
// ---------------------------------------------------------------------------

fn rewrite_group(
    toml_src: &str,
    group: &DependencyGroup,
    entries: &[String],
) -> Result<String, &'static str> {
    let path = group.path();

    if let Some(replaced) = try_replace_existing_array(toml_src, &path, entries) {
        return Ok(replaced);
    }

    Ok(append_new_array(toml_src, &path, entries))
}

/// Locate `<path.table_header>` ... `<path.key> = [ ... ]` in the source
/// and rewrite the array in place. Returns `None` when the array is not
/// present so the caller can fall through to `append_new_array`.
fn try_replace_existing_array(
    toml_src: &str,
    path: &ArrayPath,
    entries: &[String],
) -> Option<String> {
    let lines: Vec<&str> = toml_src.split_inclusive('\n').collect();
    let mut in_target = false;
    let mut start_idx: Option<usize> = None;
    let mut end_idx: Option<usize> = None;

    for (i, raw) in lines.iter().enumerate() {
        let stripped = raw.trim_start();
        if stripped.starts_with('[') {
            in_target = section_matches(stripped, &path.table_header);
            continue;
        }
        if !in_target {
            continue;
        }
        // Match `<key> = [` on the line.
        if let Some(after_key) = stripped.strip_prefix(&path.key) {
            let after_key = after_key.trim_start();
            if !after_key.starts_with('=') {
                continue;
            }
            let after_eq = after_key.trim_start_matches('=').trim_start();
            if !after_eq.starts_with('[') {
                // Not the array form — bail and let append handle it.
                return None;
            }
            start_idx = Some(i);
            // Find the closing `]` by scanning forward, balancing nesting.
            let mut depth = 0_i32;
            for (j, line) in lines.iter().enumerate().skip(i) {
                for c in line.chars() {
                    if c == '[' {
                        depth += 1;
                    } else if c == ']' {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = Some(j);
                            break;
                        }
                    }
                }
                if end_idx.is_some() {
                    break;
                }
            }
            break;
        }
    }

    let (start, end) = (start_idx?, end_idx?);
    let mut out = String::with_capacity(toml_src.len() + 64);
    for line in lines[..start].iter() {
        out.push_str(line);
    }
    out.push_str(&render_array(&path.key, entries));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    for line in lines[end + 1..].iter() {
        out.push_str(line);
    }
    Some(out)
}

fn append_new_array(toml_src: &str, path: &ArrayPath, entries: &[String]) -> String {
    let mut out = toml_src.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    // Skip emitting an empty array as a side-effect of removing the last
    // entry from a non-existent group. The list contract requires the
    // array to be present after add/remove only when it has entries.
    if entries.is_empty() {
        return out;
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    out.push_str(&path.table_header);
    out.push('\n');
    out.push_str(&render_array(&path.key, entries));
    out
}

fn render_array(key: &str, entries: &[String]) -> String {
    if entries.is_empty() {
        return format!("{key} = []\n");
    }
    let mut out = String::new();
    out.push_str(key);
    out.push_str(" = [\n");
    for e in entries {
        out.push_str("    \"");
        out.push_str(&escape_toml_string(e));
        out.push_str("\",\n");
    }
    out.push_str("]\n");
    out
}

fn section_matches(line: &str, header: &str) -> bool {
    // Treat `[project]` and `[project] # comment` alike.
    line.trim_end_matches('\n').trim_end() == header
        || line.starts_with(header)
            && line.trim_end_matches('\n').trim_end().starts_with(header)
            && line
                .trim_end_matches('\n')
                .trim_end()
                .chars()
                .nth(header.len())
                .map(|c| c == ' ' || c == '\t' || c == '#')
                .unwrap_or(true)
}

fn escape_toml_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn parse_name_from_requirement(line: &str) -> Result<String, &'static str> {
    match parse_one_line(line) {
        Ok(RequirementLine::Package(p)) => Ok(p.name),
        _ => Err("dependency entry must be a PEP 508 package requirement"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
[project]
name = \"demo\"
version = \"0.1.0\"
dependencies = [
    \"click>=8.0\",
    \"requests==2.31.0\",
]
";

    // ---- list -----------------------------------------------------------

    #[test]
    fn lists_main_dependencies() {
        let r = list_dependencies(SAMPLE, &DependencyGroup::Main).unwrap();
        assert_eq!(r, vec!["click>=8.0".to_string(), "requests==2.31.0".into()]);
    }

    #[test]
    fn lists_missing_group_as_empty() {
        let r = list_dependencies(SAMPLE, &DependencyGroup::Dev).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn lists_optional_extra() {
        let src = "\
[project]
name = \"demo\"
version = \"0.1\"

[project.optional-dependencies]
test = [
    \"pytest>=7\",
    \"pytest-asyncio\",
]
";
        let r = list_dependencies(src, &DependencyGroup::Optional("test".into())).unwrap();
        assert_eq!(r, vec!["pytest>=7".to_string(), "pytest-asyncio".into()]);
    }

    // ---- add ------------------------------------------------------------

    #[test]
    fn add_new_dependency_keeps_alphabetical_order() {
        let r = add_dependency(SAMPLE, &DependencyGroup::Main, "rich>=13").unwrap();
        let listed = list_dependencies(&r, &DependencyGroup::Main).unwrap();
        assert_eq!(
            listed,
            vec![
                "click>=8.0".to_string(),
                "requests==2.31.0".into(),
                "rich>=13".into(),
            ]
        );
    }

    #[test]
    fn add_replaces_existing_entry_by_normalized_name() {
        let r = add_dependency(SAMPLE, &DependencyGroup::Main, "Requests>=2.32").unwrap();
        let listed = list_dependencies(&r, &DependencyGroup::Main).unwrap();
        assert_eq!(
            listed,
            vec!["click>=8.0".to_string(), "Requests>=2.32".into()]
        );
    }

    #[test]
    fn add_creates_section_when_absent() {
        let src = "[project]\nname = \"demo\"\nversion = \"0.1\"\n";
        let r = add_dependency(src, &DependencyGroup::Dev, "ruff").unwrap();
        let dev = list_dependencies(&r, &DependencyGroup::Dev).unwrap();
        assert_eq!(dev, vec!["ruff".to_string()]);
        assert!(r.contains("[tool.uv]"));
        assert!(r.contains("dev-dependencies = ["));
    }

    #[test]
    fn add_into_optional_dependencies_creates_extra_group() {
        let src = "[project]\nname = \"demo\"\nversion = \"0.1\"\n";
        let r = add_dependency(src, &DependencyGroup::Optional("test".into()), "pytest").unwrap();
        // The synthetic header path is `[project.optional-dependencies]`
        // with the key being the extra's name.
        let listed = list_dependencies(&r, &DependencyGroup::Optional("test".into())).unwrap();
        assert_eq!(listed, vec!["pytest".to_string()]);
    }

    #[test]
    fn add_idempotent_on_repeat() {
        let a = add_dependency(SAMPLE, &DependencyGroup::Main, "rich>=13").unwrap();
        let b = add_dependency(&a, &DependencyGroup::Main, "rich>=13").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn add_rejects_malformed_requirement() {
        let err = add_dependency(SAMPLE, &DependencyGroup::Main, "@@@bad@@@").unwrap_err();
        assert!(err.contains("PEP 508"));
    }

    // ---- remove ---------------------------------------------------------

    #[test]
    fn remove_existing_dependency() {
        let r = remove_dependency(SAMPLE, &DependencyGroup::Main, "click").unwrap();
        let listed = list_dependencies(&r, &DependencyGroup::Main).unwrap();
        assert_eq!(listed, vec!["requests==2.31.0".to_string()]);
    }

    #[test]
    fn remove_matches_normalized_name() {
        let r = remove_dependency(SAMPLE, &DependencyGroup::Main, "Click").unwrap();
        let listed = list_dependencies(&r, &DependencyGroup::Main).unwrap();
        assert_eq!(listed, vec!["requests==2.31.0".to_string()]);
    }

    #[test]
    fn remove_missing_is_noop() {
        let r = remove_dependency(SAMPLE, &DependencyGroup::Main, "nope").unwrap();
        let listed = list_dependencies(&r, &DependencyGroup::Main).unwrap();
        // List preserved (sorted by name on rewrite, but here both
        // existing entries already came in alphabetical order).
        assert_eq!(
            listed,
            vec!["click>=8.0".to_string(), "requests==2.31.0".into()]
        );
    }

    #[test]
    fn remove_last_entry_leaves_empty_array() {
        let r1 = remove_dependency(SAMPLE, &DependencyGroup::Main, "click").unwrap();
        let r2 = remove_dependency(&r1, &DependencyGroup::Main, "requests").unwrap();
        let listed = list_dependencies(&r2, &DependencyGroup::Main).unwrap();
        assert!(listed.is_empty());
        assert!(r2.contains("dependencies = []"));
    }

    // ---- round-trip / shape --------------------------------------------

    #[test]
    fn round_trip_through_toml_parses() {
        let r = add_dependency(SAMPLE, &DependencyGroup::Main, "rich>=13").unwrap();
        // Must still be valid TOML.
        let _: toml::Value = toml::from_str(&r).unwrap();
    }

    #[test]
    fn preserves_other_table_content() {
        let src = "\
[project]
name = \"demo\"
version = \"0.1.0\"
authors = [{ name = \"Alice\" }]
dependencies = [\"click\"]

[tool.other]
key = \"keep me\"
";
        let r = add_dependency(src, &DependencyGroup::Main, "rich").unwrap();
        assert!(r.contains("[tool.other]"));
        assert!(r.contains("key = \"keep me\""));
        assert!(r.contains("authors"));
    }
}
