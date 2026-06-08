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
