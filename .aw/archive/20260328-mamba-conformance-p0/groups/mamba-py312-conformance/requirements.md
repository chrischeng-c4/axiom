---
change: mamba-conformance-p0
group: mamba-py312-conformance
date: 2026-03-25
---

# Requirements

Automated conformance test runner: cclab mamba test --conformance. Test fixtures that run identically on CPython 3.12 and Mamba. Output diff comparison — any difference is a conformance bug. All list, dict, set, tuple, str, bytes methods produce same results as CPython 3.12. Generator send/throw/close match CPython behavior. StopIteration.value propagation matches CPython. Iterator protocol __iter__/__next__ matches CPython.
