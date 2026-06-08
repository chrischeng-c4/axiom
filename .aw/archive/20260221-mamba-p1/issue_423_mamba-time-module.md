---
number: 423
title: "mamba: time module"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #423 — mamba: time module

## Description

Implement `time` module — one of the most commonly used stdlib modules.

## Requirements

- R1: `time.time()` — current time as float (seconds since epoch)
- R2: `time.sleep(secs)` — suspend execution
- R3: `time.monotonic()` — monotonic clock for measuring intervals
- R4: `time.perf_counter()` — high-resolution performance counter
- R5: `time.strftime(format, t=None)` — format time as string
- R6: `time.strptime(string, format)` — parse time string
- R7: `time.localtime(secs=None)` — convert to struct_time
- R8: `time.gmtime(secs=None)` — convert to UTC struct_time
- R9: `time.mktime(t)` — convert struct_time to seconds
- R10: `time.process_time()` — CPU time

## Priority

P1 — almost every non-trivial program needs timing.
