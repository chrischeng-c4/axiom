---
change: mamba-test-coverage
group: stdlib-coverage
date: 2026-03-08
---

# Requirements

Improve stdlib module coverage from 56.9% to 80-95%. 72 modules total, 29 have only 1 test, 3 have zero tests (dataclasses, enum, time). Worst: configparser_mod (9.8%), difflib_mod (12.3%), hmac_mod (12.3%), heapq_mod (13.9%), enum_mod (15.7%).
