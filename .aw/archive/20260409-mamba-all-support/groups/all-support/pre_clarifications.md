---
change: mamba-all-support
group: all-support
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: How should star imports interact with name resolution (resolver) given that the set of names is only known at runtime?
- **Answer**: Star imports skip symbol registration in the resolver. At runtime, mb_import_star writes names into the module globals dict. CodeGen emits a call to mb_import_star which returns a list of (name, value) pairs, then iterates and stores each as a global.

### Q2: General
- **Question**: Should __all__ support work for both file-based modules and native (FFI) modules?
- **Answer**: Yes. Both file-based modules and native modules store their attrs in MbModule.attrs. The __all__ check reads from the same attrs dict regardless of module type.

