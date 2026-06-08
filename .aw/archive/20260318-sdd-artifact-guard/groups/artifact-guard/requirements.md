---
change: sdd-artifact-guard
group: artifact-guard
date: 2026-03-18
---

# Requirements

Add delegation guard check to artifact CLI entry point. When delegation_guard is active in STATE.yaml, only allow artifact commands that match guard.action. Reject all other artifact commands with a clear error message. Map CLI kebab-case action names to guard snake_case names for comparison.
