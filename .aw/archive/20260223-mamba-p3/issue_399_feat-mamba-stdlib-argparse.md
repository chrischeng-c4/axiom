---
number: 399
title: "feat(mamba): stdlib argparse"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #399 — feat(mamba): stdlib argparse

## Summary
Implement `argparse` module for CLI argument parsing.

## Required
- `ArgumentParser(description)` constructor
- `.add_argument(name, type, default, help, required, action, nargs, choices)`
- `.parse_args(args=None)` → Namespace object
- Positional and optional arguments
- Subparsers: `.add_subparsers()`
- Auto-generated `--help`

## Implementation Notes
- Can wrap Rust `clap` crate or implement simple parser
- Namespace object is basically a dict with attribute access
