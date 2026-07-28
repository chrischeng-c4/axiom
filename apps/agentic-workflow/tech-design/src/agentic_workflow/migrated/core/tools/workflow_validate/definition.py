"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/tools/workflow_validate/definition.md`.

Migrated by batch `projection-core-tools-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-workflow-validate-definition"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/tools/workflow_validate/definition.md"
__legacy_projection_digest__ = "sha256:2a87a96ca3681ee578a903c930bed14062142ecf7b8ff338000a52223c8e9afd"


def render_markdown() -> Annotated[str, "sha256:2a87a96ca3681ee578a903c930bed14062142ecf7b8ff338000a52223c8e9afd"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-tools-workflow-validate-definition\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# sdd tools workflow validate definition\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/tools/workflow_validate.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `definition` | apps/agentic-workflow/src/tools/workflow_validate.rs | function | pub | 24 | definition() -> ToolDefinition |\n| `execute` | apps/agentic-workflow/src/tools/workflow_validate.rs | function | pub | 57 | execute(args: &Value, project_root: &Path) -> Result<String> |\n## Source\n<!-- type: source lang: rust -->\n\n````rust\n/// @spec apps/agentic-workflow/tech-design/surface/specs/three-role-contract.md#changes\npub fn definition() -> ToolDefinition {\n    ToolDefinition {\n        name: \"sdd_workflow_validate\".to_string(),\n        description: \"Validate artifact output of a score-* subagent and advance phase on pass.\"\n            .to_string(),\n        input_schema: json!({\n            \"type\": \"object\",\n            \"required\": [\"project_path\", \"change_id\", \"agent_type\"],\n            \"properties\": {\n                \"project_path\": { \"type\": \"string\" },\n                \"change_id\": { \"type\": \"string\" },\n                \"agent_type\": {\n                    \"type\": \"string\",\n                    \"enum\": [\n                        \"score-issue-author\",\n                        \"score-change-spec\",\n                        \"score-change-implementation\",\n                        \"score-review\",\n                    ]\n                }\n            }\n        }),\n    }\n}\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/workflow_validate.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:missing-generator:sdd-workflow-validate-tool-definition-json-schema>\"\n    description: \"Workflow validation MCP tool definition.\"\n```\n"
