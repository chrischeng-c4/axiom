---
number: 446
title: "mamba: pprint module (pretty printing)"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #446 — mamba: pprint module (pretty printing)

## Description

Implement `pprint` module for readable object formatting.

## Requirements

- R1: `pprint.pprint(object, stream=None, indent=1, width=80)`
- R2: `pprint.pformat(object)` — return formatted string
- R3: `pprint.PrettyPrinter(indent, width, depth)` — configurable printer
- R4: Handles nested dicts, lists, tuples with proper indentation

## Priority

P3 — convenience for debugging, not critical.
