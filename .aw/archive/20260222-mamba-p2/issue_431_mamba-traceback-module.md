---
number: 431
title: "mamba: traceback module"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #431 — mamba: traceback module

## Description

Implement `traceback` module for exception formatting and stack trace display.

## Requirements

- R1: `traceback.format_exc()` — format current exception as string
- R2: `traceback.print_exc(file=None)` — print current exception
- R3: `traceback.format_exception(exc)` — format exception chain
- R4: `traceback.format_tb(tb)` — format traceback object
- R5: `traceback.extract_tb(tb)` — extract traceback as FrameSummary list
- R6: `traceback.print_stack()` — print current stack

## Notes

Requires stack frame tracking in the runtime (file, line, function name).

## Priority

P2 — essential for debugging and error reporting.
