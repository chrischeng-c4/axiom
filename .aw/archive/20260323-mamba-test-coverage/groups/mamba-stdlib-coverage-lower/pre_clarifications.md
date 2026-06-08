---
change: mamba-test-coverage
group: mamba-stdlib-coverage-lower
date: 2026-03-23
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Follow existing stdlib spec groupings. The 10 target files map to ~6-7 existing spec groups (e.g. queue_mod → concurrency, bisect/statistics → numeric, lzma/zlib → archive-and-compression). Produce one change-spec per existing spec group, not one per file.

### Q2: General
- **Answer**: Both inline #[cfg(test)] unit tests at the bottom of each *_mod.rs AND integration tests in crates/mamba/tests/. Inline tests cover per-function branch paths; integration tests cover cross-module interactions.

### Q3: General
- **Answer**: Use real C bindings with small fixed in-memory byte arrays. No mocks, no external files. Example: compress a known 16-byte payload, verify decompressed output matches original.

### Q4: General
- **Answer**: Test everything, no exclusions. No coverage exclusion annotations. Find a way to exercise every branch including platform-specific guards. If a branch is truly dead code, remove it rather than annotating it.

