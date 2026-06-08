---
number: 665
title: "Add native stdlib: concurrent.futures — async execution"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-io-networking"
---

# #665 — Add native stdlib: concurrent.futures — async execution

Implement `concurrent.futures`: `ThreadPoolExecutor`, `ProcessPoolExecutor`, `Future`, `as_completed()`, `wait()` backed by tokio/rayon.
