---
change: mamba-py312-p0
group: py312-conformance
date: 2026-03-10
status: answered
---

# Pre-Clarifications

### Q1: harness-design
- **Answer**: Golden files. Pre-generate expected outputs from CPython 3.12 and check them into the repo. CI does not need CPython installed. Provide a regen command to refresh golden files when needed.

### Q2: harness-scope
- **Answer**: Compare stdout + stderr + exception type. Full comparison including exception type for thorough conformance verification.

### Q3: mamba-invocation
- **Answer**: Rust unit tests. Call the mamba interpreter API directly within cargo test to run .py snippets. No subprocess needed for mamba side.

### Q4: object-model-depth
- **Answer**: Tests + implement. If features are missing (descriptors, metaclass, __slots__), implement them as part of this change. Don't just mark expected-failure.

### Q5: builtins-priority
- **Answer**: All builtins in one pass. Verify all ~50 builtins systematically since the harness makes it efficient.

