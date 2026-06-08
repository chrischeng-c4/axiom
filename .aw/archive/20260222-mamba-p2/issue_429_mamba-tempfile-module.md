---
number: 429
title: "mamba: tempfile module"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #429 — mamba: tempfile module

## Description

Implement `tempfile` module for creating temporary files and directories.

## Requirements

- R1: `tempfile.NamedTemporaryFile(mode, suffix, prefix, dir, delete)` — temp file with name
- R2: `tempfile.TemporaryDirectory(suffix, prefix, dir)` — temp directory (auto-cleanup)
- R3: `tempfile.mkstemp(suffix, prefix, dir)` — create temp file, return (fd, name)
- R4: `tempfile.mkdtemp(suffix, prefix, dir)` — create temp directory
- R5: `tempfile.gettempdir()` — get temp directory path

## Priority

P2 — commonly used in tests and data processing.
