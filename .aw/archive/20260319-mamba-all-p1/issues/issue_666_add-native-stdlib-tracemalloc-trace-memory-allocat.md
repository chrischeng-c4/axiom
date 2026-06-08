---
number: 666
title: "Add native stdlib: tracemalloc — trace memory allocations"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-system"
---

# #666 — Add native stdlib: tracemalloc — trace memory allocations

Implement `tracemalloc` module: `start()`, `stop()`, `get_traced_memory()`, `take_snapshot()` integrated with Mamba allocator.
