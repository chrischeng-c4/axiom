---
change: all-open-jet-issues
group: jet-install-optimizations
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Cache Location
- **Answer**: Use ~/.jet-store/ as the default location for simplicity and consistency with current development proposals (see #881).

### Q2: Lock-only Mode
- **Answer**: Yes, implement a lock-only mode (e.g., via a --no-install flag) to support workflows that only require lockfile updates without the overhead of full tarball downloads.

