---
change: mamba-all-support
group: all-support
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: Static vs Runtime Detection
- **Question**: Should __all__ detection be static-only or also runtime?
- **Answer**: Static-only for now. Only literal list/tuple of string literals at top-level is supported (R4). Dynamic __all__ construction is deferred.

### Q2: Submodules vs Packages
- **Question**: How should star-import behave for submodules vs packages?
- **Answer**: Focus on module-level __all__. Package __init__.py __all__ controls what from pkg import * exposes — same mechanism.

### Q3: Explicit Imports
- **Question**: Should __all__ affect from X import name (explicit imports) or X.name access?
- **Answer**: No (R5). __all__ only restricts star-imports. Explicit imports and attribute access are unaffected.

### Q4: Missing Names
- **Question**: What happens when __all__ references a name that does not exist in the module?
- **Answer**: Raise AttributeError at import time (R3/AC3): 'module <name> has no attribute <missing_name>'.
