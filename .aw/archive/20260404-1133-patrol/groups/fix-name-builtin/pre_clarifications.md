---
change: 1133-patrol
group: fix-name-builtin
date: 2026-04-04
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Where should __name__ be set?
- **Answer**: In the driver/runtime module initialization — set __name__ = "__main__" as a global string before executing the module body. This matches CPython behavior.

