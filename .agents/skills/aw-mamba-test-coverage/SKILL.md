---
name: aw:mamba:test-coverage
description: Analyze mamba test coverage — total tests, distribution, per-module stdlib detail, line ratio
user-invocable: true
---

# /aw:mamba:test-coverage

Runs the test coverage analysis script for the mamba package
(`projects/mamba`, formerly `crates/cclab-mamba`). Inventory-only: it
calls `cargo test -p mamba -- --list` and walks `*.rs` files on disk.
A full test run is **not** required.

## Instructions

Run the script:

```bash
.agents/skills/aw-mamba-test-coverage/scripts/coverage.sh
```

Present the output to the user as-is. If the user asks for more detail on a specific area, read the relevant test or source files.

## AW CLI Drift & Defect Reporting

`aw` changes frequently. If this skill's documented invocation, result shape, or
semantics contradict the current `aw --help` output or CLI envelope, treat that
as a suspected AW defect; do not silently invent a compatibility command or
work around it.

Before reporting, reproduce the smallest failing command and capture the `aw`
version, exact command, expected result, actual stdout/stderr, and any relevant
envelope fields. Confirm the current surface with the relevant `aw <verb>
--help`; when working on AW itself, prefer a freshly built
`target/debug/aw` if the installed binary could be stale.

Once confirmed, report an AW-owned defect with `aw issue create --title "aw:
<short symptom>" "<reproduction and evidence>"`. Do not pass `--yes` unless
GitHub writes are already authorized. Expected validation failures or defects
owned by the target project belong in that project's tracker, not as AW bugs.
