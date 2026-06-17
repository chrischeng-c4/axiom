//! Capability TYPE -> production-required EC dimensions.
//!
//! A capability's *type* is a structural classification (AgentFirst / Service /
//! Devops) that determines which external-contract (EC) dimensions are
//! production-required. This is deliberately decoupled from maturity/env (vat):
//! the type decides *which* EC dimensions are required for production; maturity
//! decides only *whether* a given gate is verified/runnable, and must never
//! flip `required_for_production`.
//!
//! The PRIMARY source of a capability's type is the project README's Capability
//! Index: capabilities grouped under `**Pillar — agent-first**` are AgentFirst,
//! under `**Pillar — serve…**` are Service, and under `**Pillar — devops…**` are
//! Devops. Readers (`aw ec`, deriving `required_for_production`) resolve it via
//! [`load_capability_types`] which parses that pillar grouping
//! ([`load_capability_types_from_readme`]). An optional
//! `.aw/capability-types.toml` `[capability_types]` table overrides per
//! capability (the README stays the single source of truth):
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
/// the exact strings `"AgentFirst"`, `"Service"`, and `"Devops"` so the on-disk
/// `.aw/capability-types.toml` is human-authorable. The enum is extensible: add
/// a variant plus its required-dimension set and the serde rename.
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
}

impl CapabilityType {
    /// The exact string used on disk and in HITL choices.
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityType::AgentFirst => "AgentFirst",
            CapabilityType::Service => "Service",
            CapabilityType::Devops => "Devops",
        }
    }

    /// Parse a CLI / HITL-answer string into a [`CapabilityType`].
    ///
    /// Accepts the canonical names (`AgentFirst` / `Service` / `Devops`,
    /// case-insensitively) as well as the HITL choice ids (`agent_first` /
    /// `service` / `devops`).
    pub fn from_cli_str(value: &str) -> Result<CapabilityType> {
        match value.trim().to_ascii_lowercase().as_str() {
            "agentfirst" | "agent_first" | "agent-first" => Ok(CapabilityType::AgentFirst),
            "service" => Ok(CapabilityType::Service),
            "devops" => Ok(CapabilityType::Devops),
            other => anyhow::bail!(
                "unknown capability type `{other}`; expected AgentFirst, Service, or Devops"
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
/// - `AgentFirst` -> `{behavior}`
/// - `Service`    -> `{behavior, efficiency, security, stability}`
/// - `Devops`     -> `{behavior, stability}`
pub fn required_ec_dimensions(capability_type: &CapabilityType) -> &'static [&'static str] {
    match capability_type {
        CapabilityType::AgentFirst => &["behavior"],
        CapabilityType::Service => &["behavior", "efficiency", "security", "stability"],
        CapabilityType::Devops => &["behavior", "stability"],
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

/// Map of `capability_id -> CapabilityType` derived from the README Capability
/// Index. Capabilities are grouped under `**Pillar — <name>**` headers; each
/// capability row's first table cell is its `capability_id`, and the pillar name
/// maps to a type: `agent-first -> AgentFirst`, `serve… -> Service`,
/// `devops… -> Devops`. A missing README (or no pillars) yields an empty map.
///
/// This is the PRIMARY source of capability types — the README is the single
/// source of truth. A bold header that is not a pillar (e.g. `**Honest scope**`)
/// or a markdown heading ends the current pillar's table scope.
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
    Ok(map)
}

/// Load the optional `.aw/capability-types.toml` `[capability_types]` override
/// table. This is layered ON TOP of the README-derived types
/// ([`load_capability_types_from_readme`]) by callers (README is the primary
/// source). A missing file/table yields an empty map; a malformed file errors
/// with the path.
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
        // A non-pillar bold header ("**Honest scope**") ends pillar scope.
        assert_eq!(map.get("not-a-capability"), None);
        // Header/separator rows are not captured as capabilities.
        assert_eq!(map.get("Capability"), None);
    }

    #[test]
    fn load_capability_types_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".aw")).unwrap();
        std::fs::write(
            capability_types_path(dir.path()),
            "[capability_types]\nserve-thing = \"Service\"\nagent-thing = \"AgentFirst\"\nops-thing = \"Devops\"\n",
        )
        .unwrap();
        let map = load_capability_types(dir.path()).unwrap();
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("serve-thing"), Some(&CapabilityType::Service));
        assert_eq!(map.get("agent-thing"), Some(&CapabilityType::AgentFirst));
        assert_eq!(map.get("ops-thing"), Some(&CapabilityType::Devops));
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
