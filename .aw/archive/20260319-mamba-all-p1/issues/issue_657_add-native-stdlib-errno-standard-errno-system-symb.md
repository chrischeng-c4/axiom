---
number: 657
title: "Add native stdlib: errno — standard errno system symbols"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-system"
---

# #657 — Add native stdlib: errno — standard errno system symbols

Implement `errno` module: expose platform errno constants (ENOENT, EACCES, EEXIST, etc.) via `libc` crate.
