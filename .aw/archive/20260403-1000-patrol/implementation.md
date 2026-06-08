---
id: implementation
type: change_implementation
change_id: 1000-patrol
---

# Implementation

## Summary

Add native `idlelib` stdlib stub module. Registers `idlelib` as a package with standard
attributes and 13 stub submodule/function entries. All stub functions raise Python-level
`NotImplementedError` via `mb_raise()` (not `panic!`). Includes `#[cfg(test)]` module with
`test_register` and `test_stub_raises` tests.

## Changed Files

```
A	crates/mamba/src/runtime/stdlib/idlelib_mod.rs
M	crates/mamba/src/runtime/stdlib/mod.rs
```

## Diff Statistics

```
crates/mamba/src/runtime/stdlib/idlelib_mod.rs | 182 +++++++++++++++++++
crates/mamba/src/runtime/stdlib/mod.rs         |   2 +
 2 files changed, 184 insertions(+)
```

## Diff

```diff
diff --git a/crates/mamba/src/runtime/stdlib/idlelib_mod.rs b/crates/mamba/src/runtime/stdlib/idlelib_mod.rs
new file mode 100644
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/idlelib_mod.rs
@@ -0,0 +1,182 @@
+/// idlelib module for Mamba (mamba-stdlib).
+///
+/// Stub implementation of Python's `idlelib` package.
+/// Registers the `idlelib` module namespace so `import idlelib` succeeds.
+/// All functional APIs raise `NotImplementedError` — no Tkinter GUI functionality.
+use std::collections::HashMap;
+use super::super::value::MbValue;
+use super::super::rc::MbObject;
+
+pub fn register() {
+    let mut attrs = HashMap::new();
+
+    // Standard module attributes
+    attrs.insert("__name__".to_string(), MbValue::from_ptr(MbObject::new_str("idlelib".to_string())));
+    attrs.insert("__file__".to_string(), MbValue::from_ptr(MbObject::new_str("idlelib/__init__.py".to_string())));
+    attrs.insert("__package__".to_string(), MbValue::from_ptr(MbObject::new_str("idlelib".to_string())));
+    attrs.insert("__path__".to_string(), MbValue::from_ptr(MbObject::new_list(vec![])));
+
+    // Stub submodule/function attributes
+    attrs.insert("idle".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_idle".to_string())));
+    attrs.insert("run".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_run".to_string())));
+    attrs.insert("idle_test".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_idle_test".to_string())));
+    attrs.insert("PyShell".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_pyshell".to_string())));
+    attrs.insert("config".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_config".to_string())));
+    attrs.insert("colorizer".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_colorizer".to_string())));
+    attrs.insert("autocomplete".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_autocomplete".to_string())));
+    attrs.insert("calltip".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_calltip".to_string())));
+    attrs.insert("debugger".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_debugger".to_string())));
+    attrs.insert("editor".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_editor".to_string())));
+    attrs.insert("filelist".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_filelist".to_string())));
+    attrs.insert("outwin".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_outwin".to_string())));
+    attrs.insert("rpc".to_string(), MbValue::from_ptr(MbObject::new_str("mb_idlelib_rpc".to_string())));
+
+    super::register_module("idlelib", attrs);
+}
+
+// ── Helper ──
+
+fn raise_not_implemented(name: &str) -> MbValue {
+    super::super::exception::mb_raise(
+        MbValue::from_ptr(MbObject::new_str("NotImplementedError".to_string())),
+        MbValue::from_ptr(MbObject::new_str(
+            format!("idlelib.{name} is not implemented in Mamba"),
+        )),
+    );
+    MbValue::none()
+}
+
+// ── Stub functions — each raises NotImplementedError ──
+
+pub fn mb_idlelib_idle() -> MbValue {
+    raise_not_implemented("idle")
+}
+
+pub fn mb_idlelib_run() -> MbValue {
+    raise_not_implemented("run")
+}
+
+pub fn mb_idlelib_idle_test() -> MbValue {
+    raise_not_implemented("idle_test")
+}
+
+pub fn mb_idlelib_pyshell() -> MbValue {
+    raise_not_implemented("PyShell")
+}
+
+pub fn mb_idlelib_config() -> MbValue {
+    raise_not_implemented("config")
+}
+
+pub fn mb_idlelib_colorizer() -> MbValue {
+    raise_not_implemented("colorizer")
+}
+
+pub fn mb_idlelib_autocomplete() -> MbValue {
+    raise_not_implemented("autocomplete")
+}
+
+pub fn mb_idlelib_calltip() -> MbValue {
+    raise_not_implemented("calltip")
+}
+
+pub fn mb_idlelib_debugger() -> MbValue {
+    raise_not_implemented("debugger")
+}
+
+pub fn mb_idlelib_editor() -> MbValue {
+    raise_not_implemented("editor")
+}
+
+pub fn mb_idlelib_filelist() -> MbValue {
+    raise_not_implemented("filelist")
+}
+
+pub fn mb_idlelib_outwin() -> MbValue {
+    raise_not_implemented("outwin")
+}
+
+pub fn mb_idlelib_rpc() -> MbValue {
+    raise_not_implemented("rpc")
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::runtime::exception;
+    use crate::runtime::rc::ObjData;
+
+    /// Helper: extract a string from an MbValue that should be a Str ptr.
+    fn extract_str(v: MbValue) -> Option<String> {
+        unsafe {
+            if let Some(ptr) = v.as_ptr() {
+                if let ObjData::Str(ref s) = (*ptr).data {
+                    return Some(s.clone());
+                }
+            }
+        }
+        None
+    }
+
+    #[test]
+    fn test_register() {
+        register();
+        let module_val = crate::runtime::module::mb_module_getattr(
+            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
+            MbValue::from_ptr(MbObject::new_str("__name__".to_string())),
+        );
+        assert_eq!(extract_str(module_val), Some("idlelib".to_string()));
+
+        let pkg = crate::runtime::module::mb_module_getattr(
+            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
+            MbValue::from_ptr(MbObject::new_str("__package__".to_string())),
+        );
+        assert_eq!(extract_str(pkg), Some("idlelib".to_string()));
+
+        let idle_sym = crate::runtime::module::mb_module_getattr(
+            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
+            MbValue::from_ptr(MbObject::new_str("idle".to_string())),
+        );
+        assert_eq!(extract_str(idle_sym), Some("mb_idlelib_idle".to_string()));
+
+        let pyshell_sym = crate::runtime::module::mb_module_getattr(
+            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
+            MbValue::from_ptr(MbObject::new_str("PyShell".to_string())),
+        );
+        assert_eq!(extract_str(pyshell_sym), Some("mb_idlelib_pyshell".to_string()));
+    }
+
+    #[test]
+    fn test_stub_raises() {
+        exception::mb_clear_exception();
+
+        let result = mb_idlelib_idle();
+        assert!(result.is_none(), "stub should return MbValue::none()");
+        assert_eq!(
+            exception::mb_has_exception().as_bool(),
+            Some(true),
+            "stub should set pending exception"
+        );
+
+        let exc = exception::mb_catch_exception();
+        assert!(!exc.is_none(), "should have caught an exception");
+        assert_eq!(exception::mb_has_exception().as_bool(), Some(false));
+
+        let result2 = mb_idlelib_pyshell();
+        assert!(result2.is_none());
+        assert_eq!(exception::mb_has_exception().as_bool(), Some(true));
+
+        let _ = exception::mb_catch_exception();
+    }
+}

diff --git a/crates/mamba/src/runtime/stdlib/mod.rs b/crates/mamba/src/runtime/stdlib/mod.rs
index fb59318e..a48e9625 100644
--- a/crates/mamba/src/runtime/stdlib/mod.rs
+++ b/crates/mamba/src/runtime/stdlib/mod.rs
@@ -35,6 +35,7 @@ pub mod inspect_mod;
 pub mod enum_mod;
 pub mod dataclasses_mod;
 // P3 stdlib modules
+pub mod idlelib_mod;
 pub mod subprocess_mod;
 pub mod csv_mod;
 pub mod argparse_mod;
@@ -128,6 +129,7 @@ pub fn register_stdlib() {
     enum_mod::register();
     dataclasses_mod::register();
     // P3 modules
+    idlelib_mod::register();
     subprocess_mod::register();
     csv_mod::register();
     argparse_mod::register();
```

## Review: idlelib-stub

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: 1000-patrol

**Summary**: Implementation fully satisfies all R1/R2/R3 spec requirements. The `idlelib_mod.rs` file correctly registers the module via `register_module()`, exposes all 13 required stub attributes with the proper symbol strings, sets package semantics (`__package__`, `__path__`), and raises Python-level `NotImplementedError` via `mb_raise()` (not panic!). The `mod.rs` changes correctly place `pub mod idlelib_mod;` under the `// P3 stdlib modules` block and call `idlelib_mod::register()` in `register_stdlib()`. A `#[cfg(test)]` module includes `test_register` and `test_stub_raises` tests. The spec's `## Test Plan` section is present but is a TODO placeholder — the implementation's tests exceed what the placeholder defines. All hard checklist items pass. Minor soft findings noted but none require changes.

### Issues

- **[soft]** test_register does not assert __path__ attribute (R3 gap)
- **[soft]** test_stub_raises does not verify exception message content (scenario at spec:68)
- **[soft]** Only 2 of 13 stub functions tested for exception-raise behavior
- **[soft]** ## Test Plan section is a TODO placeholder — spec quality gap, not an implementation defect
