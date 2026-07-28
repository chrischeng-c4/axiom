"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/logic/python-reference-typer-cli.md`.

Migrated by batch `semantic-core-logic-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-logic/core-logic-python-reference-typer-cli"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/logic/python-reference-typer-cli.md"
__legacy_td_digest__ = "sha256:3d6d02cb300d3b0606aa6da3ccf5da6d278b78f9fe12773d02b812845be05614"


def render_markdown() -> Annotated[str, "sha256:3d6d02cb300d3b0606aa6da3ccf5da6d278b78f9fe12773d02b812845be05614"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: aw-python-reference-typer-cli\nsummary: \"Record the first DDD Typer CLI reference without generalizing it into a Python spec framework.\"\nfill_sections: [logic, unit-test, e2e-test, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: python-reference-typer-cli\n    claim: python-reference-typer-cli\n    coverage: full\n    rationale: \"The reference fixture is evidence for later TD semantics; it does not introduce a new runtime or DSL.\"\n---\n\n# Python Reference: Typer CLI\n\n## Logic\n\nThe fixture expresses one task-creation journey through ordinary Python DDD\nboundaries:\n\n`interface/cli.py` accepts the public command, `application/create_task.py`\nnormalizes and validates input, and `domain/task.py` holds the returned value.\nThe CLI owns JSON rendering and exit status; the application has no Typer or\nsubprocess dependency.\n\nObserved reusable constructs:\n\n- DDD path groups (`domain`, `application`, `interface`) provide a stable\n  narrative and keep binding code out of the application use case.\n- A callable application use case is independently unit-testable and can be\n  bound by a Typer command without adapter-specific semantics leaking inward.\n- The public boundary can map a domain/application validation error to a\n  deterministic JSON error plus a non-zero exit code.\n- Black-box EC tests run the installed CPython entrypoint in a subprocess with\n  only public arguments and stdout/stderr/exit-code observations.\n\nUnsupported assumptions, deliberately not generalized from this one example:\n\n- The fixture does not establish a Python spec framework, decorator model, or\n  `mambalibs` dependency.\n- It does not prove how HTTP routes, persistence, async execution, dependency\n  injection, or multi-command command trees should map into TD sections.\n- It does not prescribe Rust or TypeScript lowering; those require additional\n  reference projects and their own semantic evidence.\n\n## Unit Test\n\n`tests/unit/test_create_task.py` imports only the application use case and\nproves title normalization. It neither shells out nor imports the Typer CLI.\nThis is product-unit evidence, not a substitute for the external contract.\n\n## E2E Test\n\n`external-contracts/tests/test_cli_contract.py` invokes the CPython CLI as a\nsubprocess. The behavior case asserts deterministic public JSON; the security\ncase asserts that path-like input is rejected at the public boundary. These\ntests intentionally do not call the application function directly.\n\n## Changes\n\nThe reference remains a normal installable Python project under\n`apps/agentic-workflow/tests/fixtures/python_spec_typer/` with `pyproject.toml`,\nTyper, pytest, product unit tests, and independent EC tests. Its owning Rust\nfixture test is `python_spec_typer_fixture`.\n"
