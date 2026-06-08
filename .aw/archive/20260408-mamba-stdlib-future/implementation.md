---
id: implementation
type: change_implementation
change_id: mamba-stdlib-future
---

# Implementation

## Summary

Add native __future__ module for Mamba CPython compatibility. Exports CO_FUTURE_* compiler flag constants and feature flag names (annotations, division, print_function, unicode_literals, with_statement, absolute_import) as integer values matching CPython 3.12. Register as '__future__' in stdlib.

## Diff

```diff
diff --git a/crates/mamba/src/runtime/stdlib/future_mod.rs b/crates/mamba/src/runtime/stdlib/future_mod.rs
new file mode 100644
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/future_mod.rs
@@ -0,0 +1,155 @@
+/// __future__ module for Mamba.
+///
+/// Exposes CPython 3.12 future feature flags and compiler flag constants.
+/// This is a constants-only module for compatibility — Mamba enables all
+/// modern Python features by default, so these flags are informational only.
+use std::collections::HashMap;
+use super::super::value::MbValue;
+
+// Compiler flag constants (matching CPython 3.12 code object flags)
+const CO_NESTED: i64 = 0x0010;
+const CO_GENERATOR_ALLOWED: i64 = 0;
+const CO_FUTURE_DIVISION: i64 = 0x2_0000;
+const CO_FUTURE_ABSOLUTE_IMPORT: i64 = 0x4_0000;
+const CO_FUTURE_WITH_STATEMENT: i64 = 0x8_0000;
+const CO_FUTURE_PRINT_FUNCTION: i64 = 0x10_0000;
+const CO_FUTURE_UNICODE_LITERALS: i64 = 0x20_0000;
+const CO_FUTURE_ANNOTATIONS: i64 = 0x100_0000;
+
+// @spec .score/changes/mamba-stdlib-future/groups/future-stdlib-module/specs/future-stdlib.md
+pub fn register() {
+    let mut attrs = HashMap::new();
+
+    // CO_* compiler flag constants
+    attrs.insert("CO_NESTED".to_string(), MbValue::from_int(CO_NESTED));
+    attrs.insert("CO_GENERATOR_ALLOWED".to_string(), MbValue::from_int(CO_GENERATOR_ALLOWED));
+    attrs.insert("CO_FUTURE_DIVISION".to_string(), MbValue::from_int(CO_FUTURE_DIVISION));
+    attrs.insert("CO_FUTURE_ABSOLUTE_IMPORT".to_string(), MbValue::from_int(CO_FUTURE_ABSOLUTE_IMPORT));
+    attrs.insert("CO_FUTURE_WITH_STATEMENT".to_string(), MbValue::from_int(CO_FUTURE_WITH_STATEMENT));
+    attrs.insert("CO_FUTURE_PRINT_FUNCTION".to_string(), MbValue::from_int(CO_FUTURE_PRINT_FUNCTION));
+    attrs.insert("CO_FUTURE_UNICODE_LITERALS".to_string(), MbValue::from_int(CO_FUTURE_UNICODE_LITERALS));
+    attrs.insert("CO_FUTURE_ANNOTATIONS".to_string(), MbValue::from_int(CO_FUTURE_ANNOTATIONS));
+
+    // Feature flag names
+    attrs.insert("annotations".to_string(), MbValue::from_int(CO_FUTURE_ANNOTATIONS));
+    attrs.insert("division".to_string(), MbValue::from_int(CO_FUTURE_DIVISION));
+    attrs.insert("print_function".to_string(), MbValue::from_int(CO_FUTURE_PRINT_FUNCTION));
+    attrs.insert("unicode_literals".to_string(), MbValue::from_int(CO_FUTURE_UNICODE_LITERALS));
+    attrs.insert("with_statement".to_string(), MbValue::from_int(CO_FUTURE_WITH_STATEMENT));
+    attrs.insert("absolute_import".to_string(), MbValue::from_int(CO_FUTURE_ABSOLUTE_IMPORT));
+
+    super::register_module("__future__", attrs);
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_co_future_annotations_value() { ... }
+    // ... 9 more tests
+}
diff --git a/crates/mamba/src/runtime/stdlib/mod.rs b/crates/mamba/src/runtime/stdlib/mod.rs
index fb59318e..b925c90c 100644
--- a/crates/mamba/src/runtime/stdlib/mod.rs
+++ b/crates/mamba/src/runtime/stdlib/mod.rs
@@ -89,6 +89,7 @@ pub mod tracemalloc_mod;
 pub mod ast_mod;
 pub mod dis_mod;
 pub mod tokenize_mod;
+pub mod future_mod;
 
 use std::collections::HashMap;
 use super::value::MbValue;
@@ -183,6 +184,7 @@ pub fn register_stdlib() {
     ast_mod::register();
     dis_mod::register();
     tokenize_mod::register();
+    future_mod::register();
 }

```

## Review: future-stdlib

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-stdlib-future

**Summary**: Implementation matches spec: __future__ module exports all 8 CO_* constants and 6 feature flags as integers matching CPython 3.12. 10 unit tests pass. No regressions.

