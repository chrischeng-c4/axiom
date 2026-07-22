---
id: aw-python-reference-typer-cli
summary: "Record the first DDD Typer CLI reference without generalizing it into a Python spec framework."
fill_sections: [logic, unit-test, e2e-test, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: python-reference-typer-cli
    claim: python-reference-typer-cli
    coverage: full
    rationale: "The reference fixture is evidence for later TD semantics; it does not introduce a new runtime or DSL."
---

# Python Reference: Typer CLI

## Logic

The fixture expresses one task-creation journey through ordinary Python DDD
boundaries:

`interface/cli.py` accepts the public command, `application/create_task.py`
normalizes and validates input, and `domain/task.py` holds the returned value.
The CLI owns JSON rendering and exit status; the application has no Typer or
subprocess dependency.

Observed reusable constructs:

- DDD path groups (`domain`, `application`, `interface`) provide a stable
  narrative and keep binding code out of the application use case.
- A callable application use case is independently unit-testable and can be
  bound by a Typer command without adapter-specific semantics leaking inward.
- The public boundary can map a domain/application validation error to a
  deterministic JSON error plus a non-zero exit code.
- Black-box EC tests run the installed CPython entrypoint in a subprocess with
  only public arguments and stdout/stderr/exit-code observations.

Unsupported assumptions, deliberately not generalized from this one example:

- The fixture does not establish a Python spec framework, decorator model, or
  `mambalibs` dependency.
- It does not prove how HTTP routes, persistence, async execution, dependency
  injection, or multi-command command trees should map into TD sections.
- It does not prescribe Rust or TypeScript lowering; those require additional
  reference projects and their own semantic evidence.

## Unit Test

`tests/unit/test_create_task.py` imports only the application use case and
proves title normalization. It neither shells out nor imports the Typer CLI.
This is product-unit evidence, not a substitute for the external contract.

## E2E Test

`external-contracts/tests/test_cli_contract.py` invokes the CPython CLI as a
subprocess. The behavior case asserts deterministic public JSON; the security
case asserts that path-like input is rejected at the public boundary. These
tests intentionally do not call the application function directly.

## Changes

The reference remains a normal installable Python project under
`apps/agentic-workflow/tests/fixtures/python_spec_typer/` with `pyproject.toml`,
Typer, pytest, product unit tests, and independent EC tests. Its owning Rust
fixture test is `python_spec_typer_fixture`.
