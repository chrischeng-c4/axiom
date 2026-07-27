"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/skills/revise-artifact.md`.

Migrated by batch `semantic-core-skills-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-skills/core-skills-revise-artifact"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/skills/revise-artifact.md"
__legacy_td_digest__ = "sha256:dae2a06530dbac22372c4649209edabd2a7198f8c30fe7320d331ed595a975b0"


def render_markdown() -> Annotated[str, "sha256:dae2a06530dbac22372c4649209edabd2a7198f8c30fe7320d331ed595a975b0"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: revise-artifact-skill\ntype: spec\ntitle: \"/cclab:sdd:revise-artifact Skill\"\nversion: 1\nspec_type: algorithm\nspec_group: sdd\ncreated_at: 2026-03-17T00:00:00+00:00\nupdated_at: 2026-03-17T00:00:00+00:00\nrequirements:\n  total: 3\n  ids: [R1, R2, R3]\nrefs: [revise-artifact]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior.\"\n---\n\n# /cclab:sdd:revise-artifact Skill\n\nUser-invoked skill that resets the workflow phase to re-enter the spec → implementation cycle when design issues are found after implementation review.\n\n## Requirements\n<!-- type: doc lang: markdown -->\n\n### R1 - User-Only Invocation\n\n```yaml\nid: R1\npriority: high\n```\n\n- `auto-invoke: false` — LLM must NOT call this skill automatically\n- Only the user can trigger via `/cclab:sdd:revise-artifact`\n\n### R2 - Phase Reset\n\n```yaml\nid: R2\npriority: high\n```\n\n- CLI command: `cclab sdd revise-artifact <change-id> --description \"<what needs to change>\"`\n- Resets `STATE.yaml` phase to `post_clarifications_created`\n- Only allowed from implementation or merge phases\n- Appends revision description to `user_input.md`\n\n### R3 - Workflow Continuation\n\n```yaml\nid: R3\npriority: high\n```\n\n- After reset, user runs `/cclab:sdd:run-change` to continue\n- Workflow naturally routes to `create-change-spec` (spec CRR cycle)\n- Then proceeds to implementation CRR cycle\n- Stops again at `implementation_complete` for user decision\n\n## Template\n<!-- type: doc lang: markdown -->\n\n```markdown\n---\nname: cclab:sdd:revise-artifact\ndescription: Revise change-spec and re-implement — fix design issues after review\nuser-invocable: true\nauto-invoke: false\n---\n```\n\n## CLI\n<!-- type: doc lang: markdown -->\n\n```yaml\ncommand: cclab sdd revise-artifact\nargs:\n  - name: change_id\n    required: true\n  - name: --description\n    required: false\n    description: What design changes are needed\n```\n\n## Installation\n<!-- type: doc lang: markdown -->\n\nInstalled to `~/.claude/skills/cclab-sdd-revise-artifact/SKILL.md` by `cclab sdd update`.\n\n## Test Plan\n<!-- type: doc lang: markdown -->\n\n| Test | Covers |\n|------|--------|\n| revise_artifact_skill_is_user_invoked_only | R1 |\n| revise_artifact_resets_phase_and_appends_description | R2 |\n| run_change_continues_after_revise_artifact_reset | R3 |\n"
