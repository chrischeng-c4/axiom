"""Black-box contract for linear WI authoring without CRRR (#3303)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw, show


CASE_ID = "work-item-planning-wi-linear-authoring-without-crrr"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-linear-authoring-without-crrr"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-wi-linear-authoring-without-crrr"
)
ASSERTIONS = (
    "aw wi exposes no review/revise/arbitrate/runtime subcommand at all -- the live CLI surface enumerates only the linear create/fill-section/validate authoring path plus planning and lifecycle verbs",
    "a freshly authored epic goes from wi create straight to wi validate and terminates in exactly one hop with a done envelope naming validation itself as the terminal reason -- proving authoring never routes through an intermediate review or arbitration step",
    "a legacy review_count value injected into the on-disk tracker record the way an older tracker's decode would round-trips unchanged through wi show and does not block wi validate from terminating in that same one hop -- proving the field is retained for compatibility only, never consulted as an authoring gate",
    "an epic that is genuinely missing its Verification Inventory fails wi validate with passed=false and a concrete verification error -- proving the passing assertions above are not vacuous, validation can and does reject a real defect",
)


_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate linear authoring.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi validate <slug>` | validate reports passed in one hop. | - |\n"
)

_INCOMPLETE_EPIC_BODY = "## Requirements\n\n- R1: Demonstrate a genuine validation failure.\n"


def _issue_path(root: Path, slug: str) -> Path:
    raw = str(root.resolve())
    collapsed: list[str] = []
    last_dash = True
    for character in raw:
        if character.isascii() and character.isalnum():
            collapsed.append(character.lower())
            last_dash = False
        elif not last_dash:
            collapsed.append("-")
            last_dash = True
    workspace_slug = "".join(collapsed).strip("-")
    return Path("/tmp/aw/workspaces") / workspace_slug / "issues" / "open" / f"{slug}.md"


def verify() -> list[str]:
    # Cluster 1: the live CLI surface never exposed a review/arbitrate hop to
    # begin with.
    help_text = run_aw(Path("."), "wi", "--help").stdout.lower()
    command_lines = [
        line.strip()
        for line in help_text.splitlines()
        if line.strip() and not line.strip().startswith(("usage", "commands", "options"))
    ]
    first_words = {line.split()[0] for line in command_lines if line.split()}
    for retired_role in ("review", "revise", "arbitrate", "arbitration", "runtime"):
        assert retired_role not in first_words, (retired_role, first_words)

    with project_fixture() as root:
        # Cluster 2: create -> validate terminates in one hop.
        created = create(root, "Linear authoring epic", "epic", "--body", _EPIC_BODY)
        slug = created["slug"]
        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["status"] == "done", validated
        assert validated["passed"] is True, validated
        assert validated["next"] == {
            "kind": "done",
            "reason": "work-item authoring validation passed",
        }, validated

        # Cluster 3: a legacy review_count value, injected the way decoding an
        # older tracker would produce it, round-trips through show and does
        # not block a second epic's one-hop validate.
        legacy = create(root, "Legacy review-count epic", "epic", "--body", _EPIC_BODY)
        legacy_slug = legacy["slug"]
        issue_path = _issue_path(root, legacy_slug)
        raw = issue_path.read_text(encoding="utf-8")
        assert "review_count" not in raw, raw
        injected = raw.replace("phase: created\n", "phase: created\nreview_count: 2\n", 1)
        assert injected != raw, raw
        issue_path.write_text(injected, encoding="utf-8")

        shown = show(root, legacy_slug)
        assert shown["review_count"] == 2, shown

        legacy_validated = final_json(run_aw(root, "wi", "validate", legacy_slug))
        assert legacy_validated["status"] == "done", legacy_validated
        assert legacy_validated["passed"] is True, legacy_validated
        assert legacy_validated["next"]["kind"] == "done", legacy_validated

        # The legacy field is preserved through the validation write-through
        # rather than silently dropped.
        shown_after = show(root, legacy_slug)
        assert shown_after["review_count"] == 2, shown_after

        # Cluster 4 (positive control): a genuinely incomplete epic is
        # rejected, proving the one-hop "done" results above are not
        # vacuously true.
        broken = create(root, "Missing inventory epic", "epic", "--body", _INCOMPLETE_EPIC_BODY)
        broken_result = run_aw(root, "wi", "validate", broken["slug"], expect_success=False)
        broken_payload = json.loads(broken_result.stdout)
        assert broken_payload["passed"] is False, broken_payload
        assert broken_payload["errors"], broken_payload

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
