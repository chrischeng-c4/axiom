//! Capability TYPE -> production-required EC dimensions.
//!
//! A capability's *type* is a structural classification (AgentFirst / Service /
//! Devops / DeveloperTool / RuntimeTool / SecurityTool) that determines which
//! external-contract (EC) dimensions are
//! production-required. This is deliberately decoupled from maturity/env (vat):
//! the type decides *which* EC dimensions are required for production; maturity
//! decides only *whether* a given gate is verified/runnable, and must never
//! flip `required_for_production`.
//!
//! The PRIMARY source of a capability's type is the explicit `Type:` /
//! `Capability Type:` field in the project README's canonical field-style
//! capability contract under `## Capabilities`. Readers (`aw ec`, deriving
//! `required_for_production`) resolve it via [`load_capability_types`] and
//! [`load_capability_types_from_readme`]. Legacy Capability Index pillar groups
//! such as `**Pillar — agent-first**` remain readable only as migration
//! fallback; explicit section-local fields always win because they are local to
//! the product promise. An optional
//! `.aw/capability-types.toml` `[capability_types]` table is a migration
//! fallback for capabilities that do not yet carry an explicit README type:
//!
//! ```toml
//! [capability_types]
//! my-capability = "Service"
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Relative path (from a project root) to the capability-type binding file.
pub const CAPABILITY_TYPES_REL: &str = ".aw/capability-types.toml";

/// Structural classification of a product capability.
///
/// The variant determines which EC dimensions are production-required (see
/// [`CapabilityType::required_ec_dimensions`]). Serde (de)serializes the enum as
/// exact human-authored strings so the on-disk `.aw/capability-types.toml`
/// remains readable. The enum is extensible: add a variant plus its
/// required-dimension set and the serde rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Agent-facing capability: only behavioral correctness is production-required.
    #[serde(rename = "AgentFirst")]
    AgentFirst,
    /// Externally-served capability: behavior + efficiency + security + stability.
    #[serde(rename = "Service")]
    Service,
    /// Operational/devops capability: behavior + stability.
    #[serde(rename = "Devops")]
    Devops,
    /// Developer-facing toolchain capability: behavior + efficiency + stability.
    #[serde(rename = "DeveloperTool")]
    DeveloperTool,
    /// Runtime/tool execution capability: behavior + efficiency + stability.
    #[serde(rename = "RuntimeTool")]
    RuntimeTool,
    /// Security evidence capability: behavior + security + stability.
    #[serde(rename = "SecurityTool")]
    SecurityTool,
}

impl CapabilityType {
    /// The exact string used on disk and in HITL choices.
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityType::AgentFirst => "AgentFirst",
            CapabilityType::Service => "Service",
            CapabilityType::Devops => "Devops",
            CapabilityType::DeveloperTool => "DeveloperTool",
            CapabilityType::RuntimeTool => "RuntimeTool",
            CapabilityType::SecurityTool => "SecurityTool",
        }
    }

    /// Parse a CLI / HITL-answer string into a [`CapabilityType`].
    ///
    /// Accepts the canonical names (`AgentFirst` / `Service` / `Devops` /
    /// `DeveloperTool` / `RuntimeTool` / `SecurityTool`, case-insensitively) as
    /// well as the HITL choice ids (`agent_first` / `service` / `devops` /
    /// `developer_tool` / `runtime_tool` / `security_tool`).
    pub fn from_cli_str(value: &str) -> Result<CapabilityType> {
        match value.trim().to_ascii_lowercase().as_str() {
            "agentfirst" | "agent_first" | "agent-first" => Ok(CapabilityType::AgentFirst),
            "service" => Ok(CapabilityType::Service),
            "devops" => Ok(CapabilityType::Devops),
            "developertool" | "developer_tool" | "developer-tool" | "developer" => {
                Ok(CapabilityType::DeveloperTool)
            }
            "runtimetool" | "runtime_tool" | "runtime-tool" | "runtime" => {
                Ok(CapabilityType::RuntimeTool)
            }
            "securitytool" | "security_tool" | "security-tool" | "security" => {
                Ok(CapabilityType::SecurityTool)
            }
            other => anyhow::bail!(
                "unknown capability type `{other}`; expected AgentFirst, Service, Devops, DeveloperTool, RuntimeTool, or SecurityTool"
            ),
        }
    }
}

/// The EC dimensions that are production-required for a capability of this type.
///
/// Returned dimensions are a subset of [`crate::cli::ec::ec_categories`]
/// (`behavior` / `efficiency` / `security` / `stability`). The lists are sorted
/// and contain no duplicates so callers may treat them as a set.
///
/// - `AgentFirst`    -> `{behavior}`
/// - `Service`       -> `{behavior, efficiency, security, stability}`
/// - `Devops`        -> `{behavior, stability}`
/// - `DeveloperTool` -> `{behavior, efficiency, stability}`
/// - `RuntimeTool`   -> `{behavior, efficiency, stability}`
/// - `SecurityTool`  -> `{behavior, security, stability}`
pub fn required_ec_dimensions(capability_type: &CapabilityType) -> &'static [&'static str] {
    match capability_type {
        CapabilityType::AgentFirst => &["behavior"],
        CapabilityType::Service => &["behavior", "efficiency", "security", "stability"],
        CapabilityType::Devops => &["behavior", "stability"],
        CapabilityType::DeveloperTool | CapabilityType::RuntimeTool => {
            &["behavior", "efficiency", "stability"]
        }
        CapabilityType::SecurityTool => &["behavior", "security", "stability"],
    }
}

/// `true` if an EC `category` is production-required for the given type.
pub fn category_is_required_for_type(capability_type: &CapabilityType, category: &str) -> bool {
    required_ec_dimensions(capability_type).contains(&category)
}

/// On-disk shape of `.aw/capability-types.toml`.
#[derive(Debug, Default, Deserialize)]
struct CapabilityTypesFile {
    #[serde(default)]
    capability_types: BTreeMap<String, CapabilityType>,
}

/// Absolute path to `.aw/capability-types.toml` under `project_root`.
pub fn capability_types_path(project_root: &Path) -> PathBuf {
    project_root.join(CAPABILITY_TYPES_REL)
}

/// Map of `capability_id -> CapabilityType` derived from the README capability
/// map. Canonical README contracts use explicit field-style `Type:` /
/// `Capability Type:` lines under each capability heading. Legacy Capability
/// Index pillar groups (`**Pillar — <name>**`) are still parsed as migration
/// fallback: each capability row's first table cell is its `capability_id`, and
/// the pillar name maps to a type (`agent-first -> AgentFirst`,
/// `serve… -> Service`, `devops… -> Devops`, etc.). Explicit field-style
/// section values override pillar fallback. A missing README yields an empty
/// map.
pub fn load_capability_types_from_readme(
    readme_path: &Path,
) -> Result<BTreeMap<String, CapabilityType>> {
    let content = match std::fs::read_to_string(readme_path) {
        Ok(c) => c,
        Err(_) => return Ok(BTreeMap::new()),
    };
    let mut map = BTreeMap::new();
    let mut current: Option<CapabilityType> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("**Pillar — ") {
            let name = rest
                .split("**")
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            current = if name.starts_with("agent-first") {
                Some(CapabilityType::AgentFirst)
            } else if name.starts_with("serve") {
                Some(CapabilityType::Service)
            } else if name.starts_with("devops") {
                Some(CapabilityType::Devops)
            } else if name.starts_with("developer") || name.starts_with("tool") {
                Some(CapabilityType::DeveloperTool)
            } else if name.starts_with("runtime") {
                Some(CapabilityType::RuntimeTool)
            } else if name.starts_with("security") {
                Some(CapabilityType::SecurityTool)
            } else {
                None
            };
        } else if trimmed.starts_with("**") || trimmed.starts_with('#') {
            current = None;
        } else if let Some(ty) = current {
            if let Some(inner) = trimmed.strip_prefix('|') {
                let first = inner.split('|').next().unwrap_or("").trim();
                let is_separator = !first.is_empty() && first.chars().all(|c| c == '-' || c == ':');
                if !first.is_empty() && !first.eq_ignore_ascii_case("Capability") && !is_separator {
                    map.insert(first.to_string(), ty);
                }
            }
        }
    }
    for (id, ty) in explicit_capability_types_from_readme(&content) {
        map.insert(id, ty);
    }
    Ok(map)
}

fn explicit_capability_types_from_readme(content: &str) -> BTreeMap<String, CapabilityType> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut map = BTreeMap::new();
    let mut idx = 0;
    while idx < lines.len() {
        if !is_markdown_heading(lines[idx]) {
            idx += 1;
            continue;
        }
        let block_end = next_markdown_heading(&lines, idx + 1).unwrap_or(lines.len());
        if let Some((id, capability_type)) =
            explicit_capability_type_from_block(&lines[idx + 1..block_end])
        {
            map.insert(id, capability_type);
        }
        idx = block_end;
    }
    map
}

fn explicit_capability_type_from_block(lines: &[&str]) -> Option<(String, CapabilityType)> {
    let mut id = None;
    let mut capability_type = None;
    let mut cursor = 0;
    while cursor < lines.len() {
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            cursor += 1;
            continue;
        }
        if let Some((key, value)) = split_markdown_field(trimmed) {
            match normalize_key(key).as_str() {
                "id" | "capabilityid" => id = Some(value.trim().to_string()),
                "type" | "capabilitytype" => {
                    capability_type = CapabilityType::from_cli_str(value).ok();
                }
                _ => {}
            }
        }
        if let Some((headers, rows, next_cursor)) = parse_markdown_table_at(lines, cursor) {
            if let (Some(field_idx), Some(value_idx)) = (
                find_table_column(&headers, &["field", "property", "key"]),
                find_table_column(&headers, &["value"]),
            ) {
                for row in rows {
                    let field = normalize_key(table_cell(&row, field_idx).as_str());
                    let value = table_cell(&row, value_idx);
                    match field.as_str() {
                        "id" | "capabilityid" => id = Some(value),
                        "type" | "capabilitytype" => {
                            capability_type = CapabilityType::from_cli_str(&value).ok();
                        }
                        _ => {}
                    }
                }
            }
            cursor = next_cursor;
            continue;
        }
        cursor += 1;
    }
    Some((id?, capability_type?))
}

fn split_markdown_field(line: &str) -> Option<(&str, &str)> {
    let line = line.strip_prefix("- ").unwrap_or(line).trim();
    line.split_once(':')
}

fn is_markdown_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    (1..=6).contains(&level) && trimmed.chars().nth(level).is_some_and(|ch| ch == ' ')
}

fn next_markdown_heading(lines: &[&str], start: usize) -> Option<usize> {
    (start..lines.len()).find(|idx| is_markdown_heading(lines[*idx]))
}

fn parse_markdown_table_at(
    lines: &[&str],
    start: usize,
) -> Option<(Vec<String>, Vec<Vec<String>>, usize)> {
    let headers = parse_markdown_table_row(lines.get(start)?)?;
    let separator = parse_markdown_table_row(lines.get(start + 1)?)?;
    if !is_markdown_separator_row(&separator) {
        return None;
    }
    let mut rows = Vec::new();
    let mut cursor = start + 2;
    while cursor < lines.len() {
        let Some(cells) = parse_markdown_table_row(lines[cursor]) else {
            break;
        };
        if is_markdown_separator_row(&cells) {
            cursor += 1;
            continue;
        }
        rows.push(cells);
        cursor += 1;
    }
    Some((headers, rows, cursor))
}

fn parse_markdown_table_row(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed[1..].contains('|') {
        return None;
    }
    Some(
        trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().replace("\\|", "|"))
            .collect(),
    )
}

fn is_markdown_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            !trimmed.is_empty()
                && trimmed.chars().all(|c| matches!(c, '-' | ':' | ' '))
                && trimmed.chars().any(|c| c == '-')
        })
}

fn table_cell(cells: &[String], idx: usize) -> String {
    cells
        .get(idx)
        .map(|cell| cell.trim().to_string())
        .filter(|cell| !cell.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

fn find_table_column(cells: &[String], aliases: &[&str]) -> Option<usize> {
    cells.iter().position(|cell| {
        let normalized = normalize_key(cell);
        aliases.iter().any(|alias| normalized == *alias)
    })
}

fn normalize_key(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>()
}

/// Load the optional `.aw/capability-types.toml` `[capability_types]` fallback
/// table. Callers merge this after README-derived types without overriding
/// README entries. A missing file/table yields an empty map; a malformed file
/// errors with the path.
pub fn load_capability_types(project_root: &Path) -> Result<BTreeMap<String, CapabilityType>> {
    let path = capability_types_path(project_root);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let parsed: CapabilityTypesFile =
        toml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    Ok(parsed.capability_types)
}

/// Insert/overwrite one `<capability_id> = "<Type>"` entry in
/// `.aw/capability-types.toml`, preserving all existing entries (and the file's
/// existing formatting/comments). Creates the file and the `[capability_types]`
/// table if absent.
pub fn upsert_capability_type(
    project_root: &Path,
    capability_id: &str,
    capability_type: CapabilityType,
) -> Result<PathBuf> {
    let path = capability_types_path(project_root);
    let existing = if path.exists() {
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?
    } else {
        String::new()
    };
    let mut doc = existing
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("parse {}", path.display()))?;
    let table = doc
        .entry("capability_types")
        .or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_mut()
        .with_context(|| format!("`capability_types` in {} is not a table", path.display()))?;
    table.insert(
        capability_id,
        toml_edit::value(capability_type.as_str().to_string()),
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(&path, doc.to_string()).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn dims(capability_type: CapabilityType) -> BTreeSet<&'static str> {
        required_ec_dimensions(&capability_type)
            .iter()
            .copied()
            .collect()
    }

    #[test]
    fn required_ec_dimensions_service_is_all_four() {
        let service = dims(CapabilityType::Service);
        assert_eq!(
            service,
            BTreeSet::from(["behavior", "efficiency", "security", "stability"])
        );
        assert_eq!(service.len(), 4);
        // Every required dimension must be a real EC category.
        for dim in &service {
            assert!(
                crate::cli::ec::ec_categories().contains(dim),
                "{dim} is not an EC category"
            );
        }
    }

    #[test]
    fn required_ec_dimensions_agent_first_is_behavior_only() {
        assert_eq!(
            dims(CapabilityType::AgentFirst),
            BTreeSet::from(["behavior"])
        );
    }

    #[test]
    fn required_ec_dimensions_devops_is_behavior_and_stability() {
        assert_eq!(
            dims(CapabilityType::Devops),
            BTreeSet::from(["behavior", "stability"])
        );
    }

    #[test]
    fn required_ec_dimensions_developer_and_runtime_tools_include_efficiency() {
        assert_eq!(
            dims(CapabilityType::DeveloperTool),
            BTreeSet::from(["behavior", "efficiency", "stability"])
        );
        assert_eq!(
            dims(CapabilityType::RuntimeTool),
            BTreeSet::from(["behavior", "efficiency", "stability"])
        );
    }

    #[test]
    fn required_ec_dimensions_security_tool_includes_security() {
        assert_eq!(
            dims(CapabilityType::SecurityTool),
            BTreeSet::from(["behavior", "security", "stability"])
        );
    }

    #[test]
    fn category_is_required_for_type_matrix() {
        assert!(category_is_required_for_type(
            &CapabilityType::Service,
            "security"
        ));
        assert!(category_is_required_for_type(
            &CapabilityType::Service,
            "stability"
        ));
        assert!(!category_is_required_for_type(
            &CapabilityType::AgentFirst,
            "efficiency"
        ));
        assert!(category_is_required_for_type(
            &CapabilityType::AgentFirst,
            "behavior"
        ));
        assert!(!category_is_required_for_type(
            &CapabilityType::Devops,
            "security"
        ));
        assert!(category_is_required_for_type(
            &CapabilityType::Devops,
            "stability"
        ));
        assert!(category_is_required_for_type(
            &CapabilityType::DeveloperTool,
            "efficiency"
        ));
        assert!(!category_is_required_for_type(
            &CapabilityType::DeveloperTool,
            "security"
        ));
        assert!(category_is_required_for_type(
            &CapabilityType::SecurityTool,
            "security"
        ));
        assert!(!category_is_required_for_type(
            &CapabilityType::SecurityTool,
            "efficiency"
        ));
    }

    #[test]
    fn load_missing_file_is_empty_map() {
        let dir = tempfile::tempdir().unwrap();
        let map = load_capability_types(dir.path()).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn load_from_readme_pillar_index() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("README.md"),
            "## Capability Index\n\n\
             **Pillar — agent-first** (EC: behavior)\n\n\
             | Capability | Root WI |\n|---|---:|\n| agentic-integration | a |\n\n\
             **Pillar — serve / search** (EC: four)\n\n\
             | Capability | Root WI |\n|---|---:|\n| search | s |\n| security-auth | s |\n\n\
             **Pillar — devops-operation** (EC: render)\n\n\
             | Capability | Root WI |\n|---|---:|\n| k8s-deployment | k |\n| ops-operability | o |\n\n\
             **Pillar — security** (EC: security evidence)\n\n\
             | Capability | Root WI |\n|---|---:|\n| static-security-scan | g |\n\n\
             **Honest scope:**\n\n\
             | not-a-capability | x |\n",
        )
        .unwrap();
        let map = load_capability_types_from_readme(&dir.path().join("README.md")).unwrap();
        assert_eq!(
            map.get("agentic-integration"),
            Some(&CapabilityType::AgentFirst)
        );
        assert_eq!(map.get("search"), Some(&CapabilityType::Service));
        assert_eq!(map.get("security-auth"), Some(&CapabilityType::Service));
        assert_eq!(map.get("k8s-deployment"), Some(&CapabilityType::Devops));
        assert_eq!(map.get("ops-operability"), Some(&CapabilityType::Devops));
        assert_eq!(
            map.get("static-security-scan"),
            Some(&CapabilityType::SecurityTool)
        );
        // A non-pillar bold header ("**Honest scope**") ends pillar scope.
        assert_eq!(map.get("not-a-capability"), None);
        // Header/separator rows are not captured as capabilities.
        assert_eq!(map.get("Capability"), None);
    }

    #[test]
    fn load_from_readme_explicit_capability_type_field() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("README.md"),
            r#"# jet

## Package Manager

ID: package-manager
Type: DeveloperTool
Status: auditing

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| readiness | epic | #1 | partial | planned | smoke | fixture |

## Search

| Field | Value |
|---|---|
| ID | search |
| Type | Service |
| Status | auditing |
"#,
        )
        .unwrap();
        let map = load_capability_types_from_readme(&dir.path().join("README.md")).unwrap();

        assert_eq!(
            map.get("package-manager"),
            Some(&CapabilityType::DeveloperTool)
        );
        assert_eq!(map.get("search"), Some(&CapabilityType::Service));
    }

    #[test]
    fn load_from_canonical_field_style_contract_overrides_legacy_pillar() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("README.md"),
            r#"# Jet

## Brief

Frontend toolchain.

## Capabilities

### Capability Index

**Pillar — serve / legacy fallback** (EC: four)

| Capability | Root WI |
|---|---:|
| package-manager | #1 |

### Package Manager

ID: package-manager
Type: DeveloperTool
Root WI: #1
Status: auditing
Surfaces: CLI: `jet install` - package-management command surface
EC Dimensions: behavior: `cargo test -p jet --lib pkg_manager` - package lifecycle conformance
Required Verification: smoke
Promise:
Jet can replace package manager flows.
Gate Inventory:
- cargo test -p jet --lib pkg_manager
"#,
        )
        .unwrap();

        let map = load_capability_types_from_readme(&dir.path().join("README.md")).unwrap();

        assert_eq!(
            map.get("package-manager"),
            Some(&CapabilityType::DeveloperTool)
        );
    }

    #[test]
    fn load_capability_types_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".aw")).unwrap();
        std::fs::write(
            capability_types_path(dir.path()),
            "[capability_types]\nserve-thing = \"Service\"\nagent-thing = \"AgentFirst\"\nops-thing = \"Devops\"\ndev-tool = \"DeveloperTool\"\nruntime-tool = \"RuntimeTool\"\nsecurity-tool = \"SecurityTool\"\n",
        )
        .unwrap();
        let map = load_capability_types(dir.path()).unwrap();
        assert_eq!(map.len(), 6);
        assert_eq!(map.get("serve-thing"), Some(&CapabilityType::Service));
        assert_eq!(map.get("agent-thing"), Some(&CapabilityType::AgentFirst));
        assert_eq!(map.get("ops-thing"), Some(&CapabilityType::Devops));
        assert_eq!(map.get("dev-tool"), Some(&CapabilityType::DeveloperTool));
        assert_eq!(map.get("runtime-tool"), Some(&CapabilityType::RuntimeTool));
        assert_eq!(
            map.get("security-tool"),
            Some(&CapabilityType::SecurityTool)
        );
    }

    #[test]
    fn upsert_creates_then_preserves_existing_entries() {
        let dir = tempfile::tempdir().unwrap();
        // First upsert creates the file + table.
        upsert_capability_type(dir.path(), "first", CapabilityType::Service).unwrap();
        // Second upsert must preserve `first` and add `second`.
        upsert_capability_type(dir.path(), "second", CapabilityType::Devops).unwrap();
        // Third upsert overwrites `first`'s value.
        upsert_capability_type(dir.path(), "first", CapabilityType::AgentFirst).unwrap();

        let map = load_capability_types(dir.path()).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("first"), Some(&CapabilityType::AgentFirst));
        assert_eq!(map.get("second"), Some(&CapabilityType::Devops));
    }
}
