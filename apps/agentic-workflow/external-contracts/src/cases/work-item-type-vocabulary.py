"""Python EC implementation for terminology-first work-item vocabulary."""

from __future__ import annotations

import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import project_fixture, run_aw, verify_case


CASE_ID = "work-item-type-vocabulary"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "terminology-first-four-type-wi-taxonomy"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-type-vocabulary"
)
ASSERTIONS = (
    "aw llm defines all four work-item types by terminal state",
    "the META-doc producer projects the same canonical vocabulary",
)
TYPES = ("`epic`", "`change`", "`spike`", "`report`")
TERMINAL_TERMS = (
    "all owned children are terminal",
    "EC is green for the generated codebase",
    "ADR-style decision",
    "spawned WI refs",
    "explicit no-action",
    "`gave_up`",
    "typed `triage`",
    "`duplicate`",
    "`invalid`",
    "`by-design`",
    "intake queue",
    "spawn-and-link",
)


def assert_vocabulary(surface: str) -> None:
    for term in TYPES + TERMINAL_TERMS:
        assert term in surface, f"missing canonical work-item term: {term}"
    assert "Only `change`" in surface


def verify() -> list[str]:
    with project_fixture() as root:
        llm = run_aw(root, "llm", "--topic", "wi").stdout
        assert_vocabulary(llm)

        run_aw(root, "meta", "sync", "--repository-product")
        agents = (root / "AGENTS.md").read_text(encoding="utf-8")
        assert "### Work-item terminal states" in agents
        assert_vocabulary(agents)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
