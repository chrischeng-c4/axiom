---
number: 417
title: "mamba: threading and multiprocessing modules"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #417 — mamba: threading and multiprocessing modules

## Description

Implement basic concurrency modules. Python's threading and multiprocessing are used in many applications.

## Requirements

### threading
- R1: `threading.Thread(target=fn, args=())` — create and start threads
- R2: `.start()`, `.join()`, `.is_alive()`
- R3: `threading.Lock()` — mutual exclusion
- R4: `threading.Event()` — thread signaling
- R5: `threading.current_thread()`, `threading.active_count()`

### multiprocessing (lower priority)
- R6: `multiprocessing.Process(target=fn, args=())`
- R7: `multiprocessing.Pool(n)` — process pool
- R8: `multiprocessing.Queue()` — inter-process communication

## Notes

Mamba's NaN-boxed runtime is single-threaded. Threading support may require careful design around the runtime's thread-local state and reference counting.

## Priority

P3 — important for real applications but architecturally complex.
