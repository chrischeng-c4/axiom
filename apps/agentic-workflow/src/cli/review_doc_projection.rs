// HANDWRITE-BEGIN gap="missing-generator:logic:563ad218" tracker="pending-tracker" reason="New module (whole-file hand-written, matching the `review_rules.rs`/`review_obs_rules.rs` precedent -- no generator primitive yet renders a live-registry-driven Markdown table into a marker-delimited section of a repo-root doc and drift-tests it): the CONTRIBUTING.md profile/rule-registry doc-projection producer plus its drift test. - `pub(crate) const REVIEW_RULE_TABLE_START: &str = '<!-- aw:review-rule-table:start -- >'` and `pub(crate) const REVIEW_RULE_TABLE_END: &str = '<!-- aw:review-rule-table:end -- >'` -- the marker pair spliced into CONTRIBUTING.md, the same opt-in-per-document marker-splice shape `doc_mirror::TRAIT_TABLE_START`/`TRAIT_TABLE_END` and `meta_docs::META_DOC_MATRIX_START`/`_END` already use (reused pattern, this module intentionally does not register itself as a new `meta::MetaDocProducer`/`ProducerKind` variant -- `meta.rs`/`doc_mirror.rs` are `SPEC-MANAGED`/`CODEGEN` files outside this hand-written module's scope, and `aw meta sync` stays untouched by this change; this projection's own drift test is the enforcement mechanism instead). - `pub(crate) fn render_review_rule_table() -> String` -- builds a `| Rule ID | Family | Fires when |` Markdown table with one row per `review_rules::known_rule_docs()` entry followed by one row per `review_obs_rules::known_rule_docs()` entry (fixed insertion order: shared-kit, then negative-assertion, then obs, then raft -- deterministic, matches source declaration order in each registry), each row rendering `id` in backticks, `family` in backticks, and `description` as plain prose. - `#[cfg(test)] mod tests` with: - `contributing_review_rule_table_matches_live_registry` -- reads the repo-root `CONTRIBUTING.md` (via `PathBuf::from(env!('CARGO_MANIFEST_DIR')).parent().and_then(Path::parent)`, the exact repository-root resolution `meta_docs::tests::meta_doc_ownership_contributing_projection_matches_matrix` already uses), slices the block between `REVIEW_RULE_TABLE_START`/`REVIEW_RULE_TABLE_END`, and asserts it equals `render_review_rule_table()` (both `.trim()`-ed) -- the drift test: fails the moment a `RULE_ID_*` constant or `KIT_RULES`/`known_rule_docs()` entry is added, renamed, or removed in `review_rules.rs`/`review_obs_rules.rs` without CONTRIBUTING.md being re-spliced to match. - `render_review_rule_table_lists_every_known_rule_id` -- asserts the rendered table string contains every id from `review_rules::known_rule_docs()` and `review_obs_rules::known_rule_docs()` at least once, each wrapped in backticks. gap: review-rule-doc-projection-and-drift-test tracker: '#2169'"
//! CONTRIBUTING.md profile/rule-registry doc-projection: renders a Markdown
//! table from the live `review_rules`/`review_obs_rules` rule registries and
//! drift-tests it against the marker-delimited block in CONTRIBUTING.md.
//! Read-only, additive: never mutates a `Finding`, `KitRule`, or `RuleDoc`,
//! only reads the already-named `RULE_ID_*`/`KitRule.id` constants those
//! modules expose via `known_rule_docs()`.
//!
//! @spec apps/agentic-workflow/tech-design/validate/review-skill-doc-trait-projection-with-drift-tests.md#logic

use std::path::{Path, PathBuf};

use crate::cli::{review_obs_rules, review_rules};

/// Start marker for the generated rule table spliced into CONTRIBUTING.md,
/// following the same opt-in-per-document marker-splice shape
/// `doc_mirror::TRAIT_TABLE_START`/`TRAIT_TABLE_END` and
/// `meta_docs::META_DOC_MATRIX_START`/`_END` already use.
pub(crate) const REVIEW_RULE_TABLE_START: &str = "<!-- aw:review-rule-table:start -->";
/// End marker for the generated rule table spliced into CONTRIBUTING.md.
pub(crate) const REVIEW_RULE_TABLE_END: &str = "<!-- aw:review-rule-table:end -->";

/// Render the `| Rule ID | Family | Fires when |` Markdown table from the
/// live rule registries: `review_rules::known_rule_docs()` (shared-kit, then
/// negative-assertion) followed by `review_obs_rules::known_rule_docs()`
/// (obs, then raft). Deterministic insertion order matches each registry's
/// source declaration order. Pure function -- no I/O.
pub(crate) fn render_review_rule_table() -> String {
    let mut rows: Vec<review_rules::RuleDoc> = review_rules::known_rule_docs();
    rows.extend(review_obs_rules::known_rule_docs());

    let mut out = String::from("| Rule ID | Family | Fires when |\n|---|---|---|\n");
    for row in &rows {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            row.id, row.family, row.description
        ));
    }
    out.trim_end().to_string()
}

/// Locate the repository root from this crate's `CARGO_MANIFEST_DIR`
/// (`apps/agentic-workflow`), matching the exact resolution
/// `meta_docs::tests::meta_doc_ownership_contributing_projection_matches_matrix`
/// already uses.
#[cfg(test)]
fn repository_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("apps/agentic-workflow/Cargo.toml has two ancestors up to the repo root")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The drift test (R7): the Markdown table spliced into CONTRIBUTING.md
    /// between the `aw:review-rule-table` markers must stay byte-identical
    /// (trimmed) to `render_review_rule_table()`'s live output. Because
    /// every rule id used inside a `finding()` call site is a named
    /// `RULE_ID_*`/`KitRule.id` constant (never an inline string literal),
    /// this fails the moment a rule id is added, renamed, or removed in
    /// `review_rules.rs`/`review_obs_rules.rs` without CONTRIBUTING.md being
    /// re-spliced to match.
    #[test]
    fn contributing_review_rule_table_matches_live_registry() {
        let contributing_path = repository_root().join("CONTRIBUTING.md");
        let contributing = std::fs::read_to_string(&contributing_path).unwrap_or_else(|e| {
            panic!(
                "failed to read {}: {e}",
                contributing_path.display()
            )
        });

        let start = contributing.find(REVIEW_RULE_TABLE_START).unwrap_or_else(|| {
            panic!(
                "CONTRIBUTING.md is missing the {} marker",
                REVIEW_RULE_TABLE_START
            )
        }) + REVIEW_RULE_TABLE_START.len();
        let end = contributing.find(REVIEW_RULE_TABLE_END).unwrap_or_else(|| {
            panic!(
                "CONTRIBUTING.md is missing the {} marker",
                REVIEW_RULE_TABLE_END
            )
        });
        assert!(
            start <= end,
            "aw:review-rule-table start marker must precede its end marker"
        );

        let spliced = contributing[start..end].trim();
        let rendered = render_review_rule_table();
        assert_eq!(
            spliced,
            rendered.trim(),
            "CONTRIBUTING.md's aw:review-rule-table block is stale; re-splice it with \
             review_doc_projection::render_review_rule_table()'s live output"
        );
    }

    /// R6: the rendered table must list every rule id from both live
    /// registries, each wrapped in backticks.
    #[test]
    fn render_review_rule_table_lists_every_known_rule_id() {
        let rendered = render_review_rule_table();
        let mut all_ids: Vec<String> = review_rules::known_rule_docs()
            .into_iter()
            .map(|r| r.id.to_string())
            .collect();
        all_ids.extend(
            review_obs_rules::known_rule_docs()
                .into_iter()
                .map(|r| r.id.to_string()),
        );
        assert!(!all_ids.is_empty(), "expected at least one known rule id");
        for id in all_ids {
            let needle = format!("`{}`", id);
            assert!(
                rendered.contains(&needle),
                "render_review_rule_table() output is missing rule id {needle}"
            );
        }
    }
}
// HANDWRITE-END
