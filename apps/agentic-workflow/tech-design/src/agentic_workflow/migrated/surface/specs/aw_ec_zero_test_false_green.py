"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/specs/aw-ec-zero-test-false-green.md`.

Migrated by batch `semantic-surface-specs-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-specs/surface-specs-aw-ec-zero-test-false-green"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/specs/aw-ec-zero-test-false-green.md"
__legacy_td_digest__ = "sha256:a974a00e9d89a389a17b4beaf911b471a01c5d2b006578ac34b57c6dae911f42"


def render_markdown() -> Annotated[str, "sha256:a974a00e9d89a389a17b4beaf911b471a01c5d2b006578ac34b57c6dae911f42"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: aw-ec-zero-test-false-green\nsummary: Reject cargo-test EC commands that pass after running zero tests.\ncapability_refs:\n  - id: project-local-td-and-ec-gates\n    role: primary\n    gap: ec-false-green-guard\n    claim: ec-false-green-guard\n    coverage: full\n    rationale: \"EC verification must fail when a cargo test filter proves no tests, and EC inventory should keep precise cargo target selectors when a unit target is known.\"\n---\n\n# AW EC Zero-Test False Green\n\n`aw ec verify` and generated Rust EC wrappers must not treat a successful\n`cargo test` process as proof when cargo reports that it ran zero tests. EC case\ncommands generated or retained for Rust unit-test filters should carry explicit\ntarget selectors such as `--lib` when that target is known, instead of relying\non crate-wide filters.\n\n## External Contract\n<!-- type: e2e-test lang: yaml -->\n\n```yaml\ne2e_tests:\n  - id: aw-ec-zero-test-false-green\n    capability_id: project-local-td-and-ec-gates\n    claim_id: ec-false-green-guard\n    contract_id: aw-ec-zero-test-false-green\n    category: behavior\n    command: \"cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture\"\n    assertions:\n      - \"aw ec verify marks a cargo test command failed when the command exits 0 after running zero tests\"\n      - \"generated Rust EC wrappers capture stdout and reject the same zero-test false green\"\n      - \"ec gen keeps precise cargo test target selectors instead of relying on crate-wide filters when the source contract provides one\"\n```\n"
