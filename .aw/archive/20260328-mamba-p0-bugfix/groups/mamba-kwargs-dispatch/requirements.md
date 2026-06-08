---
change: mamba-p0-bugfix
group: mamba-kwargs-dispatch
date: 2026-03-25
---

# Requirements

sorted(key=len, reverse=True), print(sep='-', end=''), min(key=fn, default=val), int('ff', base=16), str.format(name='Bob') must all produce CPython 3.12-identical results. User-defined keyword-only args (def f(*, key)) must work.
