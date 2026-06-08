---
change: resolver-compat
group: resolver-compat
date: 2026-03-20
---

# Requirements

Fix remaining npm resolver compatibility issues in `crates/cclab-jet/src/pkg_manager/resolver.rs` so that `cclab jet install` works correctly against real-world registries (including the tech-platform Nx monorepo).

**#960 — Hyphen range syntax**
- The semver range parser does not handle npm hyphen ranges: `2 - 4`, `1.0 - 2.0`, `1.0.0 - 2.0.0`.
- Mapping rules (npm spec): `X - Y` → `>=X.0.0 <(Y+1).0.0` when Y is major-only; `>=X.0.0 <Y.m+1.0` when Y is major.minor; `>=X.0.0 <=Y.p.q` when Y is fully specified.
- Fix: detect `^\S+ - \S+$` pattern before normal semver parsing and convert to comma-separated comparator form.

**#957 — Bare package name as implicit npm alias**
- When a package-lock.json version spec is a bare npm package name (e.g. `"@storybook/expect": "storybook-jest"`), the resolver fails with `Failed to parse version range`.
- This is a valid npm pattern: bare name = implicit `npm:{name}@latest`.
- Fix: after semver parse fails, check if the spec is a valid npm package name (regex `^(@[a-z0-9-~][a-z0-9-._~]*/)?[a-z0-9-~][a-z0-9-._~]*$`); if so, rewrite to `npm:{name}@latest` and route through existing alias resolution.

**#883 — Tracking issue closure**
- #883 documents 6 resolver bugs + 3 perf improvements already implemented and verified against react-bench.
- This change should close #883 after confirming that hyphen range and bare alias fixes bring full compatibility with the tech-platform Nx monorepo.
- No new code changes expected from #883 itself unless additional regressions surface during Nx monorepo testing.

**Acceptance criteria**
- `cclab jet install` completes without parse errors on the tech-platform frontend repo.
- All 34 existing `pkg_manager` tests continue to pass.
- New unit tests for hyphen range parsing and bare alias resolution added to `resolver.rs` test module.
