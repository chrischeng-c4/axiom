"""Black-box contract for the WorkItem-first artifact admission gate (#3306).

Drives the real `aw td create` admission path directly: a raw-prompt slug
that names no WorkItem at all, a real but still `state:draft` WorkItem, and
finally the same WorkItem promoted to `state:open` -- proving "no artifact
before accepted WorkItem" is a genuine, state-driven gate rather than a
documentation-only invariant.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, git_commit_fixture, project_fixture, run_aw

CASE_ID = "aw-core-client-workitem-artifact-admission-gate"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "workitem-artifact-admission-gate"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-workitem-artifact-admission-gate"
)
ASSERTIONS = (
    "aw td create against a slug that names no WorkItem at all is denied with "
    "'issue ... not found in workspace', and aw td create against a real but "
    "still state:draft WorkItem is denied with 'issue ... is state:draft, must "
    "be state:open before starting tech-design' -- neither denial scaffolds a "
    "tech-design/ tree on disk, proving raw-prompt and unaccepted-WorkItem "
    "artifact requests are genuinely blocked before any artifact is persisted "
    "rather than merely warned about",
    "once that same WorkItem is promoted to state:open via aw wi validate, the "
    "identical aw td create <slug> command is admitted and scaffolds a real "
    "WI-bound Python TD module skeleton whose __aw_work_item__ marker names "
    "that exact WorkItem -- proving the prior denials were a genuine "
    "accepted-WorkItem admission gate rather than a general failure, and that "
    "admission is state-driven rather than static",
)

_CHANGE_BODY = (
    "## Problem\n\nDemonstrate the WorkItem-first artifact admission gate.\n\n"
    "## Requirements\n\n"
    "- R1: aw td create is admitted only once the owning WorkItem is accepted.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw td create <slug>` | td create dispatches once the WI reaches "
    "state:open. | - |\n"
)


def verify() -> list[str]:
    with project_fixture() as root:
        git_commit_fixture(root)
        td_root = root / "tech-design"

        # Cluster 1a: a raw-prompt slug naming no WorkItem at all is denied.
        missing = run_aw(
            root, "td", "create", "does-not-exist", "--project", "demo", expect_success=False
        )
        assert "not found in workspace" in missing.stderr, missing.stderr
        assert not td_root.exists(), "denied raw-prompt request scaffolded tech-design/"

        # Create a real WorkItem but leave it unaccepted -- it starts state:draft.
        created = create(root, "Gate Recon Change", "change", "--body", _CHANGE_BODY)
        slug = created["slug"]

        # Cluster 1b: the same command against a real, unaccepted WorkItem is
        # denied for a distinct, state-specific reason.
        draft_attempt = run_aw(
            root, "td", "create", slug, "--project", "demo", expect_success=False
        )
        assert "state:draft" in draft_attempt.stderr, draft_attempt.stderr
        assert (
            "must be state:open before starting tech-design" in draft_attempt.stderr
        ), draft_attempt.stderr
        assert not td_root.exists(), "denied draft-state request scaffolded tech-design/"

        # Cluster 2: accept the WorkItem (state:draft -> state:open) and prove
        # the identical td create is now admitted.
        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated

        admitted = final_json(run_aw(root, "td", "create", slug, "--project", "demo"))
        assert admitted["action"] == "dispatch", admitted
        source_path = admitted["artifact"]["source_path"]
        module = root / source_path
        assert module.exists(), f"admitted td create did not scaffold {module}"
        contents = module.read_text(encoding="utf-8")
        assert f'__aw_work_item__ = "{slug}"' in contents, contents

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
