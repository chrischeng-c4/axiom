---
number: 430
title: "mamba: glob module (file pattern matching)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #430 — mamba: glob module (file pattern matching)

## Description

Implement `glob` module for Unix-style pathname pattern matching.

## Requirements

- R1: `glob.glob(pattern)` — return list of matching paths
- R2: `glob.iglob(pattern)` — lazy iterator version
- R3: Patterns: `*`, `?`, `[seq]`, `[!seq]`
- R4: `**` recursive matching (with `recursive=True`)
- R5: `glob.escape(pathname)` — escape special characters

## Priority

P2 — commonly used for batch file processing.
