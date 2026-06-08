---
change: 1000-patrol
group: add-idlelib-stdlib
date: 2026-04-03
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Implementation level for idlelib?
- **Answer**: Stub-only. Provide module namespace so `import idlelib` succeeds. All functional APIs should raise NotImplementedError. No Tkinter GUI functionality needed. This is p3 low priority.

