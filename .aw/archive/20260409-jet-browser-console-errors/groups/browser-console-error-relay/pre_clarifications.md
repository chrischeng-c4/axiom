---
change: jet-browser-console-errors
group: browser-console-error-relay
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should console.warn() be captured alongside console.error()?
- **Answer**: Yes — capture both console.error and console.warn, plus uncaught exceptions and unhandled promise rejections.

### Q2: General
- **Question**: Should there be a way to disable this feature?
- **Answer**: No — always-on in dev mode. No flag or env var needed.

