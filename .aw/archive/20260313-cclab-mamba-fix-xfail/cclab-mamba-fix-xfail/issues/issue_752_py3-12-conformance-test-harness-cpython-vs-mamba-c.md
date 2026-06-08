---
number: 752
title: "Py3.12 conformance test harness — CPython vs mamba comparison framework"
state: open
labels: [enhancement, P0, crate:mamba]
group: "mamba-conformance-xfail"
---

# #752 — Py3.12 conformance test harness — CPython vs mamba comparison framework

## Parent

Part of #750

## Goal

Build a test harness that runs the same test case in CPython 3.12 and mamba, then compares results to verify conformance.

## Requirements

- [ ] Runner that executes a `.py` snippet in both CPython 3.12 and mamba
- [ ] Output comparison: stdout, stderr, exit code, exception type
- [ ] Report format: pass/fail/diff per test case
- [ ] Integration with `cargo test` (snapshot-based or golden-file)
- [ ] Support for expected-failure annotations (known divergences)

## Notes

This is the foundation for all other conformance sub-issues. Without it, conformance testing is manual and unreliable.
