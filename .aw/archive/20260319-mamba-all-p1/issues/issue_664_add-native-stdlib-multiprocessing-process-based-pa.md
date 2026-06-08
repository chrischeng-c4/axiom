---
number: 664
title: "Add native stdlib: multiprocessing — process-based parallelism"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-io-networking"
---

# #664 — Add native stdlib: multiprocessing — process-based parallelism

Implement `multiprocessing` module: `Process`, `Pool`, `Queue`, `Pipe`, `Value`, `Array` backed by std::process.
