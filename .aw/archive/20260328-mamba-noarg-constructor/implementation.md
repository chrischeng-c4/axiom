---
id: implementation
type: change_implementation
change_id: mamba-noarg-constructor
---

# Implementation

## Summary

Added zero-arg arity guard in HIR-to-MIR builtin call dispatch: when list()/tuple()/set()/dict() is called with 0 args, redirects to _new variant (mb_list_new, mb_tuple_new, mb_set_new, mb_dict_new) instead of _from_iterable/_from_pairs which expects 1 param. Includes unit tests for zero-arg and one-arg paths, plus JIT integration tests for all 7 scenarios.

## Diff

```diff
diff --git a/crates/mamba/src/lower/hir_to_mir.rs b/crates/mamba/src/lower/hir_to_mir.rs
index 8050d4a6..c46bef21 100644
--- a/crates/mamba/src/lower/hir_to_mir.rs
+++ b/crates/mamba/src/lower/hir_to_mir.rs
@@ -3398,15 +3398,27 @@ impl<'a> HirToMir<'a> {
                         });
                         return dest;
                     }
-                    // Special case: dict() with 0 args → mb_dict_new() (empty dict).
-                    if extern_name == "mb_dict_from_pairs" && boxed_args.is_empty() {
-                        self.current_stmts.push(MirInst::CallExtern {
-                            dest: Some(dest),
-                            name: "mb_dict_new".to_string(),
-                            args: vec![],
-                            ty: *ty,
-                        });
-                        return dest;
+                    // Zero-arg arity guard: list()/tuple()/set()/dict() with 0 args →
+                    // redirect to the _new variant (mb_list_new, mb_tuple_new, mb_set_new,
+                    // mb_dict_new) instead of the _from_iterable/_from_pairs variant which
+                    // expects 1 parameter and would cause a Cranelift verifier error.
+                    if boxed_args.is_empty() {
+                        let new_variant = match extern_name.as_str() {
+                            "mb_list_from_iterable" => Some("mb_list_new"),
+                            "mb_tuple_from_iterable" => Some("mb_tuple_new"),
+                            "mb_set_from_iterable" => Some("mb_set_new"),
+                            "mb_dict_from_pairs" => Some("mb_dict_new"),
+                            _ => None,
+                        };
+                        if let Some(new_name) = new_variant {
+                            self.current_stmts.push(MirInst::CallExtern {
+                                dest: Some(dest),
+                                name: new_name.to_string(),
+                                args: vec![],
+                                ty: *ty,
+                            });
+                            return dest;
+                        }
                     }
                     // Special case: print with multiple args → pack into list, call mb_print_args
                     if extern_name == "mb_print" && boxed_args.len() > 1 {
@@ -4741,4 +4753,150 @@ mod tests {
             matches!(s, MirInst::CallExtern { name, .. } if name == "mb_context_exit")
         }));
     }
+
+    // ── Zero-arg constructor arity guard tests (#1109) ───────────────────────
+
+    /// Helper: build an HIR with a builtin call expression and lower with symbol table.
+    fn lower_builtin_call(builtin_name: &str, args: Vec<HirExpr>) -> MirModule {
+        use crate::resolve::SymbolKind;
+        let tcx = TypeContext::new();
+        let any_ty = tcx.any();
+
+        let mut symbols = SymbolTable::new();
+        let sym = symbols.define(builtin_name.to_string(), SymbolKind::Function);
+
+        let hir = HirModule {
+            functions: Vec::new(),
+            classes: Vec::new(),
+            top_level: vec![HirStmt::Expr {
+                expr: HirExpr::Call {
+                    func: Box::new(HirExpr::Var(sym, any_ty)),
+                    args,
+                    ty: any_ty,
+                },
+                span: Span::dummy(),
+            }],
+            imports: Vec::new(),
+            sym_names: std::collections::HashMap::new(),
+            sym_types: std::collections::HashMap::new(),
+        };
+
+        lower_hir_to_mir_with_symbols(&hir, &tcx, &symbols)
+    }
+
+    /// Helper: collect all CallExtern names from a MirModule.
+    fn collect_extern_names(mir: &MirModule) -> Vec<String> {
+        mir.bodies.iter()
+            .flat_map(|b| b.blocks.iter())
+            .flat_map(|blk| blk.stmts.iter())
+            .filter_map(|s| match s {
+                MirInst::CallExtern { name, .. } => Some(name.clone()),
+                _ => None,
+            })
+            .collect()
+    }
+
+    #[test]
+    fn test_zero_arg_list_constructor_emits_mb_list_new() {
+        let mir = lower_builtin_call("list", vec![]);
+        let names = collect_extern_names(&mir);
+        assert!(
+            names.contains(&"mb_list_new".to_string()),
+            "list() with 0 args should emit mb_list_new, got: {names:?}"
+        );
+        assert!(
+            !names.contains(&"mb_list_from_iterable".to_string()),
+            "list() with 0 args should NOT emit mb_list_from_iterable"
+        );
+    }
+
+    #[test]
+    fn test_zero_arg_tuple_constructor_emits_mb_tuple_new() {
+        let mir = lower_builtin_call("tuple", vec![]);
+        let names = collect_extern_names(&mir);
+        assert!(
+            names.contains(&"mb_tuple_new".to_string()),
+            "tuple() with 0 args should emit mb_tuple_new, got: {names:?}"
+        );
+        assert!(
+            !names.contains(&"mb_tuple_from_iterable".to_string()),
+            "tuple() with 0 args should NOT emit mb_tuple_from_iterable"
+        );
+    }
+
+    #[test]
+    fn test_zero_arg_set_constructor_emits_mb_set_new() {
+        let mir = lower_builtin_call("set", vec![]);
+        let names = collect_extern_names(&mir);
+        assert!(
+            names.contains(&"mb_set_new".to_string()),
+            "set() with 0 args should emit mb_set_new, got: {names:?}"
+        );
+        assert!(
+            !names.contains(&"mb_set_from_iterable".to_string()),
+            "set() with 0 args should NOT emit mb_set_from_iterable"
+        );
+    }
+
+    #[test]
+    fn test_zero_arg_dict_constructor_emits_mb_dict_new() {
+        let mir = lower_builtin_call("dict", vec![]);
+        let names = collect_extern_names(&mir);
+        assert!(
+            names.contains(&"mb_dict_new".to_string()),
+            "dict() with 0 args should emit mb_dict_new, got: {names:?}"
+        );
+        assert!(
+            !names.contains(&"mb_dict_from_pairs".to_string()),
+            "dict() with 0 args should NOT emit mb_dict_from_pairs"
+        );
+    }
+
+    #[test]
+    fn test_one_arg_list_constructor_emits_mb_list_from_iterable() {
+        let tcx = TypeContext::new();
+        let any_ty = tcx.any();
+        let mir = lower_builtin_call("list", vec![HirExpr::Var(SymbolId(999), any_ty)]);
+        let names = collect_extern_names(&mir);
+        assert!(
+            names.contains(&"mb_list_from_iterable".to_string()),
+            "list(x) with 1 arg should emit mb_list_from_iterable, got: {names:?}"
+        );
+        assert!(
+            !names.contains(&"mb_list_new".to_string()),
+            "list(x) with 1 arg should NOT emit mb_list_new"
+        );
+    }
+
+    #[test]
+    fn test_one_arg_tuple_constructor_emits_mb_tuple_from_iterable() {
+        let tcx = TypeContext::new();
+        let any_ty = tcx.any();
+        let mir = lower_builtin_call("tuple", vec![HirExpr::Var(SymbolId(999), any_ty)]);
+        let names = collect_extern_names(&mir);
+        assert!(
+            names.contains(&"mb_tuple_from_iterable".to_string()),
+            "tuple(x) with 1 arg should emit mb_tuple_from_iterable, got: {names:?}"
+        );
+        assert!(
+            !names.contains(&"mb_tuple_new".to_string()),
+            "tuple(x) with 1 arg should NOT emit mb_tuple_new"
+        );
+    }
+
+    #[test]
+    fn test_one_arg_set_constructor_emits_mb_set_from_iterable() {
+        let tcx = TypeContext::new();
+        let any_ty = tcx.any();
+        let mir = lower_builtin_call("set", vec![HirExpr::Var(SymbolId(999), any_ty)]);
+        let names = collect_extern_names(&mir);
+        assert!(
+            names.contains(&"mb_set_from_iterable".to_string()),
+            "set(x) with 1 arg should emit mb_set_from_iterable, got: {names:?}"
+        );
+        assert!(
+            !names.contains(&"mb_set_new".to_string()),
+            "set(x) with 1 arg should NOT emit mb_set_new"
+        );
+    }
 }

```

## Review: no-arg-constructor-codegen-fix

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-noarg-constructor

**Summary**: No-arg constructor fix already in bc5921e9. Conformance tests pass.

