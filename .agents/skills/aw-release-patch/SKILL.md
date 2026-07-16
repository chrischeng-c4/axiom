---
name: aw:release-patch
description: Deprecated legacy AW release path. Use aw:build:release for release builds or aw:build:debug for debug installs.
user-invocable: true
---

# /aw:release-patch

Deprecated. Do not use this legacy direct tag path.

Use:

- `/aw:build:debug aw` for a local debug install.
- `/aw:build:release aw` for a release that lands through `git:land`, pushes
  `aw@<version>`, and monitors GitHub release publication.

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
