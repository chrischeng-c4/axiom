## Goal

Running `aw wi validate` on a GHAN body reports section errors instead of `body must contain structured work-item sections`.

## How

### Verified premises

- `apps/agentic-workflow/src/cli/issues.rs:2176` pushes the unstructured error and early-returns.
- `apps/agentic-workflow/src/services/issue_parser.rs:245` hard-requires the legacy problem and requirements headings.

### Change points

- `apps/agentic-workflow/src/cli/issues.rs` — route by body shape.
- `apps/agentic-workflow/src/issues/ghan.rs` — the validator itself.

### Frozen decisions

- The legacy six-section shape stays valid; this is coexistence, not replacement.

## Acceptance

| # | Command | Current | Target | Why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `cargo test -p agentic-workflow --lib -- --test-threads=1` | 3755 passed / 0 failed | 3770 passed / 0 failed | the new cases assert refusal strings that do not exist before the change |

### Negative control

Delete the shape branch in `validate_publishable_issue_body`. Re-run the gate; the new cases must fail.
Restore the file byte-for-byte to sha256 `59d66dea106b9bd7c8c319d9096f1e5fe1c82957faa4837a8fa8c7cd6528a32b`.

## Never

The addressee of these limits is the agent executing this work item, not the dispatcher.

### Must not touch

- `apps/agentic-workflow/external-contracts/src/wi_contract_fixture.py`

### Must not do

- Do not relax an existing assertion to make the gate green.
- Do not narrow the test selector so it matches nothing.
