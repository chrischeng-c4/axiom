"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/interfaces/src/project.md`.

Migrated by batch `semantic-surface-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-interfaces/surface-interfaces-src-project"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/interfaces/src/project.md"
__legacy_td_digest__ = "sha256:dd311002d171162db93e6d11a442afbc898d305a0f6d341e2f8cfc616a8353da"


def render_markdown() -> Annotated[str, "sha256:dd311002d171162db93e6d11a442afbc898d305a0f6d341e2f8cfc616a8353da"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-score-src-project-rs\nfill_sections: [changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Support CLI surfaces expose AW Core bootstrap, chat, hook, project, and workspace invariants.\"\n---\n\n# Standardized apps/agentic-workflow/src/cli/project.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSource TD for top-level AW health reports. `aw health` reports\nproduction readiness, managed/semantic coverage, regenerability maturity, cb\nand cold verification state, plus active WI projection locks so a pending\nTD/CB gate is visible to operators and agents.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Signature |\n|------|--------|------|------------|-----------|\n| `ProjectHealthArgs` | apps/agentic-workflow/src/cli/project.rs | struct | pub | health command args |\n| `ProjectHealthReport` | apps/agentic-workflow/src/cli/project.rs | struct | pub | health JSON report |\n| `ProjectHealthStatus` | apps/agentic-workflow/src/cli/project.rs | enum | pub | healthy/blocked |\n| `build_health_report` | apps/agentic-workflow/src/cli/project.rs | function | pub | build_health_report(project) |\n| `ProjectHealthReport::from_components` | apps/agentic-workflow/src/cli/project.rs | function | pub | aggregate coverage components |\n| `run_health` | apps/agentic-workflow/src/cli/project.rs | function | pub | run health command |\n\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/cli/project.rs\n    action: modify\n    impl_mode: codegen\n    section: source\n    description: |\n      Include workflow_lock_count and WI projection blocker summaries in\n      project health.\n```\n"
