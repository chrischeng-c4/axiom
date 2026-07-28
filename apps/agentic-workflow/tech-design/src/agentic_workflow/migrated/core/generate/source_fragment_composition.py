"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/source-fragment-composition.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-source-fragment-composition"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/source-fragment-composition.md"
__legacy_td_digest__ = "sha256:8e47849d2447da2e8960e226f0bb73e80087a096cfff2acae4f45cec18847f7e"


def render_markdown() -> Annotated[str, "sha256:8e47849d2447da2e8960e226f0bb73e80087a096cfff2acae4f45cec18847f7e"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-source-fragment-composition\nfill_sections: [changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Source Fragment Composition\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nLarge claimed source files can exceed the spec markdown hard limit when a\nsingle `type: source` section owns the whole file. The `source` template must\ntherefore support multiple codegen fragments for the same Rust module without\ninventing narrower language-specific section types.\n\nThis spec adds two `replaces:` sentinel symbols for module-edge ownership and\nextends ordinary symbol replacement to `mod` blocks:\n\n- `<module-preamble>` replaces the module prefix before the first top-level\n  item, including a whole-file HANDWRITE opener, module docs, attributes, and\n  imports.\n- `<module-trailer>` replaces the suffix after the last top-level item,\n  including a whole-file HANDWRITE closer.\n- `<handwrite-gap:...>` replaces a tracked HANDWRITE region by matching its\n  `gap` attribute, including the opener and closer marker lines.\n- `mod <name>` blocks are replaceable through the existing bare symbol form,\n  so `replaces: [tests]` can own `#[cfg(test)] mod tests { ... }`.\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/apply.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - special_module_replace_range\n      - special_replace_range\n      - find_handwrite_gap_range\n      - strip_handwrite_comment_lead\n      - extract_handwrite_attr\n      - is_handwrite_open\n      - is_handwrite_close\n      - find_module_preamble_range\n      - find_module_trailer_range\n      - is_module_item_start\n      - match_item_start\n      - apply_replaces_handles_module_blocks\n      - apply_replaces_handles_module_preamble_and_trailer\n      - apply_replaces_handles_handwrite_gap_blocks\n    description: |\n      Allow `source` specs to split a large Rust module across several\n      fragments by replacing module-edge ranges, tracked HANDWRITE regions,\n      and test modules through the existing `replaces:` mechanism. This keeps\n      `section: source` as the cross-language raw template while adding\n      Rust-aware placement rules only at apply time.\n```\n\n## Reviews\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- [changes] The change strengthens template composition instead of adding a\n  language-specific section taxonomy. It is scoped to `apply.rs` replacement\n  mechanics and has direct unit coverage for the new sentinels and `mod`\n  replacement.\n"
