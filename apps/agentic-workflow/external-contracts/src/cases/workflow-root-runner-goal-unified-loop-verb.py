"""Black-box contract for the `aw goal` unified loop verb (#3298)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-goal-unified-loop-verb"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "goal-unified-loop-verb"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-goal-unified-loop-verb"
)
ASSERTIONS = (
    "the retired `aw wi run <id>` clap leaf still parses on a real work item but never re-enters the run engine -- it exits non-zero with a structured retired_verb envelope naming the exact live `aw goal wi <id>` replacement, and that exact replacement command, run for real, produces a genuine (non-retired) workflow envelope",
    "the retired `aw capability run --project <p>` clap leaf (no capability id, the project-wide rollup form) likewise still parses but exits non-zero with a structured retired_verb envelope naming the exact live `aw goal capability --project <p> --non-interactive --max-ticks 1` replacement, and that exact replacement command, run for real, produces a genuine (non-retired) envelope rather than a second redirect",
    "the fourth leaf added by the unification, `aw goal backlog --project <p>`, is not merely aliased to the wi/capability leaves above: it is live and dispatches into real backlog-drain-specific logic that reports its own root kind, distinct from both the retired_verb redirect action and from a wi/capability root",
)


def _change_body() -> str:
    return (
        "## Problem\n\nDemonstrate that the retired wi/capability run verbs redirect through the single aw goal loop verb.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing retired-verb redirect\n"
        "Progress Evidence: the public goal/retired-verb envelopes are the evidence\n\n"
        "## Requirements\n\n- R1: trace the retired-verb redirect envelopes.\n\n"
        "## Scope\n\n### In Scope\n- trace retired wi/capability run redirects.\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: retired verbs redirect instead of re-entering the run engine.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| routing-trace | update | complete-platform.md |\n"
    )


def verify() -> list[str]:
    with project_fixture() as root:
        created = create(root, "Trace goal unified loop verb", "change", "--body", _change_body())
        slug = created["slug"]
        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated

        # Cluster 1: the retired `aw wi run <id>` leaf still parses but only
        # ever redirects -- it never re-enters the run engine itself.
        wi_redirect = final_json(run_aw(root, "wi", "run", slug, expect_success=False))
        assert wi_redirect.get("action") == "retired_verb", wi_redirect
        assert wi_redirect.get("status") == "error", wi_redirect
        expected_wi_replacement = f"aw goal wi {slug}"
        assert wi_redirect["next"]["command"] == expected_wi_replacement, wi_redirect
        assert wi_redirect["completion"]["workflow_complete"] is False, wi_redirect

        # The redirect target is a real, live command -- not just a printed
        # string -- and it produces a genuine (non-retired) envelope.
        wi_live = final_json(run_aw(root, "goal", "wi", slug))
        assert wi_live.get("action") != "retired_verb", wi_live

        # Cluster 2: the retired `aw capability run --project <p>` leaf
        # (project-wide rollup form, no capability id) behaves identically.
        cap_redirect = final_json(
            run_aw(root, "capability", "--project", "demo", "run", expect_success=False)
        )
        assert cap_redirect.get("action") == "retired_verb", cap_redirect
        assert cap_redirect.get("status") == "error", cap_redirect
        expected_cap_replacement = "aw goal capability --project demo --non-interactive --max-ticks 1"
        assert cap_redirect["next"]["command"] == expected_cap_replacement, cap_redirect

        # The redirect target is likewise real and live: whatever it decides
        # about this bare fixture project, it is never a second redirect.
        cap_live = final_json(
            run_aw(
                root,
                "goal",
                "capability",
                "--project",
                "demo",
                "--non-interactive",
                "--max-ticks",
                "1",
                expect_success=None,
            )
        )
        assert cap_live.get("action") != "retired_verb", cap_live

        # Cluster 3: the fourth, genuinely new leaf (`aw goal backlog`) is
        # live and dispatches into its own backlog-drain-specific logic,
        # distinct from both the retired_verb action and a wi/capability
        # root kind.
        backlog_envelope = final_json(
            run_aw(root, "goal", "backlog", "--project", "demo", expect_success=None)
        )
        assert backlog_envelope.get("action") != "retired_verb", backlog_envelope
        assert backlog_envelope.get("root", {}).get("kind") == "backlog", backlog_envelope

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
