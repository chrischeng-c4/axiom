"""Black-box contract for the typed agent-prompt vocabulary/grammar (#2439, #3307).

Drives the real `aw llm --topic prompt --format json` and proves its JSON
body carries a genuine closed vocabulary and closed ASCII grammar -- not
inert prose -- by cross-checking every declared blocker-vocabulary term and
every declared grammar operator against a second, independently obtained
real `aw goal wi` envelope that actually renders a blocked prompt using
those exact tokens.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import AW_BINARY, _ensure_aw_binary, create, final_json, project_fixture, run_aw

CASE_ID = "aw-core-client-prompt-vocabulary-and-grammar"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "prompt-vocabulary-and-grammar"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-prompt-vocabulary-and-grammar"
)
ASSERTIONS = (
    "a real `aw llm --topic prompt --format json` emits well-formed JSON "
    "with non-empty project/topic/summary/body fields whose body declares a "
    "closed vocabulary (truth: unknown/red/green; terminal level: stage "
    "terminal/change closed/root complete; owner: EC/TD/CB; blocker: "
    "decision/approval/environment/red_gate/missing_evidence) and states "
    "the canonical ASCII grammar operator set is exactly `->`, `--gate->`, "
    "`:=`, `==`, `!=`, `in`, and `notin`",
    "every one of those 5 declared blocker-vocabulary terms and every one "
    "of those 7 declared grammar operators is independently confirmed to "
    "be genuinely load-bearing by cross-checking against a real, separately "
    "obtained `aw goal wi` envelope: an actually blocked report-triage "
    "dispatch's `prompt_contract.blocker.kind` is a member of the declared blocker set "
    "and its rendered `agent_prompt` text uses only declared operators, "
    "proving the vocabulary/grammar doc describes tokens the runtime "
    "genuinely emits rather than aspirational, unused documentation",
)

_DECLARED_BLOCKERS = ("decision", "approval", "environment", "red_gate", "missing_evidence")
_DECLARED_OPERATORS = ("->", "--gate->", ":=", "==", "!=", "in", "notin")


def verify() -> list[str]:
    _ensure_aw_binary()
    result = subprocess.run(
        [str(AW_BINARY), "llm", "--topic", "prompt", "--format", "json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    payload = json.loads(result.stdout)
    assert payload["topic"] == "prompt", payload
    assert payload["project"], payload
    assert payload["summary"], payload
    body = payload["body"]
    assert body, payload

    assert "## Closed vocabulary" in body, body
    assert "## Closed ASCII grammar" in body, body
    for term in _DECLARED_BLOCKERS:
        assert f"`{term}`" in body, (term, body)
    for operator in _DECLARED_OPERATORS:
        assert f"`{operator}`" in body, (operator, body)
    assert (
        "canonical operator set is exactly `->`, `--gate->`, `:=`, `==`, "
        "`!=`,\n`in`, and `notin`" in body
    ), body

    with project_fixture() as root:
        report = create(root, "Prompt vocabulary blocked report", "report")
        slug = report["slug"]
        goal = run_aw(root, "goal", "wi", slug)
        env = final_json(goal)

    assert env["action"] == "blocked", env
    contract = env["prompt_contract"]
    blocker = contract["blocker"]
    assert blocker["kind"] in _DECLARED_BLOCKERS, blocker
    assert blocker["kind"] == "environment", blocker

    agent_prompt = env["agent_prompt"]
    assert ":=" in agent_prompt, agent_prompt
    assert "-> hitl" in agent_prompt, agent_prompt
    assert "--gate->" in agent_prompt, agent_prompt
    assert "==" in agent_prompt, agent_prompt
    assert "!=" in agent_prompt, agent_prompt
    assert f"blocker := {blocker['kind']}:" in agent_prompt, agent_prompt
    used_undeclared = [token for token in ("=>",) if token in agent_prompt]
    assert not used_undeclared, (used_undeclared, agent_prompt)
    for line in agent_prompt.splitlines():
        if "-->" in line or " -> " in line or ":=" in line:
            for banned in ("→", "⇒"):
                assert banned not in line, line

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
