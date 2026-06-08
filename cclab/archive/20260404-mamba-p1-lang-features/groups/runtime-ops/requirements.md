---
change: mamba-p1-lang-features
group: runtime-ops
date: 2026-04-04
---

# Requirements

Runtime operation additions: (1) Tuple comparison operators (lt, le, gt, ge) — lexicographic comparison in tuple_ops.rs. (2) type() 3-arg dynamic class creation — type(name, bases, dict) in builtins.rs. Both are runtime-only changes.
