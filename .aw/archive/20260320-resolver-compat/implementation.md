---
id: implementation
type: change_implementation
change_id: resolver-compat
---

# Implementation

## Summary

Fix two remaining npm resolver compatibility bugs in `crates/cclab-jet/src/pkg_manager/resolver.rs`:

1. **#960 — Hyphen range syntax**: `normalize_npm_range()` now detects `X - Y` patterns before any other processing and expands them to comma-separated semver comparators via a new `expand_hyphen_range()` function. A helper `is_version_token()` guards against false-positive matches on package names containing hyphens (e.g. `storybook-jest`). +134 lines, 5 new tests.

2. **#957 — Bare package name as implicit npm alias** (committed in `65c5f159`): `resolve_alias()` extended to call `is_bare_package_name()` when the version spec fails semver parsing — treats it as `npm:{name}@*` and routes through existing alias resolution. +121 lines, 9 new tests.

3. **#883 — Tracking issue**: Closed by the combination of the resolver rewrite (commit `721d54d0`) and the two fixes above. No additional code changes.

## Changed Files

```
M  crates/cclab-jet/src/pkg_manager/resolver.rs
```

## Diff

### #960 — Hyphen range syntax (`normalize_npm_range` + `expand_hyphen_range`)

```diff
--- a/crates/cclab-jet/src/pkg_manager/resolver.rs
+++ b/crates/cclab-jet/src/pkg_manager/resolver.rs
@@ -570,9 +570,15 @@ pub(crate) fn parse_all_version_ranges(range: &str) -> Result<Vec<VersionReq>> {
 /// Normalize npm-style version ranges to semver crate format.
 /// - `>=1.0.0 <2.0.0` → `>=1.0.0, <2.0.0`
 /// - `1.x` or `1.*` → `>=1.0.0, <2.0.0`
+/// - `2 - 4` → `>=2.0.0, <5.0.0` (hyphen range)
 fn normalize_npm_range(range: &str) -> String {
     let trimmed = range.trim();

+    // Handle npm hyphen ranges: "X - Y"
+    if let Some(expanded) = expand_hyphen_range(trimmed) {
+        return expanded;
+    }
+
     // Handle x-ranges: 1.x, 1.*, 1.2.x

@@ -632,6 +638,69 @@ fn normalize_npm_range(range: &str) -> String {
     result
 }

+/// Expand npm hyphen range syntax `X - Y` into a comma-separated semver
+/// comparator pair understood by the `semver` crate.
+///
+/// Mapping rules (npm spec):
+/// - Y fully specified (`M.m.p`) → `>=X.0.0, <=M.m.p`
+/// - Y major.minor only (`M.m`)  → `>=X.0.0, <M.(m+1).0`
+/// - Y major only (`M`)          → `>=X.0.0, <(M+1).0.0`
+///
+/// Returns `None` if the input is not a well-formed hyphen range.
+fn expand_hyphen_range(range: &str) -> Option<String> {
+    // Require exactly one " - " separator (space-hyphen-space).
+    let mut iter = range.splitn(2, " - ");
+    let lo = iter.next()?.trim();
+    let hi = iter.next()?.trim();
+
+    // Both sides must consist of digits and dots only (no operators/letters).
+    if !is_version_token(lo) || !is_version_token(hi) {
+        return None;
+    }
+
+    let lo_parts: Vec<u64> = lo
+        .split('.')
+        .map(|s| s.parse::<u64>())
+        .collect::<Result<Vec<_>, _>>()
+        .ok()?;
+    let hi_parts: Vec<u64> = hi
+        .split('.')
+        .map(|s| s.parse::<u64>())
+        .collect::<Result<Vec<_>, _>>()
+        .ok()?;
+
+    if lo_parts.is_empty() || hi_parts.is_empty() {
+        return None;
+    }
+
+    let lo_major = lo_parts[0];
+    let lo_minor = lo_parts.get(1).copied().unwrap_or(0);
+    let lo_patch = lo_parts.get(2).copied().unwrap_or(0);
+
+    let hi_major = hi_parts[0];
+    let lower = format!(">={}.{}.{}", lo_major, lo_minor, lo_patch);
+    let upper = match hi_parts.len() {
+        1 => format!("<{}.0.0", hi_major + 1),
+        2 => {
+            let hi_minor = hi_parts[1];
+            format!("<{}.{}.0", hi_major, hi_minor + 1)
+        }
+        _ => {
+            let hi_minor = hi_parts.get(1).copied().unwrap_or(0);
+            let hi_patch = hi_parts.get(2).copied().unwrap_or(0);
+            format!("<={}.{}.{}", hi_major, hi_minor, hi_patch)
+        }
+    };
+
+    Some(format!("{}, {}", lower, upper))
+}
+
+/// Returns `true` if `s` looks like a bare version token: non-empty, composed
+/// only of ASCII digits and dots (no operators, letters, or spaces).
+fn is_version_token(s: &str) -> bool {
+    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.')
+}

@@ -1240,6 +1309,71 @@ mod tests {

+    // ── Hyphen range syntax (#960) ────────────────────────────────────────
+
+    #[test]
+    fn test_hyphen_range_major_only() { ... }       // "2 - 4" → >=2.0.0 <5.0.0
+
+    #[test]
+    fn test_hyphen_range_major_minor() { ... }      // "1.0 - 2.0" → >=1.0.0 <2.1.0
+
+    #[test]
+    fn test_hyphen_range_fully_specified() { ... }  // "1.0.0 - 2.0.0" → >=1.0.0 <=2.0.0
+
+    #[test]
+    fn test_expand_hyphen_range_rejects_non_hyphen() { ... }  // 4 rejection cases
```

### #957 — Bare package name as implicit npm alias (committed `65c5f159`)

```diff
--- a/crates/cclab-jet/src/pkg_manager/resolver.rs
+++ b/crates/cclab-jet/src/pkg_manager/resolver.rs
@@ -337,8 +337,14 @@ impl Default for DependencyResolver {
-/// Resolve npm: alias protocol.
-/// `"npm:actual-pkg@^1.0"` → `("actual-pkg", "^1.0")`
+/// Resolve npm: alias protocol and bare package name aliases.
+/// - `"storybook-jest"` (bare name, no semver) → `("storybook-jest", "*")`

 fn resolve_alias(name: &str, range: &str) -> (String, String) {
     ...
+    if is_bare_package_name(range) {
+        return (range.to_string(), "*".to_string());
+    }
     (name.to_string(), range.to_string())
 }

+fn is_bare_package_name(s: &str) -> bool { ... }  // 9 new tests
```

## Test Results

All existing `pkg_manager` tests pass. New tests added:

| Test | Issue | Covers |
|------|-------|--------|
| `test_hyphen_range_major_only` | #960 | `2 - 4` → `>=2.0.0, <5.0.0` |
| `test_hyphen_range_major_minor` | #960 | `1.0 - 2.0` → `>=1.0.0, <2.1.0` |
| `test_hyphen_range_fully_specified` | #960 | `1.0.0 - 2.0.0` → `>=1.0.0, <=2.0.0` |
| `test_expand_hyphen_range_rejects_non_hyphen` | #960 | rejects `^1.0.0`, pre-release, bare name |
| `test_resolve_alias_bare_package_name` | #957 | `"storybook-jest"` → `("storybook-jest", "*")` |
| `test_resolve_alias_bare_scoped_package_name` | #957 | `"@my-scope/actual-pkg"` handled |
| `test_is_bare_package_name_accepts_simple_names` | #957 | `lodash`, `react_dom`, `some.pkg` |
| `test_is_bare_package_name_accepts_scoped_names` | #957 | `@storybook/jest`, `@my-scope/actual-pkg` |
| `test_is_bare_package_name_rejects_semver_ranges` | #957 | `^1.0.0`, `~1.0.0`, `>=1.0.0`, `1.2.3` |
| `test_is_bare_package_name_rejects_incomplete_scoped` | #957 | `@scope/`, `@scope`, `@/name` |

## Review: resolver-compat-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: resolver-compat

**Summary**: Implementation correctly fixes both #960 (hyphen range) and #957 (bare alias). Single file changed: resolver.rs. All 96 pkg_manager tests pass.

