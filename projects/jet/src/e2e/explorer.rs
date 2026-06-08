// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Open-mode case explorer filtering (#2727).
//!
//! Open mode renders the discovered E2E case list (from #2722's
//! [`DiscoveryManifest`]) in a left-side explorer panel so reviewers
//! can select a single case to replay. This module owns the
//! filtering layer that sits between the manifest and the UI:
//!
//! - text filter — case-insensitive substring on case title and file
//!   path, so reviewers can type a phrase and narrow without learning
//!   the tag vocabulary.
//! - tag filter — AND-include / NONE-exclude semantics shared with
//!   `jet e2e run --tag` so run and open render identical lists.
//!
//! Filtering is pure: the underlying [`DiscoveryManifest`] is never
//! mutated. Empty / no-match states are exposed via
//! [`ExplorerView::is_empty`] so the UI can render an explicit
//! "no cases match" panel.

use crate::e2e_discovery::{DiscoveryManifest, E2eCase, TagFilter};
use serde::{Deserialize, Serialize};

/// Filter applied on top of the discovery manifest. All fields are
/// optional; the empty filter returns every case in the manifest.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplorerFilter {
    /// Case-insensitive substring matched against title + file path.
    /// `None` means "no text filter".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Tag include/exclude filter. See [`TagFilter`] for semantics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_tags: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ExplorerFilter {
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn include_tag(mut self, tag: impl Into<String>) -> Self {
        self.include_tags.push(tag.into());
        self
    }

    pub fn exclude_tag(mut self, tag: impl Into<String>) -> Self {
        self.exclude_tags.push(tag.into());
        self
    }

    fn tag_filter(&self) -> TagFilter {
        TagFilter {
            include: self.include_tags.clone(),
            exclude: self.exclude_tags.clone(),
        }
    }

    fn matches(&self, case: &E2eCase) -> bool {
        if !self.tag_filter().matches(case) {
            return false;
        }
        if let Some(needle) = &self.text {
            let needle = needle.to_lowercase();
            let hay_title = case.title.to_lowercase();
            let hay_file = case.file.display().to_string().to_lowercase();
            if !hay_title.contains(&needle) && !hay_file.contains(&needle) {
                return false;
            }
        }
        true
    }
}

/// Rendered view: the cases that pass the filter, in manifest order.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerView {
    pub cases: Vec<E2eCase>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ExplorerView {
    /// Apply a filter to a manifest. Underlying manifest is left
    /// untouched.
    pub fn from_manifest(manifest: &DiscoveryManifest, filter: &ExplorerFilter) -> Self {
        Self {
            cases: manifest
                .cases
                .iter()
                .filter(|c| filter.matches(c))
                .cloned()
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cases.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e_discovery::E2E_DISCOVERY_SCHEMA_VERSION;
    use std::path::PathBuf;

    fn case(file: &str, title: &str, tags: &[&str]) -> E2eCase {
        E2eCase {
            id: format!("{file}::{title}"),
            title: title.into(),
            file: PathBuf::from(file),
            line: 1,
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn manifest(cases: Vec<E2eCase>) -> DiscoveryManifest {
        DiscoveryManifest {
            schema_version: E2E_DISCOVERY_SCHEMA_VERSION.to_string(),
            root: PathBuf::from("/tmp/proj"),
            cases,
        }
    }

    #[test]
    fn empty_filter_returns_all_cases() {
        let m = manifest(vec![
            case("flows/buy.case.ts", "buy", &[]),
            case("flows/sell.case.ts", "sell", &[]),
        ]);
        let view = ExplorerView::from_manifest(&m, &ExplorerFilter::default());
        assert_eq!(view.len(), 2);
        assert!(!view.is_empty());
    }

    #[test]
    fn text_filter_is_case_insensitive_on_title() {
        // Stop condition (#2727): fixture case list filters by text.
        let m = manifest(vec![
            case("flows/buy.case.ts", "buyer completes checkout", &[]),
            case("flows/sell.case.ts", "seller cancels order", &[]),
            case("flows/admin.case.ts", "admin reviews returns", &[]),
        ]);
        let view = ExplorerView::from_manifest(&m, &ExplorerFilter::default().with_text("BUYER"));
        assert_eq!(view.len(), 1);
        assert_eq!(view.cases[0].title, "buyer completes checkout");
    }

    #[test]
    fn text_filter_also_matches_file_path() {
        let m = manifest(vec![
            case("flows/checkout/order.case.ts", "first", &[]),
            case("flows/admin/order.case.ts", "second", &[]),
        ]);
        let view = ExplorerView::from_manifest(&m, &ExplorerFilter::default().with_text("admin"));
        assert_eq!(view.len(), 1);
        assert_eq!(view.cases[0].title, "second");
    }

    #[test]
    fn tag_include_filter_keeps_only_matching_cases() {
        // Stop condition (#2727): fixture case list filters by tag.
        let m = manifest(vec![
            case("flows/buy.case.ts", "buy", &["smoke"]),
            case("flows/sell.case.ts", "sell", &["smoke", "checkout"]),
            case("flows/admin.case.ts", "admin", &["admin"]),
        ]);
        let view =
            ExplorerView::from_manifest(&m, &ExplorerFilter::default().include_tag("checkout"));
        assert_eq!(view.len(), 1);
        assert_eq!(view.cases[0].title, "sell");
    }

    #[test]
    fn tag_exclude_filter_drops_matching_cases() {
        let m = manifest(vec![
            case("flows/buy.case.ts", "buy", &["smoke"]),
            case("flows/wip.case.ts", "wip", &["wip"]),
        ]);
        let view = ExplorerView::from_manifest(&m, &ExplorerFilter::default().exclude_tag("wip"));
        assert_eq!(view.len(), 1);
        assert_eq!(view.cases[0].title, "buy");
    }

    #[test]
    fn combined_text_and_tag_filter_apply_with_and_semantics() {
        let m = manifest(vec![
            case("flows/buy.case.ts", "buyer cancel", &["smoke"]),
            case("flows/buy2.case.ts", "buyer purchase", &["wip"]),
        ]);
        let filter = ExplorerFilter::default()
            .with_text("buyer")
            .exclude_tag("wip");
        let view = ExplorerView::from_manifest(&m, &filter);
        assert_eq!(view.len(), 1);
        assert_eq!(view.cases[0].title, "buyer cancel");
    }

    #[test]
    fn no_match_produces_empty_view_without_mutating_manifest() {
        let m = manifest(vec![case("flows/buy.case.ts", "buy", &[])]);
        let view = ExplorerView::from_manifest(&m, &ExplorerFilter::default().with_text("zzz"));
        assert!(view.is_empty());
        // Underlying manifest is untouched.
        assert_eq!(m.cases.len(), 1);
    }

    #[test]
    fn view_preserves_manifest_order() {
        let m = manifest(vec![
            case("a/one.case.ts", "alpha", &["t"]),
            case("a/two.case.ts", "beta", &["t"]),
            case("b/three.case.ts", "gamma", &["t"]),
        ]);
        let view = ExplorerView::from_manifest(&m, &ExplorerFilter::default().include_tag("t"));
        let titles: Vec<&str> = view.cases.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, vec!["alpha", "beta", "gamma"]);
    }
}
// CODEGEN-END
