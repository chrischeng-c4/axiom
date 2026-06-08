---
change: mamba-p1-bugfix
group: iter-two-arg
date: 2026-03-28
---

# Requirements

Implement iter(callable, sentinel) two-argument form. Currently only iter(iterable) single-arg is supported. Add: when iter() receives 2 args, call first arg repeatedly until return value equals sentinel, yield each non-sentinel value.
