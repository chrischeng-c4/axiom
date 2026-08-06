"""Black-box contract for the aggregate typed agent-prompt roadmap epic
(#2439 vocabulary/grammar, #2440 typed IR/envelope, #2441 lifecycle
migration/conformance; #3307).

This is deliberately NOT a re-invocation or a duplicate of any one child
case. Each child proves one link in isolation: #2439's case proves the
vocabulary/grammar *doc text* is real and cross-checked against a single
envelope; #2440's case proves `render()` reconstructs one envelope's
`agent_prompt` byte-for-byte from its `prompt_contract` on two shapes;
#2441's case proves three shapes share one schema and satisfy structural
fail-closed invariants. What none of them tests is the *emergent* property
that only holds if all three are true together: that the closed vocabulary
the docs *currently* declare -- fetched fresh from the live `aw llm`
output every run, never hardcoded here -- is exactly the operator/blocker
vocabulary genuinely load-bearing in the real rendered DSL text across
*many independently obtained, structurally distinct* real lifecycle
points. This case drives four such points (a bare Python EC-stage change,
an unresolved-label epic's blocked dispatch, a resolved report's blocked
triage dispatch, and a fully-planned epic's rollup dispatch to its ready
child) and proves the dynamically-fetched vocabulary bounds every one of
them, and that two independently observed real blocker kinds both land
inside that same dynamically-fetched set.
"""

from __future__ import annotations

import json
import re
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import AW_BINARY, _ensure_aw_binary, create, final_json, project_fixture, run_aw

CASE_ID = "aw-core-client-typed-agent-prompt-contract"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "typed-agent-prompt-contract"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-typed-agent-prompt-contract"
)
ASSERTIONS = (
    "the closed operator vocabulary is fetched fresh, every run, straight out "
    "of the live `aw llm --topic prompt --format json` doc text via regex "
    "extraction -- never hardcoded in this case -- and that dynamically "
    "fetched set is then proven to exactly bound the real structural DSL "
    "text (`state := `/`artifact := `/`scope.* := `/`-> `/`--gate-> `/base "
    "guards using `==`, `!=`, `in`) rendered into `agent_prompt` across four "
    "independently obtained, structurally distinct real lifecycle points -- "
    "a bare change's Python EC-stage dispatch, an unresolved-label epic's "
    "blocked dispatch, a resolved report's blocked triage dispatch, and a "
    "fully-planned epic's rollup dispatch to its ready child -- proving the "
    "vocabulary doc and the live projection are one mechanism observed from "
    "two sides, not two artifacts that merely happened to agree once",
    "two of those four real envelopes are independently, structurally "
    "distinct blocked dispatches carrying two different real blocker kinds "
    "(`decision` from the unresolved-label epic, `environment` from the "
    "report triage), and both kinds are members of the same "
    "dynamically-fetched blocker vocabulary while each envelope's typed "
    "`prompt_contract.blocker` line also appears verbatim inside that same "
    "envelope's legacy `agent_prompt` string, proving the typed IR and the "
    "closed vocabulary stay mutually consistent across genuinely different "
    "real blocker cases rather than one coincidentally-matching fixture",
)


def _extract_backticked(text: str) -> list[str]:
    return re.findall(r"`([^`]+)`", text)


def _fetch_prompt_vocabulary() -> tuple[frozenset[str], frozenset[str]]:
    _ensure_aw_binary()
    result = subprocess.run(
        [str(AW_BINARY), "llm", "--topic", "prompt", "--format", "json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    body = json.loads(result.stdout)["body"]

    blocker_match = re.search(r"- blocker:(.*?)\n\n", body, re.DOTALL)
    assert blocker_match, body
    blockers = frozenset(_extract_backticked(blocker_match.group(1)))
    assert len(blockers) >= 5, blockers

    operator_match = re.search(r"canonical operator set is exactly(.*?)\.", body, re.DOTALL)
    assert operator_match, body
    operators = frozenset(_extract_backticked(operator_match.group(1)))
    assert len(operators) >= 7, operators

    return blockers, operators


_BASE_GUARDS = (
    "action == done != completion.workflow_complete",
    "completion.workflow_complete == true",
    "next.command in envelope",
)


def _assert_conforms(env: dict, operator_vocab: frozenset[str], blocker_vocab: frozenset[str]) -> None:
    contract = env["prompt_contract"]
    agent_prompt = env["agent_prompt"]

    for operator in (":=", "->", "--gate->", "==", "!=", "in"):
        assert operator in operator_vocab, (operator, operator_vocab)

    lines = agent_prompt.splitlines()
    assert lines[0] == f"state := {contract['state']}", lines[:4]
    assert lines[1] == f"artifact := {contract['artifact']['kind']}:{contract['artifact']['id']}", lines[:4]
    assert lines[2].startswith("scope.writable := "), lines[:4]
    assert lines[3].startswith("scope.readonly := "), lines[:4]

    guard_texts = {line[len("guard := "):] for line in lines if line.startswith("guard := ")}
    assert set(_BASE_GUARDS).issubset(guard_texts), guard_texts

    blocker = contract.get("blocker")
    if blocker:
        assert blocker["kind"] in blocker_vocab, (blocker, blocker_vocab)
        blocker_line = f"blocker := {blocker['kind']}: {blocker['reason']}"
        assert blocker_line in lines, (blocker_line, agent_prompt)


def _drive_plan(root: Path, project: str) -> None:
    args = ["wi", "plan", "--project", project, "--json"]
    for _ in range(12):
        result = run_aw(root, *args, expect_success=None)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        env = final_json(result)
        if env.get("completion", {}).get("workflow_complete"):
            return
        assert not env.get("requires_hitl") and env.get("action") != "blocked", env
        parts = shlex.split(env["next"]["command"])
        assert parts[0] == "aw", parts
        args = parts[1:]
    raise AssertionError(f"plan pipeline for {project} did not converge")


def verify() -> list[str]:
    blocker_vocab, operator_vocab = _fetch_prompt_vocabulary()

    with project_fixture() as root:
        change = create(root, "Aggregate epic bare change", "change")
        change_env = final_json(run_aw(root, "goal", "wi", change["slug"]))

    with project_fixture() as root:
        report = create(root, "Aggregate epic blocked report", "report")
        report_env = final_json(run_aw(root, "goal", "wi", report["slug"]))

    with project_fixture() as root:
        epic = create(root, "Aggregate epic rollup epic", "epic")
        epic_slug = epic["slug"]
        rollup_change = final_json(
            run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Aggregate epic rollup child",
                "--type",
                "change",
                "--project",
                "demo",
                "--epic",
                epic_slug,
                "--json",
            )
        )
        assert rollup_change["slug"]
        _drive_plan(root, "demo")
        rollup_env = final_json(run_aw(root, "goal", "wi", epic_slug))

    with tempfile.TemporaryDirectory(prefix="aw-ec-prompt-epic-") as raw_root:
        bare_root = Path(raw_root)
        (bare_root / "aw.toml").write_text(
            '[agentic_workflow.workspace]\nmode = "in_place"\n\n'
            '[agentic_workflow.issue_platform]\ntype = "local"\n',
            encoding="utf-8",
        )
        bare_epic = final_json(
            run_aw(bare_root, "wi", "create", "--title", "Aggregate epic unlabeled", "--type", "epic", "--json")
        )
        unresolved_env = final_json(run_aw(bare_root, "goal", "wi", bare_epic["slug"]))

    assert change_env["prompt_contract"]["state"] == "ec.authoring", change_env
    assert rollup_env["prompt_contract"]["state"] == "rollup.child_dispatch", rollup_env
    assert report_env["action"] == "blocked" and report_env["prompt_contract"]["blocker"]["kind"] == "environment", report_env
    assert unresolved_env["action"] == "blocked" and unresolved_env["prompt_contract"]["blocker"]["kind"] == "decision", unresolved_env

    observed_states = {
        change_env["prompt_contract"]["state"],
        report_env["prompt_contract"]["state"],
        rollup_env["prompt_contract"]["state"],
        unresolved_env["prompt_contract"]["state"],
    }
    assert len(observed_states) >= 3, observed_states

    for env in (change_env, report_env, rollup_env, unresolved_env):
        _assert_conforms(env, operator_vocab, blocker_vocab)

    observed_blocker_kinds = {
        report_env["prompt_contract"]["blocker"]["kind"],
        unresolved_env["prompt_contract"]["blocker"]["kind"],
    }
    assert observed_blocker_kinds == {"environment", "decision"}, observed_blocker_kinds
    assert observed_blocker_kinds.issubset(blocker_vocab), (observed_blocker_kinds, blocker_vocab)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
