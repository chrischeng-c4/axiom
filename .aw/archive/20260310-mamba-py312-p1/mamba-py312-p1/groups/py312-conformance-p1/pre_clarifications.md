---
change: mamba-py312-p1
group: py312-conformance-p1
date: 2026-03-10
status: answered
---

# Pre-Clarifications

### Q1: Data structure priority
- **Answer**: Focus on list/dict first. Cover list and dict thoroughly with all major ops, then set/tuple/str with basic ops. Most practical impact for conformance.

### Q2: Exception scope
- **Answer**: xfail ExceptionGroup and except* (PEP 654) entirely. Focus on core exception hierarchy, raise from chaining (__cause__, __context__), custom subclassing, and args attribute.

### Q3: Generator completeness
- **Answer**: Fixtures first, fix later. Write all conformance test fixtures for yield, yield from, send, throw, close, StopIteration.value. Mark what doesn't work as xfail. Fix runtime in this or next pass. Async generators also xfail.

