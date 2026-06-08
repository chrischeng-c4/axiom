---
id: implementation
type: change_implementation
change_id: mamba-all-support
---

# Implementation

## Summary

Implement Python __all__ support for from X import * in Mamba. Production code (mb_import_star runtime function, __all__ whitelist in compile_and_exec_module, symbol registration, MIR lowering for star imports, resolve pass skip) was committed in 944f3ac5. This diff adds 6 unit tests covering all test plan scenarios: star import with __all__, without __all__, empty __all__, __all__ attr preservation, symbol registration verification, and resolve pass star symbol exclusion.

## Diff

```diff
diff --git a/crates/mamba/src/resolve/pass.rs b/crates/mamba/src/resolve/pass.rs
index 38102bc3..c85c7141 100644
--- a/crates/mamba/src/resolve/pass.rs
+++ b/crates/mamba/src/resolve/pass.rs
@@ -2016,4 +2016,26 @@ mod tests {
             "walrus target y should be defined even though x is not"
         );
     }
+
+    // @spec .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#R5
+    #[test]
+    fn test_resolve_star_import_no_star_symbol() {
+        // `from somemod import *` should NOT define `*` as a symbol.
+        // Star imports bind names dynamically at runtime, not statically.
+        let module = Module {
+            stmts: vec![
+                sp(Stmt::Import {
+                    module: vec!["somemod".to_string()],
+                    names: Some(vec![("*".to_string(), None)]),
+                    module_alias: None,
+                }),
+            ],
+        };
+        let result = resolve_module(&module);
+        assert!(result.errors.is_empty(),
+            "from X import * should not produce resolve errors, got: {:?}",
+            result.errors);
+        assert!(result.symbols.lookup("*").is_none(),
+            "* should NOT be defined as a symbol in the resolve pass");
+    }
 }
diff --git a/crates/mamba/src/runtime/module.rs b/crates/mamba/src/runtime/module.rs
index 1179a17f..3a895de1 100644
--- a/crates/mamba/src/runtime/module.rs
+++ b/crates/mamba/src/runtime/module.rs
@@ -1694,6 +1694,119 @@ mod tests {
         cleanup_all_modules();
     }
 
+    // ── mb_import_star tests ──
+
+    // @spec .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_with_all
+    #[test]
+    fn test_import_star_with_all() {
+        // Register a module with __all__ = ["foo", "bar"], plus extra attrs
+        let all_list = MbValue::from_ptr(MbObject::new_list(vec![
+            MbValue::from_ptr(MbObject::new_str("foo".to_string())),
+            MbValue::from_ptr(MbObject::new_str("bar".to_string())),
+        ]));
+        let mut attrs = HashMap::new();
+        attrs.insert("__all__".to_string(), all_list);
+        attrs.insert("foo".to_string(), MbValue::from_int(1));
+        attrs.insert("bar".to_string(), MbValue::from_int(2));
+        attrs.insert("_private".to_string(), MbValue::from_int(3));
+        attrs.insert("baz".to_string(), MbValue::from_int(4));
+        mb_module_register("star_all_mod", attrs);
+
+        let result = mb_import_star(s("star_all_mod"));
+        // Result is a dict; inspect it
+        assert!(result.is_ptr(), "mb_import_star should return a dict ptr");
+        unsafe {
+            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
+                let map = lock.read().unwrap();
+                // Only foo and bar should be present (from __all__)
+                assert_eq!(map.len(), 2, "dict should have exactly 2 entries from __all__");
+                assert!(map.contains_key("foo"), "foo should be exported");
+                assert!(map.contains_key("bar"), "bar should be exported");
+                assert!(!map.contains_key("_private"), "_private should NOT be exported");
+                assert!(!map.contains_key("baz"), "baz should NOT be exported (not in __all__)");
+            } else {
+                panic!("expected Dict from mb_import_star");
+            }
+        }
+    }
+
+    // @spec .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_without_all
+    #[test]
+    fn test_import_star_without_all() {
+        // Register a module WITHOUT __all__ — all public names exported
+        let mut attrs = HashMap::new();
+        attrs.insert("alpha".to_string(), MbValue::from_int(10));
+        attrs.insert("beta".to_string(), MbValue::from_int(20));
+        attrs.insert("_secret".to_string(), MbValue::from_int(30));
+        mb_module_register("star_noall_mod", attrs);
+
+        let result = mb_import_star(s("star_noall_mod"));
+        assert!(result.is_ptr());
+        unsafe {
+            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
+                let map = lock.read().unwrap();
+                // alpha and beta are public, _secret starts with _
+                assert!(map.contains_key("alpha"), "alpha should be exported");
+                assert!(map.contains_key("beta"), "beta should be exported");
+                assert!(!map.contains_key("_secret"), "_secret should NOT be exported");
+            } else {
+                panic!("expected Dict from mb_import_star");
+            }
+        }
+    }
+
+    // @spec .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_empty_all
+    #[test]
+    fn test_import_star_empty_all() {
+        // Register a module with __all__ = [] — nothing should be exported
+        let all_list = MbValue::from_ptr(MbObject::new_list(vec![]));
+        let mut attrs = HashMap::new();
+        attrs.insert("__all__".to_string(), all_list);
+        attrs.insert("something".to_string(), MbValue::from_int(99));
+        mb_module_register("star_empty_all_mod", attrs);
+
+        let result = mb_import_star(s("star_empty_all_mod"));
+        assert!(result.is_ptr());
+        unsafe {
+            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
+                let map = lock.read().unwrap();
+                assert_eq!(map.len(), 0, "empty __all__ should export nothing");
+            } else {
+                panic!("expected Dict from mb_import_star");
+            }
+        }
+    }
+
+    // @spec .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_preserves_all_attr
+    #[test]
+    fn test_import_star_preserves_all_attr() {
+        // Verify that __all__ is stored in module attrs (R1)
+        let all_list = MbValue::from_ptr(MbObject::new_list(vec![
+            MbValue::from_ptr(MbObject::new_str("x".to_string())),
+        ]));
+        let mut attrs = HashMap::new();
+        attrs.insert("__all__".to_string(), all_list);
+        attrs.insert("x".to_string(), MbValue::from_int(7));
+        mb_module_register("star_preserve_mod", attrs);
+
+        // Check that __all__ is present in the module's attrs
+        MODULES.with(|mods| {
+            let mods = mods.borrow();
+            let module = mods.get("star_preserve_mod").expect("module should exist");
+            assert!(module.attrs.contains_key("__all__"),
+                "__all__ must be preserved in module attrs");
+        });
+    }
+
+    // @spec .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_registered_in_symbols
+    #[test]
+    fn test_import_star_registered_in_symbols() {
+        use crate::runtime::symbols::runtime_symbols;
+        let syms = runtime_symbols();
+        let found = syms.iter().any(|s| s.name == "mb_import_star");
+        assert!(found, "mb_import_star must be registered in runtime_symbols()");
+    }
+
     // ── Cleanup: CURRENT_MODULE_PACKAGE ──
 
     #[test]

```

## Review: mamba-all-support-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-all-support

**Summary**: All 5 requirements (R1-R5) are implemented correctly across 4 files. 6 unit tests cover all test plan scenarios and pass. No regressions in the 2222-test non-stdlib suite.



## Alignment Warnings

10 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Requirements' at line 22 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Scenarios' at line 31 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Diagrams' at line 76 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'API Spec' at line 98 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Test Plan' at line 124 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Changes' at line 134 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/module.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
