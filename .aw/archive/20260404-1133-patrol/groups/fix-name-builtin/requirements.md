---
change: 1133-patrol
group: fix-name-builtin
date: 2026-04-04
---

# Requirements

Fix __name__ builtin to resolve to "__main__" (string) instead of 0.0 (float) when a script is the entry point. The compiler or runtime needs to set __name__ as a global string variable before executing the module body.
