"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/logic/lens-yaml-codegen.md`.

Migrated by batch `semantic-core-logic-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-logic/core-logic-lens-yaml-codegen"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/logic/lens-yaml-codegen.md"
__legacy_td_digest__ = "sha256:e63d0af617839bdc27f2f281e3f21e43ebab4ec3d60b9cb0dbbc527c2383909c"


def render_markdown() -> Annotated[str, "sha256:e63d0af617839bdc27f2f281e3f21e43ebab4ec3d60b9cb0dbbc527c2383909c"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: lens-yaml-codegen\ntype: spec\ntitle: \"Lens YAML-Based Code Generation\"\nversion: 1\nspec_type: integration\ntags: [external]\nspec_group: cclab-lens\ncreated_at: 2026-02-14T17:26:42.192461+00:00\nupdated_at: 2026-02-14T17:26:42.192461+00:00\nrequirements:\n  total: 3\n  ids:\n    - R1\n    - R2\n    - R3\ndesign_elements:\n  has_mermaid: true\n  has_json_schema: false\n  has_pseudo_code: false\n  has_api_spec: false\n  has_semantic_diagrams: false\n  diagrams:\n    - type: sequence\n      title: \"Codegen Flow\"\nhistory:\n  - timestamp: 2026-02-14T17:26:42.192461+00:00\n    agent: \"mcp\"\n    tool: \"create_spec\"\n    action: \"created\"\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"This codegen logic TD supports CB lifecycle generation and regenerable artifact production.\"\n---\n\n<spec>\n\n# Lens YAML-Based Code Generation\n\n## Overview\n<!-- type: doc lang: markdown -->\n\nUpdates Lens to read SpecIR YAML manifests from disk and dispatch them to the appropriate CodeGenerator implementation. This replaces the direct Rust struct injection used previously.\n\n## Requirements\n<!-- type: doc lang: markdown -->\n\n### R1 - YAML Reader\n\n```yaml\nid: R1\npriority: medium\nstatus: draft\n```\n\nLens must provide a reader that deserializes YAML files into the `SpecManifest` struct defined in the schema spec.\n\n### R2 - Generic Generator Input\n\n```yaml\nid: R2\npriority: medium\nstatus: draft\n```\n\nThe `CodeGenerator` trait must be updated (or wrapped) to accept `SpecManifest` input, allowing generators to consume the standard IR format.\n\n### R3 - Generator Dispatch\n\n```yaml\nid: R3\npriority: medium\nstatus: draft\n```\n\nLens must dispatch the parsed manifest to the correct generator based on the `kind` field and the target language configuration.\n\n## Acceptance Criteria\n<!-- type: doc lang: markdown -->\n\n### Scenario: Generate from YAML\n\n- **WHEN** Lens is invoked with valid YAML IR paths\n- **THEN** Code is generated successfully matching the spec content\n\n### Scenario: Invalid YAML Format\n\n- **WHEN** Lens encounters a malformed YAML file\n- **THEN** An error is returned describing the parsing failure\n\n### Scenario: Unsupported Kind\n\n- **WHEN** Lens encounters a manifest with an unknown kind\n- **THEN** An error is returned stating no generator found for kind\n\n## Diagrams\n<!-- type: doc lang: markdown -->\n\n### Codegen Flow\n\n```mermaid\nsequenceDiagram\n    participant SDD as SDD\n    participant Lens as Lens\n    participant YamlReader as YamlReader\n    participant CodeGenerator as CodeGenerator\n    SDD->>Lens: generate(spec_ir_paths)\n    Lens->>YamlReader: read_manifest(path)\n    YamlReader->>Lens: SpecManifest\n    Lens->>CodeGenerator: generate(manifest)\n    CodeGenerator->>Lens: GeneratedCode\n    Lens->>SDD: Result\n```\n\n</spec>\n"
