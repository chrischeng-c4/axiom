"""Black-box contract for lifecycle-wide prompt-contract conformance (#2441, #3307).

Drives three structurally distinct real lifecycle points through
`aw goal wi`/`aw wi plan` -- a bare Python-stage (EC) change dispatch, a
blocked report-triage HITL dispatch, and a fully-planned epic's rollup
dispatch to its one ready child change -- and proves each of the promise's
five named concerns (Python stages, rollup, HITL blockers, artifact
quality, CB ownership) is carried by the *same* typed `prompt_contract`
projection, plus proves that projection fails closed: every one of the
three differently shaped real contracts independently satisfies the same
structural well-formedness invariants (non-overlapping scope, non-empty
terminal predicate, the three base guards always present, a blocker always
paired with a resume command) rather than each lifecycle point growing its
own bespoke, unvalidated shape.
"""

from __future__ import annotations

import shlex
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "aw-core-client-lifecycle-prompt-migration-and-conformance"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "lifecycle-prompt-migration-and-conformance"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-lifecycle-prompt-migration-and-conformance"
)
ASSERTIONS = (
    "three structurally distinct real lifecycle points -- a bare change's "
    "Python EC-stage dispatch (`state == ec.authoring`), a report's blocked "
    "HITL triage dispatch (`blocker.kind == environment` with a paired "
    "`resume_command`), and a fully-planned epic's rollup dispatch to its "
    "one ready child (`state == rollup.child_dispatch`, `verifier.predicate "
    "== child.change.closed == true`) -- each carry the same typed "
    "`prompt_contract` schema, proving Python stages, HITL blockers, and "
    "rollup are three faces of one projection rather than three separate "
    "mechanisms",
    "the change and epic contracts both carry artifact-quality hard "
    "preflight guards (`artifact_quality.code-artifact-test == green`, "
    "`artifact_quality.code-artifact-spec-annotation == green`) backed by a "
    "real `artifact_quality_profile` whose `ownership_markers=required` "
    "quality dial is CB's source-ownership gate, while the report contract "
    "correctly carries neither (proving the profile is conditionally "
    "derived, not hardcoded), and all three real contracts independently "
    "satisfy the same fail-closed structural invariants -- non-overlapping "
    "scope, a non-empty terminal predicate, the three base guards always "
    "present, and any blocker always paired with a resume command -- proving "
    "one typed projection enforces its own well-formedness uniformly across "
    "every lifecycle point rather than silently emitting a malformed prompt",
)

_BASE_GUARDS = {
    "action == done != completion.workflow_complete",
    "completion.workflow_complete == true",
    "next.command in envelope",
}


def _assert_well_formed(contract: dict) -> None:
    assert contract["schema_version"] == "aw.prompt.v1", contract
    assert contract["state"], contract
    assert contract["artifact"]["kind"], contract
    assert contract["artifact"]["id"], contract
    assert contract["terminal"]["predicate"], contract
    writable = set(contract["scope"]["writable"])
    readonly = set(contract["scope"]["readonly"])
    assert not (writable & readonly), contract["scope"]
    guards = set(contract["guards"])
    assert _BASE_GUARDS.issubset(guards), guards
    blocker = contract.get("blocker")
    if blocker:
        assert blocker.get("kind"), blocker
        assert blocker.get("reason"), blocker
        assert contract.get("resume_command"), contract
    for guard in contract["guards"]:
        assert "→" not in guard and "⇒" not in guard, guard


def _drive_plan(root: Path, project: str) -> dict:
    args = ["wi", "plan", "--project", project, "--json"]
    for _ in range(12):
        result = run_aw(root, *args, expect_success=None)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        env = final_json(result)
        if env.get("completion", {}).get("workflow_complete"):
            return env
        invoke = env.get("invoke", {}).get("command")
        assert invoke, env
        parts = shlex.split(invoke)
        assert parts[0] == "aw", parts
        args = parts[1:]
    raise AssertionError("project plan pipeline did not converge")


def verify() -> list[str]:
    with project_fixture() as root:
        change = create(root, "Bare change no TD yet", "change")
        change_env = final_json(run_aw(root, "goal", "wi", change["slug"]))

    with project_fixture() as root:
        report = create(root, "Lifecycle conformance blocked report", "report")
        report_env = final_json(run_aw(root, "goal", "wi", report["slug"]))

    with project_fixture() as root:
        epic = create(root, "Lifecycle conformance rollup epic", "epic")
        epic_slug = epic["slug"]
        rollup_change = final_json(
            run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Lifecycle conformance rollup child",
                "--type",
                "change",
                "--project",
                "demo",
                "--epic",
                epic_slug,
                "--json",
            )
        )
        change_slug = rollup_change["slug"] if "slug" in rollup_change else rollup_change["issue"]["slug"]
        plan_final = _drive_plan(root, "demo")
        assert plan_final["completion"]["workflow_complete"], plan_final
        epic_env = final_json(run_aw(root, "goal", "wi", epic_slug))

    change_contract = change_env["prompt_contract"]
    report_contract = report_env["prompt_contract"]
    epic_contract = epic_env["prompt_contract"]

    assert change_contract["state"] == "ec.authoring", change_contract
    assert change_env["action"] != "blocked", change_env

    assert report_env["action"] == "blocked", report_env
    assert report_contract["blocker"]["kind"] == "environment", report_contract
    assert report_contract["resume_command"], report_contract

    assert epic_contract["state"] == "rollup.child_dispatch", epic_contract
    assert epic_contract["verifier"]["predicate"] == "child.change.closed == true", epic_contract
    assert epic_contract["artifact"]["id"] == change_slug, epic_contract
    assert "child.done != root.complete" in epic_contract["guards"], epic_contract["guards"]

    quality_guards = {
        "artifact_quality.code-artifact-test == green",
        "artifact_quality.code-artifact-spec-annotation == green",
    }
    assert quality_guards.issubset(set(change_contract["guards"])), change_contract["guards"]
    assert quality_guards.issubset(set(epic_contract["guards"])), epic_contract["guards"]
    assert not quality_guards & set(report_contract["guards"]), report_contract["guards"]
    assert "artifact_quality_profile" not in report_env, report_env

    for env in (change_env, epic_env):
        dials = {d["key"]: d for d in env["artifact_quality_profile"]["quality_dials"]}
        assert dials["ownership_markers"]["value"] == "required", dials
        assert "ownership" in dials["ownership_markers"]["rationale"], dials

    for contract in (change_contract, report_contract, epic_contract):
        _assert_well_formed(contract)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
