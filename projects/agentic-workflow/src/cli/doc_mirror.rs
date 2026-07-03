// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/doc_mirror.md#source
// CODEGEN-BEGIN
//! Shared whitelist definition for the root `CLAUDE.md` / `AGENTS.md` mirror
//! contract (issue #984, init-projector slice 1/3).
//!
//! `AGENTS.md` is `CLAUDE.md` plus a small, fixed set of Codex-only
//! insertions: a title-line swap, the `## Codex Operational Rules` section
//! (outside the `aw:start`/`aw:end` managed block), and a slash-command
//! translation paragraph (inside the managed block). This module is the ONE
//! definition of that whitelist, consumed by both `aw init`'s AGENTS.md
//! projection (`crate::cli::init`) and `root_doc_mirror_test`, so the
//! projector and the checker can never disagree (issue #984 AC3).
//!
//! Only [`CODEX_TRANSLATE_PREFIX`], [`CODEX_TRANSLATE_PARAGRAPH`],
//! [`CODEX_TRANSLATE_ANCHOR`], and [`agents_block_from_claude_block`] are
//! actually consumed by `aw init`'s projection: the title-line and
//! `## Codex Operational Rules` constants describe content that sits
//! OUTSIDE the `aw:start` block, so init's block-replace never touches it —
//! those two are relevant only to the mirror test's outside-block
//! normalization.

/// CLAUDE.md's title line.
pub const CLAUDE_TITLE: &str = "# CLAUDE.md - Implementation Essentials";

/// AGENTS.md's title line — the only whitelisted title-line divergence.
pub const AGENTS_TITLE: &str = "# AGENTS.md - Implementation Essentials";

/// Heading of the Codex-only operational-rules section. Lives OUTSIDE the
/// `aw:start`/`aw:end` block, so `aw init`'s block-replace never touches it.
pub const CODEX_RULES_HEADING: &str = "## Codex Operational Rules";

/// First line of the Codex-only slash-command translation paragraph. Lives
/// INSIDE the `aw:start` block, so `aw init` inserts it when projecting
/// AGENTS.md.
pub const CODEX_TRANSLATE_PREFIX: &str = "Codex should translate Claude slash-command references";

/// Full text of the slash-command translation paragraph (no leading/trailing
/// blank line), inserted immediately before [`CODEX_TRANSLATE_ANCHOR`].
pub const CODEX_TRANSLATE_PARAGRAPH: &str = "Codex should translate Claude slash-command references such as `/aw:td` or\n`/aw:wi` to the equivalent `aw ...` CLI command unless the user\nexplicitly asks for Claude-specific behavior.";

/// The line the translate paragraph is inserted directly before, inside the
/// `aw:start` block (immediately after the self-AW carve-out paragraph, per
/// the current CLAUDE.md/AGENTS.md content).
pub const CODEX_TRANSLATE_ANCHOR: &str = "### Workflow CLI";

/// Project the Codex-only slash-command translation paragraph into a
/// CLAUDE.md `aw:start` block's content, producing the AGENTS.md block.
///
/// `aw init` calls this on the template-derived CLAUDE.md section to build
/// the section it upserts into AGENTS.md, so the two docs' managed blocks
/// can never drift apart by hand-editing.
///
/// # Panics
///
/// Panics if `block` does not contain [`CODEX_TRANSLATE_ANCHOR`] — the
/// `aw:start` block content is compiled from
/// `templates/cli/mainthread/CLAUDE.md`, so a missing anchor is a template
/// authoring defect, not a runtime input failure.
pub fn agents_block_from_claude_block(block: &str) -> String {
    let anchor_at = block
        .find(CODEX_TRANSLATE_ANCHOR)
        .expect("aw:start block must contain the `### Workflow CLI` heading");
    format!(
        "{}{}\n\n{}",
        &block[..anchor_at],
        CODEX_TRANSLATE_PARAGRAPH,
        &block[anchor_at..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_block_from_claude_block_inserts_paragraph_before_anchor() {
        let block = "<!-- aw:start -->\nfoo\n\n### Workflow CLI\nbar\n<!-- aw:end -->";
        let out = agents_block_from_claude_block(block);

        let para_pos = out
            .find(CODEX_TRANSLATE_PREFIX)
            .expect("translate paragraph must be present");
        let anchor_pos = out
            .find(CODEX_TRANSLATE_ANCHOR)
            .expect("anchor must still be present");
        assert!(
            para_pos < anchor_pos,
            "translate paragraph must precede the Workflow CLI anchor"
        );
        assert!(out.starts_with("<!-- aw:start -->\nfoo\n\n"));
        assert!(out.ends_with("bar\n<!-- aw:end -->"));
    }

    #[test]
    #[should_panic(expected = "Workflow CLI")]
    fn agents_block_from_claude_block_panics_without_anchor() {
        let _ =
            agents_block_from_claude_block("<!-- aw:start -->\nno anchor here\n<!-- aw:end -->");
    }
}
// CODEGEN-END
