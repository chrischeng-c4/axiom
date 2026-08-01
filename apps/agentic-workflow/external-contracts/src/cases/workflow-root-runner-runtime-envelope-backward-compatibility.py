"""Black-box contract for runtime envelope backward compatibility (#3298)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-runtime-envelope-backward-compatibility"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "runtime-envelope-backward-compatibility"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-runtime-envelope-backward-compatibility"
)
ASSERTIONS = (
    "a live dispatch envelope for a fresh, open Change root's first hop carries a fully populated artifact_quality_profile object -- artifact_kind, intent_read, audience, quality_dials[].key/value, source_policy.mode, preflight_gate_set.id -- alongside every other required envelope field",
    "after the same work item is closed through the real aw wi close command, the live done envelope for that same root omits the artifact_quality_profile key entirely -- not null, structurally absent -- while every other required envelope field (schema_version, status, action, root, current, completion, next, agent_prompt, prompt_contract) remains present and correctly typed, proving a consumer unaware of the optional field keeps working",
    "the raw stdout text of both captures independently corroborates the parsed structure: the literal JSON key name appears in the open-root capture's text and is entirely absent from the closed-root capture's text",
)

_REQUIRED_ENVELOPE_KEYS = (
    "schema_version",
    "status",
    "action",
    "root",
    "current",
    "completion",
    "next",
    "agent_prompt",
    "prompt_contract",
)

_PROFILE_KEY_LITERAL = '"artifact_quality_profile"'


def _change_body(in_scope: str) -> str:
    return (
        "## Problem\n\nDemonstrate the runtime envelope's backward-compatible optional artifact quality profile.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing envelope\n"
        "Progress Evidence: the public goal wi envelope is the evidence\n\n"
        "## Requirements\n\n- R1: trace the profile field's presence and absence.\n\n"
        f"## Scope\n\n### In Scope\n- {in_scope}\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: the profile key is present when applicable and structurally absent otherwise.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| envelope-profile-trace | update | complete-platform.md |\n"
    )


def _assert_profile_schema(profile: dict[str, object]) -> None:
    assert isinstance(profile.get("artifact_kind"), str) and profile["artifact_kind"], profile
    assert isinstance(profile.get("intent_read"), str) and profile["intent_read"], profile
    assert isinstance(profile.get("audience"), str) and profile["audience"], profile
    dials = profile.get("quality_dials")
    assert isinstance(dials, list) and dials, profile
    for dial in dials:
        assert isinstance(dial.get("key"), str) and dial["key"], dial
        assert isinstance(dial.get("value"), str) and dial["value"], dial
    source_policy = profile.get("source_policy")
    assert isinstance(source_policy, dict), profile
    assert isinstance(source_policy.get("mode"), str) and source_policy["mode"], profile
    gate_set = profile.get("preflight_gate_set")
    assert isinstance(gate_set, dict), profile
    assert isinstance(gate_set.get("id"), str) and gate_set["id"], profile


def _assert_required_keys(envelope: dict[str, object]) -> None:
    for key in _REQUIRED_ENVELOPE_KEYS:
        assert key in envelope, (key, envelope)


def verify() -> list[str]:
    with project_fixture() as root:
        created = create(
            root,
            "Trace the envelope profile lifecycle",
            "change",
            "--body",
            _change_body("trace the envelope profile"),
        )
        slug = created["slug"]

        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated

        # Positive capture: a fresh, open root's first live dispatch hop.
        open_proc = run_aw(root, "goal", "wi", slug)
        open_envelope = final_json(open_proc)
        assert open_envelope["action"] != "done", open_envelope
        _assert_required_keys(open_envelope)
        assert "artifact_quality_profile" in open_envelope, open_envelope
        _assert_profile_schema(open_envelope["artifact_quality_profile"])
        assert _PROFILE_KEY_LITERAL in open_proc.stdout, open_proc.stdout

        # Close the root directly. `aw wi close` is the real, public escape
        # hatch this fixture uses to reach a genuine done envelope without
        # walking the entire EC/TD/CB lifecycle.
        closed = final_json(
            run_aw(root, "wi", "close", slug, "--reason", "fixture teardown", "--json")
        )
        assert closed["state"] == "closed", closed

        # Negative capture: the same root's done envelope.
        done_proc = run_aw(root, "goal", "wi", slug)
        done_envelope = final_json(done_proc)
        assert done_envelope["action"] == "done", done_envelope
        _assert_required_keys(done_envelope)
        assert "artifact_quality_profile" not in done_envelope, done_envelope
        assert _PROFILE_KEY_LITERAL not in done_proc.stdout, done_proc.stdout

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
