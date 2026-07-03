---
id: projects-agentic-workflow-tests-cli-tests-root-doc-mirror-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: partial
    rationale: "Root doc mirror contract (meta-doc sheet 1): executable guarantee that AGENTS.md equals CLAUDE.md outside the fixed Codex-only whitelist, replacing manual keep-both-in-sync discipline for the scanned active docs."
---

# Standardized projects/agentic-workflow/tests/cli/tests/root_doc_mirror_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/root_doc_mirror_test.rs`.

### Symbols

No public AST symbols.

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/root_doc_mirror_test.rs -->
```rust
//! Root doc mirror contract (meta-doc sheet 1).
//!
//! `AGENTS.md` must equal `CLAUDE.md` plus a fixed whitelist of Codex-only
//! insertions: the title-line swap, the `## Codex Operational Rules`
//! section, and the slash-command translation paragraph. Any divergence
//! outside that whitelist is drift — the two files are ONE implementation
//! quick-reference maintained in two agent-runtime flavors, and this test
//! replaces "remember to edit both" with an executable contract.
//!
//! The whitelist constants come from `agentic_workflow::cli::doc_mirror` —
//! the SAME module `aw init`'s AGENTS.md projection consumes (issue #984
//! AC3), so the projector and this checker can never disagree.

use agentic_workflow::cli::doc_mirror::{
    AGENTS_TITLE, CLAUDE_TITLE, CODEX_RULES_HEADING, CODEX_TRANSLATE_PREFIX,
};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root")
        .to_path_buf()
}

/// Strip the whitelisted Codex-only blocks from AGENTS.md content so the
/// remainder can be compared byte-for-byte against CLAUDE.md.
fn strip_codex_only_blocks(agents: &str) -> String {
    let mut lines: Vec<&str> = agents.lines().collect();

    // 1. Title swap (exactly one occurrence expected).
    let title_hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == AGENTS_TITLE)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        title_hits.len(),
        1,
        "AGENTS.md must contain exactly one title line `{AGENTS_TITLE}`"
    );
    lines[title_hits[0]] = CLAUDE_TITLE;

    // 2. Remove the `## Codex Operational Rules` section: from its heading
    //    up to (exclusive) the next structural line (`## ` heading or an
    //    HTML comment such as `<!-- aw:start -->`).
    let start = lines
        .iter()
        .position(|l| *l == CODEX_RULES_HEADING)
        .expect("AGENTS.md must contain the `## Codex Operational Rules` section");
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.starts_with("## ") || l.starts_with("<!--"))
        .map(|off| start + 1 + off)
        .expect("Codex Operational Rules section must be followed by a structural line");
    lines.drain(start..end);

    // 3. Remove the slash-command translation paragraph: from its first
    //    line through the following blank line (inclusive).
    let para = lines
        .iter()
        .position(|l| l.starts_with(CODEX_TRANSLATE_PREFIX))
        .expect("AGENTS.md must contain the Codex slash-command translation paragraph");
    let blank = lines[para..]
        .iter()
        .position(|l| l.trim().is_empty())
        .map(|off| para + off)
        .expect("translation paragraph must end with a blank line");
    lines.drain(para..=blank);

    let mut out = lines.join("\n");
    if agents.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[test]
fn agents_md_is_claude_md_plus_codex_whitelist() {
    let root = repo_root();
    let claude = fs::read_to_string(root.join("CLAUDE.md")).expect("read CLAUDE.md");
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");

    // The whitelisted Codex-only blocks must exist in AGENTS.md and must
    // NOT leak into CLAUDE.md.
    assert!(
        claude.contains(CLAUDE_TITLE) && !claude.contains(AGENTS_TITLE),
        "CLAUDE.md must carry its own title line only"
    );
    assert!(
        !claude.contains(CODEX_RULES_HEADING) && !claude.contains(CODEX_TRANSLATE_PREFIX),
        "Codex-only blocks must not appear in CLAUDE.md"
    );

    let normalized = strip_codex_only_blocks(&agents);
    if normalized != claude {
        let first_divergence = normalized
            .lines()
            .zip(claude.lines())
            .enumerate()
            .find(|(_, (a, c))| a != c)
            .map(|(i, (a, c))| {
                format!(
                    "line {}:\n  AGENTS(normalized): {a}\n  CLAUDE:             {c}",
                    i + 1
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "files diverge in length only (normalized AGENTS {} lines vs CLAUDE {} lines)",
                    normalized.lines().count(),
                    claude.lines().count()
                )
            });
        panic!(
            "AGENTS.md and CLAUDE.md have drifted outside the Codex-only whitelist.\n\
             Edit both files identically (the whitelist covers only the title line,\n\
             the `## Codex Operational Rules` section, and the slash-command\n\
             translation paragraph). First divergence at {first_divergence}"
        );
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/root_doc_mirror_test.rs
    action: modify
    section: source
    description: |
      Mirror-contract test agents_md_is_claude_md_plus_codex_whitelist
      (meta-doc sheet 1): the title-line swap, the Codex Operational Rules
      section, and the slash-command translation paragraph are the only
      permitted AGENTS.md/CLAUDE.md divergences; anything else fails the
      suite with the first divergent line named. Issue #984 (AC3): the
      whitelist constants now import from `agentic_workflow::cli::doc_mirror`
      instead of duplicating them as private consts, so this checker and
      `aw init`'s AGENTS.md projection share one definition.
    impl_mode: hand-written
```
