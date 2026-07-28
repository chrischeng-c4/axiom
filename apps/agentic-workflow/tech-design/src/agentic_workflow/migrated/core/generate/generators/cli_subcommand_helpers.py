"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/cli_subcommand_helpers.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-cli-subcommand-helpers"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/cli_subcommand_helpers.md"
__legacy_projection_digest__ = "sha256:42d1a1aa3339dfb2ed034f2fb0a9a0161265de8ae7ba92d7667a9bbb3a02d091"


def render_markdown() -> Annotated[str, "sha256:42d1a1aa3339dfb2ed034f2fb0a9a0161265de8ae7ba92d7667a9bbb3a02d091"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-cli-subcommand-helpers\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# CLI Subcommand Helper Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/cli_subcommand.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CliArg` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | struct | pub | 18 |  |\n| `CliArgKind` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | enum | pub | 44 |  |\n| `CliCommand` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | struct | pub | 53 |  |\n| `CliEmitted` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | struct | pub | 83 |  |\n| `emit_cli_subcommand` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | function | pub | 150 | emit_cli_subcommand(cmd: &CliCommand) -> CliEmitted |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap missing-generator:struct-with-derives -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/cli_subcommand.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:missing-generator:struct-with-derives>\"\n    description: \"Source template owns CliEmitted and CLI subcommand helper functions.\"\n```\n"
