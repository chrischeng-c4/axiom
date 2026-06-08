---
change: mamba-conformance-xfail
group: conformance-xfail-reduction
date: 2026-03-26
---

# Requirements

Reduce the 48 xfail conformance tests by fixing implementation gaps in the Mamba JIT pipeline. Current breakdown: 30 output mismatch (runtime behavior differs from CPython), 5 Cranelift verifier errors (kwargs argument count mismatch in codegen), 6 other (compilation/runtime failures), 3 parse errors (unsupported syntax). Target: fix low-hanging fruit across all categories to maximize xfail reduction.
