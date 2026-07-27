"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/logic/ignore-foreign-legacy-tds-during-lock-generation.md`.

Migrated by batch `semantic-logic-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:logic/logic-ignore-foreign-legacy-tds-during-lock-generation"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/logic/ignore-foreign-legacy-tds-during-lock-generation.md"
__legacy_td_digest__ = "sha256:d6bc1c4de969183b7dae3c1013841863f274e368bed3929a5ab51e0b0b92abcd"


def render_markdown() -> Annotated[str, "sha256:d6bc1c4de969183b7dae3c1013841863f274e368bed3929a5ab51e0b0b92abcd"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: '1705'\nsummary: (fill)\nfill_sections: [logic, changes, unit-test]\n---\n\n## Logic\n<!-- type: logic lang: mermaid -->\n\n```mermaid\n---\nid: td-gen-spec-resolution-contract\nentry: requested\nnodes:\n  requested:\n    kind: start\n    label: \"td gen work-item request\"\n  explicit:\n    kind: decision\n    label: \"Explicit or project-qualified TD spec available?\"\n  active:\n    kind: process\n    label: \"Use the active configured spec and its project lock\"\n  legacy:\n    kind: decision\n    label: \"A unique legacy candidate is configured?\"\n  fallback:\n    kind: terminal\n    label: \"Use configured legacy fallback only\"\n  error:\n    kind: terminal\n    label: \"Require an explicit spec path\"\nedges:\n  - { from: requested, to: explicit }\n  - { from: explicit, to: active, label: \"yes\" }\n  - { from: explicit, to: legacy, label: \"no\" }\n  - { from: legacy, to: fallback, label: \"yes\" }\n  - { from: legacy, to: error, label: \"no\" }\n---\nflowchart TD\n    requested([td gen request]) --> explicit{configured or explicit spec?}\n    explicit -->|yes| active[use active project spec + lock]\n    explicit -->|no| legacy{unique configured legacy candidate?}\n    legacy -->|yes| fallback([configured fallback])\n    legacy -->|no| error([request explicit spec])\n```\n\n`td gen` resolves an explicit or project-qualified active spec before any\nlegacy worktree discovery. Legacy fallback is valid only for a unique candidate\nunder a configured project root. A foreign `.aw` path is never selected and is\nnever passed into TD lock validation.\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/tech-design/surface/interfaces/src/td.md\n    action: modify\n    section: logic\n    impl_mode: codegen\n  - path: apps/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md\n    action: modify\n    section: unit-test\n    impl_mode: codegen\n```\n## Unit Test\n<!-- type: unit-test lang: mermaid -->\n\n```mermaid\n---\nid: td-gen-foreign-legacy-contract-verification\nrequirements:\n  configured_root_only:\n    id: R1\n    text: \"An unconfigured foreign legacy TD cannot be selected or lock-checked while generating an active configured work-item TD.\"\n    kind: functional\n    risk: medium\n    verify: cargo test -p agentic-workflow --test cli_tests test_td_gen_ignores_foreign_unconfigured_legacy_spec -- --nocapture\n---\nflowchart TD\n    r1[R1 configured root only] --> cargo_test_p_agentic_workflow_test_cli_tests_test_td_gen_ignores_foreign_unconfigured_legacy_spec_nocapture[cargo test -p agentic-workflow --test cli_tests test_td_gen_ignores_foreign_unconfigured_legacy_spec -- --nocapture]\n```\n"
