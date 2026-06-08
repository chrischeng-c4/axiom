---
id: implementation
type: change_implementation
change_id: mamba-stdlib-main
---

# Implementation

## Summary

Add native __main__ module for Mamba CPython compatibility. Registers __main__ with standard module attributes (__name__, __doc__, __loader__, __spec__) so that top-level scripts behave consistently with CPython. The __name__ attribute is set to "__main__" as a string; __doc__, __loader__, and __spec__ are None.

## Diff

```diff
diff --git a/crates/mamba/src/runtime/stdlib/main_mod.rs b/crates/mamba/src/runtime/stdlib/main_mod.rs
new file mode 100644
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/main_mod.rs
@@ -0,0 +1,72 @@
+/// __main__ module for Mamba.
+///
+/// Provides the top-level script execution module. In CPython, `__main__` is
+/// the module where top-level code runs. The `if __name__ == "__main__"` idiom
+/// checks this attribute to distinguish script execution from module import.
+use std::collections::HashMap;
+use super::super::rc::MbObject;
+use super::super::value::MbValue;
+
+// @spec .score/changes/mamba-stdlib-main/groups/stdlib-main-module/specs/mamba-stdlib-main-spec.md
+pub fn register() {
+    let mut attrs = HashMap::new();
+
+    // __name__ = "__main__"
+    attrs.insert(
+        "__name__".to_string(),
+        MbValue::from_ptr(MbObject::new_str("__main__".to_string())),
+    );
+
+    // __doc__ = None
+    attrs.insert("__doc__".to_string(), MbValue::none());
+
+    // __loader__ = None
+    attrs.insert("__loader__".to_string(), MbValue::none());
+
+    // __spec__ = None
+    attrs.insert("__spec__".to_string(), MbValue::none());
+
+    super::register_module("__main__", attrs);
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_main_name_value() { ... }
+
+    #[test]
+    fn test_main_doc_is_none() { ... }
+
+    #[test]
+    fn test_main_loader_is_none() { ... }
+
+    #[test]
+    fn test_main_spec_is_none() { ... }
+
+    #[test]
+    fn test_register_module() { ... }
+}
diff --git a/crates/mamba/src/runtime/stdlib/mod.rs b/crates/mamba/src/runtime/stdlib/mod.rs
index b925c90c..606a4518 100644
--- a/crates/mamba/src/runtime/stdlib/mod.rs
+++ b/crates/mamba/src/runtime/stdlib/mod.rs
@@ -90,6 +90,7 @@ pub mod ast_mod;
 pub mod dis_mod;
 pub mod tokenize_mod;
 pub mod future_mod;
+pub mod main_mod;
 
 use std::collections::HashMap;
 use super::value::MbValue;
@@ -185,6 +186,7 @@ pub fn register_stdlib() {
     dis_mod::register();
     tokenize_mod::register();
     future_mod::register();
+    main_mod::register();
 }

```

## Review: mamba-stdlib-main-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-stdlib-main

**Summary**: Implementation matches spec: __main__ module exports __name__ = "__main__" as string and __doc__, __loader__, __spec__ as None, matching CPython behavior. 5 unit tests pass. Module wired into stdlib via mod.rs. No regressions (SIGABRT in full test suite is pre-existing JIT crash #1187).
