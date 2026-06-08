---
change: resolver-compat
group: resolver-compat
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Follow npm semantics exactly: 1.0 - 2.0 → >=1.0.0 <2.1.0 (partial right-hand = exclusive next). 1.0.0 - 2.0.0 → >=1.0.0 <=2.0.0 (full right-hand = inclusive).

### Q2: General
- **Answer**: Pin concrete version in lockfile, same as normal alias. Re-resolve only when lockfile is absent or --no-lockfile.

### Q3: General
- **Answer**: Yes, handle scoped packages too. If spec doesn't parse as semver and looks like a package name (starts with @ or lowercase alpha), treat as bare alias.

### Q4: General
- **Answer**: Close #883 after tech-platform Nx monorepo install succeeds. No other remaining work — all original bugs were addressed.

### Q5: General
- **Answer**: Commit a trimmed fixture to tests/fixtures/. Manual reproduction is not reliable for CI.

