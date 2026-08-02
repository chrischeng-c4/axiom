"""Black-box contract for the agent-first CLI product model (#3306).

Drives the real, compiled `aw llm --topic model` surface in both its
Markdown and JSON output formats and cross-checks the repository's real
human-facing README, proving the live CLI product boundary affirmatively
states the current agent-first model and never advertises retired
multi-client/desktop/"cue" architecture, in both formats and in prose.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import REPOSITORY_ROOT, final_json, run_aw

CASE_ID = "aw-core-client-agent-first-cli-product-model"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-first-cli-product-model"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-agent-first-cli-product-model"
)
ASSERTIONS = (
    "aw llm --topic model, run as a real subprocess, prints Markdown containing "
    "every canonical product-responsibility phrase (agent-first "
    "project-iteration cli, next-action guidance, durable artifact skeletons, "
    "strict format, code generation) and none of the retired-architecture "
    "phrases (cue, multi-client, future client, client-independent, repo view "
    "desktop app), and the repository's real human-facing README.md likewise "
    "names none of those retired phrases -- proving the live CLI surface and "
    "the human-facing doc both currently reject the removed architecture "
    "rather than merely having once been edited to do so",
    "the same topic's --format json envelope wraps a body field whose "
    "content is identical to the --format md stdout (modulo trailing "
    "whitespace) and a summary field that is itself clean of every "
    "retired-architecture phrase -- proving the "
    "machine-readable and human-readable CLI surfaces converge on one "
    "canonical product model instead of the JSON projection silently "
    "drifting from the Markdown one",
)

_REQUIRED_PHRASES = (
    "agent-first project-iteration cli",
    "next-action guidance",
    "durable artifact skeletons",
    "strict format",
    "code generation",
)
_FORBIDDEN_PHRASES = (
    "cue",
    "multi-client",
    "future client",
    "client-independent",
    "repo view desktop app",
)


def verify() -> list[str]:
    md = run_aw(REPOSITORY_ROOT, "llm", "--topic", "model")
    md_stdout = md.stdout
    normalized_md = md_stdout.lower()
    for required in _REQUIRED_PHRASES:
        assert required in normalized_md, f"missing `{required}` in:\n{md_stdout}"
    for forbidden in _FORBIDDEN_PHRASES:
        assert forbidden not in normalized_md, f"found retired `{forbidden}` in:\n{md_stdout}"

    readme_text = (REPOSITORY_ROOT / "README.md").read_text(encoding="utf-8")
    normalized_readme = readme_text.lower()
    for forbidden in _FORBIDDEN_PHRASES:
        assert forbidden not in normalized_readme, f"README.md still names retired `{forbidden}`"

    envelope = final_json(run_aw(REPOSITORY_ROOT, "llm", "--topic", "model", "--format", "json"))
    assert envelope["topic"] == "model", envelope
    assert envelope["body"].strip() == md_stdout.strip(), (envelope["body"], md_stdout)
    normalized_summary = envelope["summary"].lower()
    for forbidden in _FORBIDDEN_PHRASES:
        assert forbidden not in normalized_summary, envelope["summary"]

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
