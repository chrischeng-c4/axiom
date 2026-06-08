---
change: gen-thread-pool
group: gen-thread-pool
date: 2026-03-26
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Single worker thread or small pool?
- **Answer**: Small pool (2-4 threads). Allows parallel list comprehensions within a single test. Nested comprehensions like `[[j for j in range(3)] for i in range(3)]` need concurrent generators.

### Q2: General
- **Question**: When to create the worker thread(s)?
- **Answer**: Lazy initialization. Create pool on first mb_generator_create() call. No overhead for tests that don't use generators.

