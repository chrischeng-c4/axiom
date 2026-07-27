"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/specs/codegen-skip-annotation-convention.md`.

Migrated by batch `semantic-surface-specs-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-specs/surface-specs-codegen-skip-annotation-convention"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/specs/codegen-skip-annotation-convention.md"
__legacy_td_digest__ = "sha256:bb2fc2295e91efdbd437f5001d98a1f5f72156172968c946cdfb876182d20549"


def render_markdown() -> Annotated[str, "sha256:bb2fc2295e91efdbd437f5001d98a1f5f72156172968c946cdfb876182d20549"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: codegen-skip-annotation-convention\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior.\"\n---\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: /Users/chris.cheng/cclab/main/apps/agentic-workflow/src/tools/analyze/mod.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    anchor: \"//! \"\n    description: |\n      Insert above the existing `//!` module-level doc lines (or as the first\n      lines if none exist). Three new lines:\n        //! @codegen-skip: test-fixture-only\n        //! Reason: `pub struct/enum` matches in this file (e.g. line 372:\n        //! `pub struct Config`) are inside r#\"...\"# raw strings used as test\n        //! fixtures fed to `rust_lang::analyze`, not real Rust type definitions.\n\n  - path: /Users/chris.cheng/cclab/main/apps/agentic-workflow/src/cli/fillback.rs\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    anchor: \"//! \"\n    description: |\n      Same annotation as above. Three lines:\n        //! @codegen-skip: test-fixture-only\n        //! Reason: `pub struct/enum` matches in this file (e.g. line 127:\n        //! `pub struct Config`) are inside r#\"...\"# raw strings used as test\n        //! fixtures, not real Rust type definitions.\n\n  - path: /Users/chris.cheng/cclab/main/CLAUDE.md\n    action: modify\n    section: changes\n    impl_mode: hand-written\n    anchor: \"## Constraints\"\n    description: |\n      Append a new bullet under `## Constraints`, after the codegen-era\n      file-size three-class bullet:\n\n        - **Dogfood scan convention** (codegen markers vs skip annotation):\n          Files whose `pub struct/enum` matches are inside test fixtures\n          (`r#\"...\"#` raw strings) should carry a top-of-file\n          `//! @codegen-skip: test-fixture-only` annotation with a\n          one-paragraph reason. Future scans for remaining dogfood\n          candidates should filter these out:\n\n          ```bash\n          find <paths> -name \"*.rs\" \\\n            | xargs grep -L 'CODEGEN-BEGIN\\|@codegen-skip' \\\n            | xargs grep -lE \"^pub (struct|enum) \"\n          ```\n\n          Result is the list of unmarked, undecided files. Apply the\n          `@codegen-skip` marker to confirmed fixture-only files; run\n          `aw td gen-code` on files with real type definitions.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Accurately identifies both target files and the three-part change scope; no corrections needed.\n- [changes] All three entries carry concrete anchors and exact insertion text; the CLAUDE.md bash snippet is self-contained and correct.\n"
