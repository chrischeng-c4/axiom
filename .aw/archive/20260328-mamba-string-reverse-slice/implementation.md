---
id: implementation
type: change_implementation
change_id: mamba-string-reverse-slice
---

# Implementation

## Summary

Fix string reverse slice `[::-1]` returning empty string (#1111).

## Root Cause
`mb_str_slice_full` in `string_ops.rs` passed absent `start`/`stop` sentinels through `clamp_rev_str`, which normalized the `-1` stop sentinel to `len - 1` — collapsing `s_idx == e_idx` and making the `while i > e_idx` loop immediately false.

## Fix (runtime)
- **string_ops.rs**: When `step < 0`, absent `start` defaults to `len - 1` directly (bypass `clamp_rev_str`), absent `stop` defaults to `-1` literal (bypass `clamp_rev_str`). Explicit values still go through `clamp_rev_str`.
- 14 unit tests: full reverse, single-char, empty, partial reverse, explicit start/stop, step=-2, unicode, step=0, boundary cases.

## Fix (conformance)
- **string_edge_cases.py**: Merged reverse slicing tests from xfail fixture — `s[::-1]`, `s[-2::-1]`, `s[4:1:-1]`, `s[::-2]`.
- **string_edge_cases.expected**: Added expected outputs `fedcba`, `edcba`, `edc`, `fdb`.
- **string_edge_cases_xfail.py/.expected**: Deleted — content merged into passing fixture.

## Spec update
- **string-ops.md**: Added R6 (Negative-Step Slice Defaults) requirement with decision table and 4 acceptance scenarios.

## Diff

```diff
diff --git a/crates/mamba/src/runtime/string_ops.rs b/crates/mamba/src/runtime/string_ops.rs
--- a/crates/mamba/src/runtime/string_ops.rs
+++ b/crates/mamba/src/runtime/string_ops.rs
@@ -113,8 +113,17 @@ pub fn mb_str_slice_full(
                 let ei = normalize_index(stop.as_int().unwrap_or(len), len);
                 (si, ei)
             } else {
-                let si = clamp_rev_str(start.as_int().unwrap_or(len - 1), len);
-                let ei = clamp_rev_str(stop.as_int().unwrap_or(-1), len);
+                // For absent start/stop with negative step, use literal defaults
+                // without clamping. clamp_rev_str normalizes -1 to len-1 which
+                // breaks the loop condition (e.g. s[::-1] would produce empty).
+                let si = match start.as_int() {
+                    Some(v) => clamp_rev_str(v, len),
+                    None => len - 1,
+                };
+                let ei = match stop.as_int() {
+                    Some(v) => clamp_rev_str(v, len),
+                    None => -1,
+                };
                 (si, ei)
             };
             let mut result = String::new();
@@ tests: +14 unit tests for reverse slice
+    // ── String reverse slice tests (string-reverse-slice-fix) ──
+
+    /// S1: Full reverse slice s[::-1] produces reversed string (R6, R7, R11)
+    #[test]
+    fn test_str_slice_full_reverse() {
+        let result = mb_str_slice_full(
+            s("abcdef"),
+            MbValue::none(),  // start absent
+            MbValue::none(),  // stop absent
+            MbValue::from_int(-1),
+        );
+        unsafe { assert_eq!(as_str(result), Some("fedcba")); }
+    }
+
+    /// S2: Full reverse of single-char string
+    #[test]
+    fn test_str_slice_full_reverse_single_char() {
+        let result = mb_str_slice_full(s("x"), MbValue::none(), MbValue::none(), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("x")); }
+    }
+
+    /// S3: Full reverse of empty string
+    #[test]
+    fn test_str_slice_full_reverse_empty() {
+        let result = mb_str_slice_full(s(""), MbValue::none(), MbValue::none(), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("")); }
+    }
+
+    /// S4: Partial reverse with explicit negative start — 'abcdef'[-2::-1] → 'edcba'
+    #[test]
+    fn test_str_slice_partial_reverse_explicit_start() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::from_int(-2), MbValue::none(), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("edcba")); }
+    }
+
+    /// S5: Partial reverse with explicit start and stop — 'abcdef'[4:1:-1] → 'edc'
+    #[test]
+    fn test_str_slice_partial_reverse_explicit_start_stop() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::from_int(4), MbValue::from_int(1), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("edc")); }
+    }
+
+    /// S6: Positive step slicing unaffected — 'abcdef'[1:4] → 'bcd'
+    #[test]
+    fn test_str_slice_positive_step_unaffected() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::from_int(1), MbValue::from_int(4), MbValue::from_int(1));
+        unsafe { assert_eq!(as_str(result), Some("bcd")); }
+    }
+
+    /// S7: Step=-2 skipping reverse — 'abcdef'[::-2] → 'fdb'
+    #[test]
+    fn test_str_slice_reverse_step_minus_2() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::none(), MbValue::none(), MbValue::from_int(-2));
+        unsafe { assert_eq!(as_str(result), Some("fdb")); }
+    }
+
+    /// S8: Unicode string reverse — '你好世界'[::-1] → '界世好你'
+    #[test]
+    fn test_str_slice_full_reverse_unicode() {
+        let result = mb_str_slice_full(s("你好世界"), MbValue::none(), MbValue::none(), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("界世好你")); }
+    }
+
+    /// S9: step=0 returns empty string
+    #[test]
+    fn test_str_slice_step_zero_returns_empty() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::none(), MbValue::none(), MbValue::from_int(0));
+        unsafe { assert_eq!(as_str(result), Some("")); }
+    }
+
+    /// S10: Reverse with explicit start=0, absent stop → 'abcdef'[0::-1] → 'a'
+    #[test]
+    fn test_str_slice_reverse_explicit_start_zero() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::from_int(0), MbValue::none(), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("a")); }
+    }
+
+    /// S11: Reverse with absent start, explicit stop=0 → 'abcdef'[:0:-1] → 'fedcb'
+    #[test]
+    fn test_str_slice_reverse_explicit_stop_zero() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::none(), MbValue::from_int(0), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("fedcb")); }
+    }
+
+    /// S12: Forward with absent start/stop, step=1 → 'abcdef'[::1] → 'abcdef'
+    #[test]
+    fn test_str_slice_forward_absent_start_stop() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::none(), MbValue::none(), MbValue::from_int(1));
+        unsafe { assert_eq!(as_str(result), Some("abcdef")); }
+    }
+
+    /// S13: Forward with step=2 — 'abcdef'[::2] → 'ace'
+    #[test]
+    fn test_str_slice_forward_step_two() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::none(), MbValue::none(), MbValue::from_int(2));
+        unsafe { assert_eq!(as_str(result), Some("ace")); }
+    }
+
+    /// S14: Reverse with explicit negative stop → 'abcdef'[5:-4:-1] → 'fed'
+    #[test]
+    fn test_str_slice_reverse_explicit_negative_stop() {
+        let result = mb_str_slice_full(s("abcdef"), MbValue::from_int(5), MbValue::from_int(-4), MbValue::from_int(-1));
+        unsafe { assert_eq!(as_str(result), Some("fed")); }
+    }

diff --git a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.py b/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.py
--- a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.py
+++ b/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.py
@@ -11,7 +11,12 @@
 print('a\nb\nc'.splitlines())
 # swapcase
 print('hElLo'.swapcase())
-# String slicing (forward only — reverse slicing is a separate xfail)
+# String slicing
 s = 'abcdef'
 print(s[1:4])
 print(s[::2])
+# Reverse slicing (negative step)
+print(s[::-1])
+print(s[-2::-1])
+print(s[4:1:-1])
+print(s[::-2])

diff --git a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.expected b/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.expected
--- a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.expected
+++ b/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.expected
@@ -8,3 +8,7 @@
 HeLlO
 bcd
 ace
+fedcba
+edcba
+edc
+fdb

diff --git a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.py b/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.py
deleted file mode 100644
--- a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.py
+++ /dev/null
@@ -1,4 +0,0 @@
-# mamba-xfail: string reverse slicing [::-1] returns empty string
-# String edge cases: reverse slicing
-s = 'abcdef'
-print(s[::-1])

diff --git a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.expected b/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.expected
deleted file mode 100644
--- a/crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.expected
+++ /dev/null
@@ -1 +0,0 @@
-fedcba

diff --git a/cclab/specs/crates/mamba/runtime/string-ops.md b/cclab/specs/crates/mamba/runtime/string-ops.md
--- a/cclab/specs/crates/mamba/runtime/string-ops.md
+++ b/cclab/specs/crates/mamba/runtime/string-ops.md
@@ -106,6 +106,24 @@
 
 `mb_string_encode(s, encoding)` -- encode string to `ObjData::Bytes`. Default encoding is UTF-8.
 
+### R6 - Negative-Step Slice Defaults
+
+```yaml
+id: R6
+priority: high
+```
+
+`mb_str_slice_full(s, start, stop, step)` handles `str[start:stop:step]` slicing. When `step < 0` and start/stop are absent (`None`):
+
+| Condition | Default | Rationale |
+|-----------|---------|----------|
+| `start` absent, `step < 0` | `len - 1` | Start from last character, bypass `clamp_rev_str` |
+| `stop` absent, `step < 0` | `-1` (literal) | Stop before index 0 (inclusive), bypass `clamp_rev_str` |
+| `start` explicit, `step < 0` | `clamp_rev_str(start, len)` | Normalize negative indices as before |
+| `stop` explicit, `step < 0` | `clamp_rev_str(stop, len)` | Normalize negative indices as before |
+
+**Key invariant**: `s[::-1]` produces the full reverse of `s`, matching CPython 3.12 behavior. Unicode codepoint-based iteration handles multi-byte characters correctly.
+
 ## Acceptance Criteria
@@ -137,3 +155,23 @@
+
+### Scenario: Full reverse slice
+
+- **WHEN** `'abcdef'[::-1]`
+- **THEN** Returns `'fedcba'`
+
+### Scenario: Partial reverse with negative start
+
+- **WHEN** `'abcdef'[-2::-1]`
+- **THEN** Returns `'edcba'`
+
+### Scenario: Reverse with explicit start and stop
+
+- **WHEN** `'abcdef'[4:1:-1]`
+- **THEN** Returns `'edc'`
+
+### Scenario: Unicode reverse slice
+
+- **WHEN** `'你好世界'[::-1]`
+- **THEN** Returns `'界世好你'`
```

## Review: string-reverse-slice-fix

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-string-reverse-slice

**Summary**: Implementation correctly fixes string reverse slice `[::-1]` returning empty string (#1111). All spec requirements R6–R12 are satisfied. The fix in mb_str_slice_full (string_ops.rs lines 119–126) uses None-aware match expressions: absent start defaults to len-1 directly, absent stop defaults to literal -1, bypassing clamp_rev_str in both cases. Explicit values still go through clamp_rev_str. The spec has no ## Test Plan section, so the hard reject rule does not apply — nevertheless 14 unit tests covering all scenarios (S1–S9 plus boundary cases) are present and all pass. xfail fixture deleted and conformance test passes. Main spec updated with R6 decision table.

