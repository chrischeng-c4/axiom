// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/slug_preamble_source.md#source
// CODEGEN-BEGIN
//! Phase B: id-as-slug — canonical slug parsing, building, and resolution.
//!
//! @spec projects/agentic-workflow/tech-design/core/logic/issues/slug-and-id.md#schema
//!
//! Canonical slug form: `<id>` (e.g. `1234`). The id is the primary key
//! (GitHub number, GitLab iid, or a locally-allocated u64). Legacy
//! `<id>-<title-kebab>` inputs are accepted for compatibility; the tail is
//! ignored for matching and is no longer emitted for new artifacts.
//!
//! Three accepted input forms (any aw binary verb taking a slug positional):
//! - bare numeric:   `1234`               — direct id; bypasses alias table
//! - legacy prefix:  `1234-fix-auth-flow` — id from prefix, kebab ignored
//! - legacy:         `fix-auth-flow`      — looked up in the temp alias table

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
// CODEGEN-BEGIN
/// Issue/branch kind discriminator used by `parse_branch_name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
pub enum BranchKind {
    Issue,
    Td,
    Cb,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
impl BranchKind {
    pub fn as_prefix(&self) -> &'static str {
        match self {
            BranchKind::Issue => "issue",
            BranchKind::Td => "td",
            BranchKind::Cb => "cb",
        }
    }
}

/// Outcome of resolving a slug-or-id input string.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
pub enum ResolvedId {
    /// Resolved directly from `^\d+$` input (bypassed alias table).
    Numeric(u64),
    /// Resolved from `^(\d+)-.+` prefix; kebab tail kept for display.
    PrefixId { id: u64, kebab_tail: String },
    /// Resolved by alias-table lookup of legacy free-form slug.
    AliasHit { id: u64, legacy_slug: String },
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
impl ResolvedId {
    pub fn id(&self) -> u64 {
        match self {
            ResolvedId::Numeric(n) => *n,
            ResolvedId::PrefixId { id, .. } => *id,
            ResolvedId::AliasHit { id, .. } => *id,
        }
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, ResolvedId::AliasHit { .. })
    }
}

/// Slug alias table in the temp issue store (`{ "<old-slug>": <id>, ... }`).
/// Honoured for one release post-migration; operator deletes the file thereafter.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
pub struct SlugAliases {
    pub map: HashMap<String, u64>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
impl SlugAliases {
    /// Load the alias table from the temp issue store.
    /// Returns an empty table when the file is missing (post-migration steady state).
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = crate::shared::workspace::issues_path(project_root).join(".slug-aliases.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let body = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let map: HashMap<String, u64> = serde_json::from_str(&body)
            .with_context(|| format!("parsing {} as JSON", path.display()))?;
        Ok(Self { map })
    }

    /// Persist the alias table.
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = crate::shared::workspace::issues_path(project_root).join(".slug-aliases.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(&self.map)?;
        std::fs::write(&path, body)?;
        Ok(())
    }

    pub fn lookup(&self, legacy_slug: &str) -> Option<u64> {
        self.map.get(legacy_slug).copied()
    }

    pub fn insert(&mut self, legacy_slug: String, id: u64) {
        self.map.insert(legacy_slug, id);
    }
}

/// Build the canonical slug from an id. The title is accepted only for
/// source compatibility with older callers.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
pub fn build_canonical_slug(id: u64, _title: &str) -> String {
    id.to_string()
}

/// Resolve any of the three accepted slug-input forms to a [`ResolvedId`].
///
/// Resolution order matches the flowchart in `slug-and-id.md#logic`:
/// 1. bare-numeric short-circuit (no alias-table consult)
/// 2. canonical prefix-id parse
/// 3. alias-table lookup
/// 4. error: unresolvable
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
pub fn parse_slug_input(input: &str, aliases: &SlugAliases) -> Result<ResolvedId> {
    // 1. bare-numeric
    if !input.is_empty() && input.chars().all(|c| c.is_ascii_digit()) {
        let id: u64 = input
            .parse()
            .with_context(|| format!("bare-numeric slug input '{input}' overflows u64"))?;
        return Ok(ResolvedId::Numeric(id));
    }

    // 2. prefix-id (digits-then-hyphen)
    if let Some((prefix, tail)) = split_prefix_id(input) {
        return Ok(ResolvedId::PrefixId {
            id: prefix,
            kebab_tail: tail.to_string(),
        });
    }

    // 3. alias-table
    if let Some(id) = aliases.lookup(input) {
        return Ok(ResolvedId::AliasHit {
            id,
            legacy_slug: input.to_string(),
        });
    }

    // 4. error
    anyhow::bail!(
        "cannot resolve slug '{input}' — no numeric id, legacy <id>- prefix, or alias entry. \
         Provide bare-numeric (1234), legacy prefix (1234-title), or run \
         `aw wi migrate-slugs --apply` to register the legacy slug."
    )
}

fn split_prefix_id(input: &str) -> Option<(u64, &str)> {
    let dash = input.find('-')?;
    let head = &input[..dash];
    let tail = &input[dash + 1..];
    if head.is_empty() || !head.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let id: u64 = head.parse().ok()?;
    Some((id, tail))
}

/// Parse a branch name in the form `<kind>-<id>` or legacy
/// `<kind>-<id>-<kebab>` and return `(kind, id)`. Returns `None` for any
/// input that does not match.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
pub fn parse_branch_name(branch: &str) -> Option<(BranchKind, u64)> {
    for (kind, prefix) in [
        (BranchKind::Issue, "issue-"),
        (BranchKind::Td, "td-"),
        (BranchKind::Cb, "cb-"),
    ] {
        if let Some(rest) = branch.strip_prefix(prefix) {
            if let Some((id, _)) = split_prefix_id(rest) {
                return Some((kind, id));
            }
            // Allow bare `<kind>-<id>` (no kebab tail).
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                return rest.parse().ok().map(|id| (kind, id));
            }
        }
    }
    None
}

/// Build a branch name from kind + id. The title is ignored.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md#source
pub fn build_branch_name(kind: BranchKind, id: u64, title: &str) -> String {
    let canonical = build_canonical_slug(id, title);
    format!("{}-{}", kind.as_prefix(), canonical)
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/slug_tests_source.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_canonical_basic() {
        assert_eq!(build_canonical_slug(1234, "Fix auth flow"), "1234");
    }

    #[test]
    fn build_canonical_ignores_title() {
        assert_eq!(build_canonical_slug(1234, "Fix auth, restore SSO"), "1234");
    }

    #[test]
    fn build_canonical_has_no_title_tail_to_truncate() {
        let title = "this is a very long title that exceeds the sixty character limit by a lot";
        let slug = build_canonical_slug(99, title);
        assert_eq!(slug, "99");
    }

    #[test]
    fn parse_bare_numeric() {
        let aliases = SlugAliases::default();
        let r = parse_slug_input("1234", &aliases).unwrap();
        assert_eq!(r, ResolvedId::Numeric(1234));
        assert!(!r.is_legacy());
    }

    #[test]
    fn parse_canonical_strips_kebab() {
        let aliases = SlugAliases::default();
        let r = parse_slug_input("1234-fix-auth", &aliases).unwrap();
        assert_eq!(r.id(), 1234);
        match r {
            ResolvedId::PrefixId { id, kebab_tail } => {
                assert_eq!(id, 1234);
                assert_eq!(kebab_tail, "fix-auth");
            }
            other => panic!("expected PrefixId, got {other:?}"),
        }
    }

    #[test]
    fn parse_legacy_via_alias() {
        let mut aliases = SlugAliases::default();
        aliases.insert("legacy-old-slug".to_string(), 42);
        let r = parse_slug_input("legacy-old-slug", &aliases).unwrap();
        assert_eq!(r.id(), 42);
        assert!(r.is_legacy());
    }

    #[test]
    fn parse_unresolvable_errors() {
        let aliases = SlugAliases::default();
        let err = parse_slug_input("totally-unknown-slug", &aliases).unwrap_err();
        assert!(err.to_string().contains("cannot resolve slug"), "{err}");
    }

    #[test]
    fn parse_bare_numeric_bypasses_alias_table() {
        // Even if "1234" were in the alias table, the bare-numeric branch wins first.
        let mut aliases = SlugAliases::default();
        aliases.insert("1234".to_string(), 999);
        let r = parse_slug_input("1234", &aliases).unwrap();
        assert_eq!(r, ResolvedId::Numeric(1234));
    }

    #[test]
    fn parse_branch_name_td() {
        assert_eq!(
            parse_branch_name("td-1234-fix-auth"),
            Some((BranchKind::Td, 1234))
        );
    }

    #[test]
    fn parse_branch_name_issue() {
        assert_eq!(
            parse_branch_name("issue-9999-do-thing"),
            Some((BranchKind::Issue, 9999))
        );
    }

    #[test]
    fn parse_branch_name_cb_no_tail() {
        assert_eq!(parse_branch_name("cb-7"), Some((BranchKind::Cb, 7)));
    }

    #[test]
    fn parse_branch_name_rejects_legacy_form() {
        // Legacy branches like `td-fix-auth-flow` (no id prefix) return None.
        assert_eq!(parse_branch_name("td-fix-auth-flow"), None);
    }

    #[test]
    fn build_branch_name_round_trip() {
        let branch = build_branch_name(BranchKind::Td, 42, "Fix Auth Flow");
        assert_eq!(branch, "td-42");
        assert_eq!(parse_branch_name(&branch), Some((BranchKind::Td, 42)));
    }

    #[test]
    fn slug_aliases_save_load_round_trip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut aliases = SlugAliases::default();
        aliases.insert("legacy-foo".to_string(), 1);
        aliases.insert("legacy-bar".to_string(), 2);
        aliases.save(tmp.path()).unwrap();

        let reloaded = SlugAliases::load(tmp.path()).unwrap();
        assert_eq!(reloaded.lookup("legacy-foo"), Some(1));
        assert_eq!(reloaded.lookup("legacy-bar"), Some(2));
        assert_eq!(reloaded.lookup("missing"), None);
    }

    #[test]
    fn slug_aliases_load_missing_file_returns_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let aliases = SlugAliases::load(tmp.path()).unwrap();
        assert!(aliases.map.is_empty());
    }
}
// CODEGEN-END
