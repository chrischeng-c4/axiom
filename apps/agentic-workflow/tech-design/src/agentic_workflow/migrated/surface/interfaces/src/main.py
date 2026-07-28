"""Canonical Python producer for `apps/agentic-workflow/tech-design/surface/interfaces/src/main.md`.

Migrated by batch `projection-surface-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-interfaces/surface-interfaces-src-main"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/surface/interfaces/src/main.md"
__legacy_projection_digest__ = "sha256:88e4cbd1e0c8be0e7dfd70ba2f8c9a863e675a1a1bf3d3d10b998089d11bbbfb"


def render_markdown() -> Annotated[str, "sha256:88e4cbd1e0c8be0e7dfd70ba2f8c9a863e675a1a1bf3d3d10b998089d11bbbfb"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-score-src-main-rs\nfill_sections: [overview, changes]\ncapability_refs:\n  - id: workflow-root-runner\n    role: primary\n    gap: cli-workflow-chain\n    claim: cli-workflow-chain\n    coverage: full\n    rationale: \"CLI entrypoint and dispatch surfaces support root command parsing and workflow command routing.\"\n---\n\n# Standardized apps/agentic-workflow/src/bin/aw.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/bin/aw.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/bin/aw.rs -->\n```rust\n//! Score — local Spec-Driven Development orchestrator via Claude Code.\n//!\n//! Standalone binary entry point. Delegates to the `score` library for\n//! the `Commands` enum and `run_command` dispatch.\n\nuse anyhow::Context;\nuse clap::Parser;\nuse agentic_workflow::cli::{run_command, Commands};\n\n#[derive(Parser)]\n#[command(\n    name = \"agentic-workflow\",\n    version = env!(\"SCORE_BUILD_VERSION\"),\n    about = \"Score — local Spec-Driven Development orchestrator via Claude Code\"\n)]\nstruct Cli {\n    #[command(subcommand)]\n    command: Commands,\n}\n\nfn main() -> anyhow::Result<()> {\n    let cli = Cli::parse();\n    let rt = tokio::runtime::Runtime::new().context(\"Failed to create tokio runtime\")?;\n    rt.block_on(run_command(cli.command))?;\n    Ok(())\n}\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/bin/aw.rs\n    action: modify\n    impl_mode: codegen\n    section: source\n    description: |\n      Existing source claimed by `aw standardize managed run`. The code is\n      wrapped in a tracked HANDWRITE block until deterministic generator\n      coverage can replace it with CODEGEN.\n```\n"
