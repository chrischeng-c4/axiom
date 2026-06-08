---
id: implementation
type: change_implementation
change_id: mamba-all-support
---

# Implementation

## Summary

Implement __all__ support for star-imports in cclab-mamba (#975).

Changes:
1. runtime/module.rs: Added mb_import_star() - loads module, checks __all__ in attrs, returns dict of exported names. Also preserved __all__ in compile_and_exec_module() by adding it to the dunder whitelist.
2. runtime/symbols.rs: Registered mb_import_star in RT_SYMBOLS.
3. lower/hir_to_mir.rs: Detect from X import * (names == [("*", None)]) and emit CallExtern to mb_import_star instead of per-name mb_module_getattr.
4. resolve/pass.rs: Skip defining * as a symbol when processing from-import-star.

6 tests pass: test_import_star_with_all, test_import_star_without_all, test_import_star_empty_all, test_import_star_preserves_all_attr, test_import_star_registered_in_symbols, test_resolve_star_import_no_star_symbol.

## Diff

```diff
commit ab6e138436c70021e46f5f348b8d58ad12e39a56
Author: chrischeng-c4 <chris.cheng.c4@gmail.com>
Date:   Thu Apr 9 08:58:42 2026

    chore(sdd): merge __all__ support (#975) — archive + main spec update
    
    __all__ filtering for `from X import *` implemented in module.rs.
    Reviewed, approved, archived. 2216 lib tests pass (pre-existing
    SIGABRT excluded). cargo check clean.

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

**Summary**: Implementation satisfies all 5 spec requirements. (1) R1: __all__ preserved in compile_and_exec_module() via || name == "__all__" whitelist addition. (2) R2: mb_import_star() fully implemented — loads module, checks __all__ in attrs, returns dict of exported names; falls back to all public names (not starting with _) when absent. (3) R3: mb_import_star registered in RT_SYMBOLS in symbols.rs. (4) R4: hir_to_mir.rs detects from X import * (is_star check) and emits CallExtern to mb_import_star. (5) R5: resolve/pass.rs skips symbol definition for * in star-imports. All 6 test cases from the test plan are present in the diff and pass. @spec annotations added for traceability. 59 runtime module tests pass.



## Alignment Warnings

11 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Requirements' at line 17 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Scenarios' at line 26 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Diagrams' at line 71 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'API Spec' at line 93 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Test Plan' at line 119 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Changes' at line 151 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | missing_section_annotation | Section 'Logic' at line 208 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/runtime/module.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
