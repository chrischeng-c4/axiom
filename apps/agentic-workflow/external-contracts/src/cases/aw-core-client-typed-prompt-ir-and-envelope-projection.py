"""Black-box contract for the typed prompt IR/envelope projection (#2440, #3307).

Independently re-implements, in plain Python, the exact `render()` algorithm
that `agent_prompt.rs` uses to project a typed `PromptContractSpec` into the
legacy `agent_prompt` string (state/artifact/scope lines, conditional
transition/verifier lines, terminal line, guard/blocker/resume/guidance
lines, `{...}` sorted-set rendering for scope). Drives two structurally
different real `aw goal wi` envelopes -- a plain non-blocked change dispatch
and a blocked report-triage HITL dispatch -- captures each real
`prompt_contract` object, independently re-renders it in Python, and asserts
the result is byte-for-byte identical to the sibling `agent_prompt` string
from the very same real envelope. Byte-identical agreement across two
differently shaped real envelopes is what proves the typed IR is an
additive projection that retains full string-contract compatibility rather
than two independently drifting representations.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "aw-core-client-typed-prompt-ir-and-envelope-projection"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "typed-prompt-ir-and-envelope-projection"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-typed-prompt-ir-and-envelope-projection"
)
ASSERTIONS = (
    "a real, non-blocked `aw goal wi` change-dispatch envelope's typed "
    "`prompt_contract` object, independently re-rendered in plain Python "
    "using a from-scratch reimplementation of the state/artifact/scope/"
    "transition/verifier/terminal/guard line-projection algorithm, produces "
    "a string byte-for-byte identical to that same envelope's sibling "
    "`agent_prompt` string field, proving the typed IR is not a second, "
    "independently drifting representation of the legacy prompt",
    "a structurally different real, blocked `aw goal wi` report-triage "
    "envelope -- whose `prompt_contract` additionally carries `blocker` and "
    "`resume_command` fields absent from the first envelope, and lacks the "
    "artifact-quality guards the first envelope has -- re-renders through "
    "the exact same independent Python algorithm to a string that is again "
    "byte-for-byte identical to its own `agent_prompt` field, proving the "
    "additive projection generalizes across differently shaped envelopes "
    "rather than only coincidentally matching one fixed shape",
)


def _render_set(values: list[str]) -> str:
    cleaned = sorted({value.strip() for value in values if value.strip()})
    return "{" + ", ".join(cleaned) + "}"


def _render_prompt_contract(contract: dict) -> str:
    """Independent Python re-derivation of agent_prompt.rs's render()."""
    writable = _render_set(contract["scope"]["writable"])
    readonly = _render_set(contract["scope"]["readonly"])
    lines = [
        f"state := {contract['state']}",
        f"artifact := {contract['artifact']['kind']}:{contract['artifact']['id']}",
        f"scope.writable := {writable}",
        f"scope.readonly := {readonly}",
    ]

    transition = contract["transition"]
    if transition["command"].strip():
        lines.append(f"{contract['state']} -> {transition['next_state']}")
        lines.append(f"next.command := `{transition['command']}`")

    verifier = contract["verifier"]
    if verifier["command"].strip():
        lines.append(f"`{verifier['command']}` --gate-> {verifier['predicate']}")

    terminal = contract["terminal"]
    lines.append(f"terminal.{terminal['level']} --gate-> {terminal['predicate']}")

    lines.extend(f"guard := {guard}" for guard in contract["guards"])

    blocker = contract.get("blocker")
    if blocker:
        lines.append(f"blocker := {blocker['kind']}: {blocker['reason']}")

    resume = contract.get("resume_command")
    if resume:
        lines.append(f"resume := `{resume}`")

    lines.extend(f"guidance := {guidance}" for guidance in contract["guidance"])

    return "\n".join(lines)


def verify() -> list[str]:
    with project_fixture() as root:
        change = create(root, "Bare change no TD yet", "change")
        change_env = final_json(run_aw(root, "goal", "wi", change["slug"]))

        report = create(root, "Prompt projection blocked report", "report")
        report_env = final_json(run_aw(root, "goal", "wi", report["slug"]))

    assert change_env["action"] != "blocked", change_env
    change_contract = change_env["prompt_contract"]
    assert "blocker" not in change_contract or change_contract["blocker"] is None, change_contract
    assert len(change_contract["guards"]) > 3, change_contract["guards"]
    rendered_change = _render_prompt_contract(change_contract)
    assert rendered_change == change_env["agent_prompt"], (rendered_change, change_env["agent_prompt"])

    assert report_env["action"] == "blocked", report_env
    report_contract = report_env["prompt_contract"]
    assert report_contract.get("blocker") is not None, report_contract
    assert report_contract.get("resume_command"), report_contract
    assert len(report_contract["guards"]) == 3, report_contract["guards"]
    rendered_report = _render_prompt_contract(report_contract)
    assert rendered_report == report_env["agent_prompt"], (rendered_report, report_env["agent_prompt"])

    assert rendered_change != rendered_report, "the two fixtures must exercise different shapes"

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
