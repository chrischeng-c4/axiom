---
id: implementation
type: change_implementation
change_id: mamba-p1-lang-features
---

# Implementation

## Summary

Implemented R1-R7: register_builtin_exceptions() for class-based MRO hierarchy, mb_get_exception() non-clearing retrieval, mb_exception_group_split/subgroup/exceptions, mb_name_error(), symbol registration

## Diff

```diff
diff --git a/crates/cclab-mamba/src/runtime/exception.rs b/crates/cclab-mamba/src/runtime/exception.rs
index 7b526463..6f8767f2 100644
--- a/crates/cclab-mamba/src/runtime/exception.rs
+++ b/crates/cclab-mamba/src/runtime/exception.rs
@@ -581,6 +581,245 @@ pub fn mb_except_star(group: MbValue, exc_type: MbValue) -> MbValue {
     MbValue::from_ptr(MbObject::new_tuple(vec![matched_val, rest_val]))
 }
 
+// ── R1: Built-in Exception Class Registration ──
+
+/// Register all built-in exception classes in the class registry with correct
+/// MRO inheritance.  Called once during runtime init so that `check_class_hierarchy`
+/// resolves exception subclass relationships without the hard-coded match table.
+///
+/// Hierarchy (Python 3.12):
+/// ```text
+/// BaseException
+/// ├── Exception
+/// │   ├── ArithmeticError
+/// │   │   ├── ZeroDivisionError
+/// │   │   ├── OverflowError
+/// │   │   └── FloatingPointError
+/// │   ├── LookupError
+/// │   │   ├── IndexError
+/// │   │   └── KeyError
+/// │   ├── ValueError
+/// │   │   └── UnicodeError
+/// │   │       ├── UnicodeDecodeError
+/// │   │       ├── UnicodeEncodeError
+/// │   │       └── UnicodeTranslateError
+/// │   ├── OSError
+/// │   │   ├── FileNotFoundError
+/// │   │   ├── PermissionError
+/// │   │   ├── IsADirectoryError
+/// │   │   ├── FileExistsError
+/// │   │   ├── TimeoutError
+/// │   │   └── ConnectionError
+/// │   │       ├── BrokenPipeError
+/// │   │       ├── ConnectionAbortedError
+/// │   │       ├── ConnectionRefusedError
+/// │   │       └── ConnectionResetError
+/// │   ├── TypeError
+/// │   ├── AttributeError
+/// │   ├── NameError
+/// │   │   └── UnboundLocalError
+/// │   ├── StopIteration
+/// │   ├── StopAsyncIteration
+/// │   ├── RuntimeError
+/// │   │   ├── NotImplementedError
+/// │   │   └── RecursionError
+/// │   ├── ImportError
+/// │   │   └── ModuleNotFoundError
+/// │   ├── SyntaxError
+/// │   │   ├── IndentationError
+/// │   │   │   └── TabError
+/// │   ├── Warning
+/// │   │   ├── DeprecationWarning
+/// │   │   ├── RuntimeWarning
+/// │   │   ├── UserWarning
+/// │   │   ├── SyntaxWarning
+/// │   │   ├── FutureWarning
+/// │   │   ├── PendingDeprecationWarning
+/// │   │   ├── UnicodeWarning
+/// │   │   ├── BytesWarning
+/// │   │   └── ResourceWarning
+/// │   └── ExceptionGroup
+/// ├── SystemExit
+/// ├── KeyboardInterrupt
+/// └── GeneratorExit
+/// ```
+pub fn register_builtin_exceptions() {
+    use std::collections::HashMap;
+    let empty = HashMap::new;
+
+    // Root
+    super::class::mb_class_register("BaseException", vec![], empty());
+
+    // Direct BaseException children (not subclass of Exception)
+    super::class::mb_class_register("SystemExit", vec!["BaseException".into()], empty());
+    super::class::mb_class_register("KeyboardInterrupt", vec!["BaseException".into()], empty());
+    super::class::mb_class_register("GeneratorExit", vec!["BaseException".into()], empty());
+
+    // Exception
+    super::class::mb_class_register("Exception", vec!["BaseException".into()], empty());
+
+    // Arithmetic hierarchy
+    super::class::mb_class_register("ArithmeticError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("ZeroDivisionError", vec!["ArithmeticError".into()], empty());
+    super::class::mb_class_register("OverflowError", vec!["ArithmeticError".into()], empty());
+    super::class::mb_class_register("FloatingPointError", vec!["ArithmeticError".into()], empty());
+
+    // Lookup hierarchy
+    super::class::mb_class_register("LookupError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("IndexError", vec!["LookupError".into()], empty());
+    super::class::mb_class_register("KeyError", vec!["LookupError".into()], empty());
+
+    // Value / Unicode hierarchy
+    super::class::mb_class_register("ValueError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("UnicodeError", vec!["ValueError".into()], empty());
+    super::class::mb_class_register("UnicodeDecodeError", vec!["UnicodeError".into()], empty());
+    super::class::mb_class_register("UnicodeEncodeError", vec!["UnicodeError".into()], empty());
+    super::class::mb_class_register("UnicodeTranslateError", vec!["UnicodeError".into()], empty());
+
+    // OS / IO hierarchy
+    super::class::mb_class_register("OSError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("FileNotFoundError", vec!["OSError".into()], empty());
+    super::class::mb_class_register("PermissionError", vec!["OSError".into()], empty());
+    super::class::mb_class_register("IsADirectoryError", vec!["OSError".into()], empty());
+    super::class::mb_class_register("FileExistsError", vec!["OSError".into()], empty());
+    super::class::mb_class_register("TimeoutError", vec!["OSError".into()], empty());
+    super::class::mb_class_register("ConnectionError", vec!["OSError".into()], empty());
+    super::class::mb_class_register("BrokenPipeError", vec!["ConnectionError".into()], empty());
+    super::class::mb_class_register("ConnectionAbortedError", vec!["ConnectionError".into()], empty());
+    super::class::mb_class_register("ConnectionRefusedError", vec!["ConnectionError".into()], empty());
+    super::class::mb_class_register("ConnectionResetError", vec!["ConnectionError".into()], empty());
+
+    // Simple Exception subclasses
+    super::class::mb_class_register("TypeError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("AttributeError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("NameError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("UnboundLocalError", vec!["NameError".into()], empty());
+    super::class::mb_class_register("StopIteration", vec!["Exception".into()], empty());
+    super::class::mb_class_register("StopAsyncIteration", vec!["Exception".into()], empty());
+
+    // Runtime hierarchy
+    super::class::mb_class_register("RuntimeError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("NotImplementedError", vec!["RuntimeError".into()], empty());
+    super::class::mb_class_register("RecursionError", vec!["RuntimeError".into()], empty());
+
+    // Import hierarchy
+    super::class::mb_class_register("ImportError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("ModuleNotFoundError", vec!["ImportError".into()], empty());
+
+    // Syntax hierarchy
+    super::class::mb_class_register("SyntaxError", vec!["Exception".into()], empty());
+    super::class::mb_class_register("IndentationError", vec!["SyntaxError".into()], empty());
+    super::class::mb_class_register("TabError", vec!["IndentationError".into()], empty());
+
+    // Warning hierarchy
+    super::class::mb_class_register("Warning", vec!["Exception".into()], empty());
+    super::class::mb_class_register("DeprecationWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("RuntimeWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("UserWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("SyntaxWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("FutureWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("PendingDeprecationWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("UnicodeWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("BytesWarning", vec!["Warning".into()], empty());
+    super::class::mb_class_register("ResourceWarning", vec!["Warning".into()], empty());
+
+    // ExceptionGroup (PEP 654) — inherits from Exception
+    super::class::mb_class_register("ExceptionGroup", vec!["Exception".into()], empty());
+}
+
+// ── R4: Non-destructive exception retrieval ──
+
+/// Retrieve the current exception without clearing the pending state.
+/// Returns `MbValue::none()` if no exception is pending.
+pub fn mb_get_exception() -> MbValue {
+    CURRENT_EXCEPTION.with(|cell| {
+        match cell.borrow().as_ref() {
+            Some(exc) => {
+                let val = store_exception_as_value(MbException {
+                    exc_type: exc.exc_type.clone(),
+                    message: exc.message.clone(),
+                    cause: exc.cause.as_ref().map(|c| Box::new(MbException {
+                        exc_type: c.exc_type.clone(),
+                        message: c.message.clone(),
+                        cause: None,
+                        context: None,
+                        suppress_context: c.suppress_context,
+                        traceback: c.traceback.clone(),
+                    })),
+                    context: exc.context.as_ref().map(|c| Box::new(MbException {
+                        exc_type: c.exc_type.clone(),
+                        message: c.message.clone(),
+                        cause: None,
+                        context: None,
+                        suppress_context: c.suppress_context,
+                        traceback: c.traceback.clone(),
+                    })),
+                    suppress_context: exc.suppress_context,
+                    traceback: exc.traceback.clone(),
+                });
+                unsafe { super::rc::retain_if_ptr(val); }
+                val
+            }
+            None => MbValue::none(),
+        }
+    })
+}
+
+// ── R5: ExceptionGroup additional methods ──
+
+/// Split an ExceptionGroup into (matched, rest) based on a type predicate.
+/// `predicate` is a string exception type name.  Returns a tuple (matched_group, rest_group)
+/// where each element is an ExceptionGroup or None.
+///
+/// This is a named alias for `mb_except_star` that follows CPython's
+/// `ExceptionGroup.split()` semantics.
+pub fn mb_exception_group_split(group: MbValue, predicate: MbValue) -> MbValue {
+    mb_except_star(group, predicate)
+}
+
+/// Filter an ExceptionGroup to only matching exceptions (subgroup).
+/// Returns a new ExceptionGroup containing only exceptions matching `predicate`,
+/// or None if no exceptions match.
+pub fn mb_exception_group_subgroup(group: MbValue, predicate: MbValue) -> MbValue {
+    let result = mb_except_star(group, predicate);
+    // mb_except_star returns (matched, rest) — extract matched (index 0)
+    if let Some(ptr) = result.as_ptr() {
+        unsafe {
+            if let ObjData::Tuple(ref items) = (*ptr).data {
+                if !items.is_empty() {
+                    return items[0];
+                }
+            }
+        }
+    }
+    MbValue::none()
+}
+
+/// Access the sub-exceptions of an ExceptionGroup as a tuple.
+/// Returns the `exceptions` field of the group, or None if not an ExceptionGroup.
+pub fn mb_exception_group_exceptions(group: MbValue) -> MbValue {
+    if let Some(ptr) = group.as_ptr() {
+        unsafe {
+            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
+                let fields = fields.read().unwrap();
+                if let Some(exc) = fields.get("exceptions") {
+                    return *exc;
+                }
+            }
+        }
+    }
+    MbValue::none()
+}
+
+// ── Built-in Exception Constructors (additional) ──
+
+pub fn mb_name_error(msg: &str) -> MbValue {
+    mb_exception_new(
+        MbValue::from_ptr(MbObject::new_str("NameError".to_string())),
+        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
+    )
+}
+
 // ── Cleanup ──
 
 /// Reset all exception-related thread_local state to defaults.
diff --git a/crates/cclab-mamba/src/runtime/symbols.rs b/crates/cclab-mamba/src/runtime/symbols.rs
index 3cad5347..9b54ef69 100644
--- a/crates/cclab-mamba/src/runtime/symbols.rs
+++ b/crates/cclab-mamba/src/runtime/symbols.rs
@@ -235,6 +235,7 @@ pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
         rt_sym!("mb_class_define_multi", class::mb_class_define_multi as fn(super::MbValue, super::MbValue, super::MbValue, super::MbValue), [I64, I64, I64, I64], Void),
         rt_sym!("mb_class_set_metaclass", class::mb_class_set_metaclass as fn(super::MbValue, super::MbValue), [I64, I64], Void),
         rt_sym!("mb_class_set_class_attr", class::mb_class_set_class_attr as fn(super::MbValue, super::MbValue, super::MbValue), [I64, I64, I64], Void),
+        rt_sym!("mb_class_set_kwargs", class::mb_class_set_kwargs as fn(super::MbValue, super::MbValue, super::MbValue), [I64, I64, I64], Void),
         rt_sym!("mb_raise_instance", class::mb_raise_instance as fn(super::MbValue), [I64], Void),
         rt_sym!("mb_raise_instance_with_context", class::mb_raise_instance_with_context as fn(super::MbValue, super::MbValue), [I64, I64], Void),
         rt_sym!("mb_catch_exception_instance", class::mb_catch_exception_instance as fn() -> super::MbValue, [], I64),
@@ -467,6 +468,13 @@ pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
         // ── ExceptionGroup / except* (#410) ──
         rt_sym!("mb_exception_group_new", exception::mb_exception_group_new as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
         rt_sym!("mb_except_star", exception::mb_except_star as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
+        rt_sym!("mb_exception_group_split", exception::mb_exception_group_split as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
+        rt_sym!("mb_exception_group_subgroup", exception::mb_exception_group_subgroup as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
+        rt_sym!("mb_exception_group_exceptions", exception::mb_exception_group_exceptions as fn(super::MbValue) -> super::MbValue, [I64], I64),
+        // ── Exception state retrieval (non-clearing) ──
+        rt_sym!("mb_get_exception", exception::mb_get_exception as fn() -> super::MbValue, [], I64),
+        // ── Exception class registration ──
+        rt_sym!("mb_register_builtin_exceptions", exception::register_builtin_exceptions as fn(), [], Void),
         // ── FrozenSet / Set constructors (#410) ──
         rt_sym!("mb_frozenset_new", builtins::mb_frozenset_new as fn(super::MbValue) -> super::MbValue, [I64], I64),
         rt_sym!("mb_set_from_iterable", builtins::mb_set_from_iterable as fn(super::MbValue) -> super::MbValue, [I64], I64),

```

## Review: class-features-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-p1-lang-features

**Summary**: Manually approved. class.rs +255 lines for slots/properties/classmethods.

## Review: exception-chaining-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-p1-lang-features

**Summary**: Manually approved.

## Review: parser-syntax-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-p1-lang-features

**Summary**: Manually approved.

## Review: runtime-ops-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-p1-lang-features

**Summary**: Manually approved.

## Review: stdlib-conformance-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-p1-lang-features

**Summary**: Manually approved.

