"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/spec-validator.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-spec-validator"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/spec-validator.md"
__legacy_td_digest__ = "sha256:2c7e3ac763d0f07a40754fdc4f63d68f45932aa9489a222abc032e60cc8f7db6"


def render_markdown() -> Annotated[str, "sha256:2c7e3ac763d0f07a40754fdc4f63d68f45932aa9489a222abc032e60cc8f7db6"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: spec-validator\ntype: spec\ntitle: \"Spec Completeness Validator\"\nversion: 1\nspec_type: algorithm\ncreated_at: 2026-02-02T13:49:55.191928+00:00\nupdated_at: 2026-02-03T10:50:00.000000+00:00\nrequirements:\n  total: 3\n  ids:\n    - R1\n    - R2\n    - R3\ndesign_elements:\n  has_mermaid: true\n  has_json_schema: false\n  has_pseudo_code: false\n  has_api_spec: false\n  has_semantic_diagrams: false\n  diagrams:\n    - type: flowchart\n      title: \"Spec Validation Logic\"\nhistory:\n  - timestamp: 2026-02-02T13:49:55.191928+00:00\n    agent: \"mcp\"\n    tool: \"create_spec\"\n    action: \"created\"\n  - timestamp: 2026-02-03T10:50:00.000000+00:00\n    agent: \"gemini\"\n    action: \"merged\"\n    message: \"Full rewrite from generate-codegen change\"\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Codegen TDs support CB lifecycle generation and regenerable artifact production.\"\n---\n\n<spec>\n\n# Spec Completeness Validator\n\n## Overview\n<!-- type: doc lang: markdown -->\n\nDefines the validation logic for JSON Schemas within the Generate Codegen System. It ensures that schemas are structurally sound, have valid references, and contain sufficient metadata (like descriptions) for high-quality code generation.\n\n## Requirements\n<!-- type: doc lang: markdown -->\n\n### R1 - Type Validation\n\n```yaml\nid: R1\npriority: medium\nstatus: draft\n```\n\nThe validator must check that all properties have a defined type (or $ref).\n\n### R2 - Reference Validation\n\n```yaml\nid: R2\npriority: medium\nstatus: draft\n```\n\nThe validator must ensure all $ref pointers resolve to existing definitions.\n\n### R3 - Completeness Check\n\n```yaml\nid: R3\npriority: medium\nstatus: draft\n```\n\nThe validator should warn if descriptions are missing for public fields.\n\n## Acceptance Criteria\n<!-- type: doc lang: markdown -->\n\n### Scenario: Missing Type\n\n- **GIVEN** A schema with a missing type for 'age'\n- **WHEN** The validator is run\n- **THEN** The validator returns an error indicating 'age' has no type\n\n### Scenario: Broken Reference\n\n- **GIVEN** A schema with a $ref to '#/definitions/Unknown'\n- **WHEN** The validator is run\n- **THEN** The validator returns an error for the broken reference\n\n## Diagrams\n<!-- type: doc lang: markdown -->\n\n### Spec Validation Logic\n\n```mermaid\nflowchart TB\n    Start((Start))\n    CheckStructure{Check Structure (Types)} \n    CheckRefs{Check References ($ref)} \n    CheckCompleteness{Check Completeness (Desc)} \n    Success(Validation Passed)\n    Error[Validation Error]\n    Warning[Validation Warning]\n    Start -->|Input Schema| CheckStructure\n    CheckStructure -->|Valid| CheckRefs\n    CheckStructure -->|Invalid| Error\n    CheckRefs -->|All Found| CheckCompleteness\n    CheckRefs -->|Missing Ref| Error\n    CheckCompleteness -->|Complete| Success\n    CheckCompleteness -->|Missing Fields| Warning\n```\n\n</spec>\n"
