---
change: mamba-stdlib-future
group: future-stdlib-module
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should feature flags be simple integer constants or _Feature objects?
- **Answer**: Simple integer constants matching CPython's CO_* values. No need for _Feature class — Mamba just needs the constants for compatibility.

