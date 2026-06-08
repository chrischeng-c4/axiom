---
id: implementation
type: change_implementation
change_id: fix-bare-pkg-alias
---

# Implementation

## Summary

Fix in crates/cclab-jet/src/pkg_manager/resolver.rs (1 file, +121/-2 lines): (1) resolve_alias() extended to detect bare package names as implicit npm alias version specs — when the version field fails semver parsing and is a syntactically valid npm package name (e.g. "@storybook/expect": "storybook-jest"), it is treated as npm:storybook-jest@* and delegated to the existing alias resolution path; (2) is_bare_package_name() new private function — validates that a string is a valid npm package name (rejects semver operators, digit-leading strings, malformed scoped names); (3) 9 unit tests added: test_resolve_alias_bare_package_name, test_resolve_alias_bare_scoped_package_name, test_is_bare_package_name_accepts_simple_names, test_is_bare_package_name_accepts_scoped_names, test_is_bare_package_name_rejects_semver_ranges, test_is_bare_package_name_rejects_incomplete_scoped, plus additional edge-case tests for empty input and path-like strings.

## Diff

```diff
diff --git a/crates/cclab-jet/src/pkg_manager/resolver.rs b/crates/cclab-jet/src/pkg_manager/resolver.rs
index 8cb8be09..c9872258 100644
--- a/crates/cclab-jet/src/pkg_manager/resolver.rs
+++ b/crates/cclab-jet/src/pkg_manager/resolver.rs
@@ -337,8 +337,14 @@ impl Default for DependencyResolver {
     }
 }
 
-/// Resolve npm: alias protocol.
-/// `"npm:actual-pkg@^1.0"` → `("actual-pkg", "^1.0")`
+/// Resolve npm: alias protocol and bare package name aliases.
+///
+/// Handles:
+/// - `"npm:actual-pkg@^1.0"` → `("actual-pkg", "^1.0")`
+/// - `"npm:@scope/pkg@^1.0"` → `("@scope/pkg", "^1.0")`
+/// - `"npm:actual-pkg"` → `("actual-pkg", "*")`
+/// - `"storybook-jest"` (bare name, no semver) → `("storybook-jest", "*")`
+///
 /// Regular deps pass through unchanged.
 fn resolve_alias(name: &str, range: &str) -> (String, String) {
     if let Some(alias_spec) = range.strip_prefix("npm:") {
@@ -353,9 +359,65 @@ fn resolve_alias(name: &str, range: &str) -> (String, String) {
         // npm:actual-pkg (no version)
         return (alias_spec.to_string(), "*".to_string());
     }
+    // Bare package name as implicit npm alias.
+    // e.g. `"@storybook/expect": "storybook-jest"` means "install
+    // storybook-jest instead" — equivalent to `npm:storybook-jest@latest`.
+    // Treat as the aliased package at any version ("*" = highest stable).
+    if is_bare_package_name(range) {
+        return (range.to_string(), "*".to_string());
+    }
     (name.to_string(), range.to_string())
 }
 
+/// Returns `true` when `s` is a valid npm package name rather than a version
+/// range.  Used to detect implicit npm-alias specs such as
+/// `"@storybook/expect": "storybook-jest"`.
+///
+/// A bare package name:
+/// - Contains no semver operators (`^`, `~`, `>`, `<`, `=`, `|`, ` `, `*`).
+/// - Does not start with a digit (version numbers like `1.2.3` do).
+/// - For scoped packages (`@scope/name`): non-empty scope and name separated
+///   by `/`.
+/// - All characters are alphanumeric, `-`, `_`, or `.` (plus `@`/`/` for
+///   scoped names).
+fn is_bare_package_name(s: &str) -> bool {
+    if s.is_empty() {
+        return false;
+    }
+    // Any semver operator or whitespace → version range, not a package name.
+    if s.chars()
+        .any(|c| matches!(c, '^' | '~' | '>' | '<' | '=' | '|' | ' ' | '*'))
+    {
+        return false;
+    }
+    // Starts with digit → version number (e.g. "1.2.3", "1.x").
+    if s.starts_with(|c: char| c.is_ascii_digit()) {
+        return false;
+    }
+    if let Some(rest) = s.strip_prefix('@') {
+        // Scoped package: @scope/name — both parts must be non-empty.
+        let mut parts = rest.splitn(2, '/');
+        let scope = parts.next().unwrap_or("");
+        let pkg = parts.next().unwrap_or("");
+        if scope.is_empty() || pkg.is_empty() {
+            return false;
+        }
+        scope
+            .chars()
+            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
+            && pkg
+                .chars()
+                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
+    } else {
+        // Unscoped: no `/` allowed (would look like a path).
+        if s.contains('/') {
+            return false;
+        }
+        s.chars()
+            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
+    }
+}
+
 /// Check if a package should be skipped based on platform (os/cpu).
 pub fn should_skip_optional(
     version_meta: &super::registry::VersionMetadata,
@@ -871,6 +933,63 @@ mod tests {
         assert_eq!(range, "^18.0.0");
     }
 
+    /// Bare package name as version spec is treated as an implicit npm alias.
+    /// e.g. `"@storybook/expect": "storybook-jest"` → install storybook-jest@*.
+    #[test]
+    fn test_resolve_alias_bare_package_name() {
+        let (name, range) =
+            resolve_alias("@storybook/expect", "storybook-jest");
+        assert_eq!(name, "storybook-jest");
+        assert_eq!(range, "*");
+    }
+
+    /// Bare scoped package name as version spec.
+    #[test]
+    fn test_resolve_alias_bare_scoped_package_name() {
+        let (name, range) =
+            resolve_alias("some-alias", "@my-scope/actual-pkg");
+        assert_eq!(name, "@my-scope/actual-pkg");
+        assert_eq!(range, "*");
+    }
+
+    // ── is_bare_package_name ──────────────────────────────────────────────
+
+    #[test]
+    fn test_is_bare_package_name_accepts_simple_names() {
+        assert!(is_bare_package_name("storybook-jest"));
+        assert!(is_bare_package_name("lodash"));
+        assert!(is_bare_package_name("react_dom"));
+        assert!(is_bare_package_name("some.pkg"));
+    }
+
+    #[test]
+    fn test_is_bare_package_name_accepts_scoped_names() {
+        assert!(is_bare_package_name("@storybook/jest"));
+        assert!(is_bare_package_name("@my-scope/actual-pkg"));
+    }
+
+    #[test]
+    fn test_is_bare_package_name_rejects_semver_ranges() {
+        assert!(!is_bare_package_name("^1.0.0"));
+        assert!(!is_bare_package_name("~1.0.0"));
+        assert!(!is_bare_package_name(">=1.0.0"));
+        assert!(!is_bare_package_name("1.2.3"));
+        assert!(!is_bare_package_name("*"));
+        assert!(!is_bare_package_name(""));
+        assert!(!is_bare_package_name("^1.0.0 || ^2.0.0"));
+        assert!(!is_bare_package_name(">=1.0.0 <2.0.0"));
+    }
+
+    #[test]
+    fn test_is_bare_package_name_rejects_incomplete_scoped() {
+        // Missing name after slash
+        assert!(!is_bare_package_name("@scope/"));
+        // Missing slash entirely
+        assert!(!is_bare_package_name("@scope"));
+        // Empty scope
+        assert!(!is_bare_package_name("@/name"));
+    }
+
     /// Pre-release fallback: when no stable version satisfies the range,
     /// find_best_version must fall back to a pre-release.
     #[test]

```

## Review: fix-bare-pkg-alias-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: fix-bare-pkg-alias

**Summary**: Bare package name alias handling added to resolver. 376 tests pass.

