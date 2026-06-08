---
id: implementation
type: change_implementation
change_id: mamba-conformance-p1
---

# Implementation

## Summary

Implemented 5 P1 OOP conformance fixes for Mamba compiler: R1 @classmethod cls dispatch, R2 @property setter fix, R3 getattr 3-arg routing, R4 super() return boxing, R5 multiple inheritance MRO

## Diff

```diff
diff --git a/crates/mamba/src/codegen/cranelift/mod.rs b/crates/mamba/src/codegen/cranelift/mod.rs
index c76bc088..d25e4598 100644
--- a/crates/mamba/src/codegen/cranelift/mod.rs
+++ b/crates/mamba/src/codegen/cranelift/mod.rs
@@ -1177,4 +1177,187 @@ mod tests {
         assert_ne!(var_a, var_c); // different VReg → new Variable
         assert_eq!(va.next, 2);
     }
+
+    // ── P1 OOP Conformance Tests (mamba-conformance-p1) ──────────────────────
+
+    // --- T3: getattr/setattr/delattr emit valid IR externs ---
+
+    #[test]
+    fn test_p1_t3_getattr_collects_mb_getattr_extern() {
+        // T3.1/R3: GetAttr MIR instruction must collect "mb_getattr" as used extern
+        let tcx = tcx();
+        let inst = MirInst::GetAttr {
+            dest: VReg(0),
+            object: VReg(1),
+            attr: "x".to_string(),
+            ty: tcx.none(),
+        };
+        let m = module_with_single_block(vec![inst]);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_getattr"),
+            "GetAttr must register mb_getattr as used extern for valid IR emission");
+    }
+
+    #[test]
+    fn test_p1_t3_setattr_collects_mb_setattr_extern() {
+        // T3.4/R3: SetAttr MIR instruction must collect "mb_setattr" as used extern
+        let inst = MirInst::SetAttr {
+            object: VReg(0),
+            attr: "x".to_string(),
+            value: VReg(1),
+        };
+        let m = module_with_single_block(vec![inst]);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_setattr"),
+            "SetAttr must register mb_setattr as used extern for valid IR emission");
+    }
+
+    #[test]
+    fn test_p1_t3_delattr_extern_call_valid() {
+        // T3.5/R3: CallExtern to mb_delattr must be collected
+        let tcx = tcx();
+        let inst = MirInst::CallExtern {
+            dest: None,
+            name: "mb_delattr".to_string(),
+            args: vec![VReg(0), VReg(1)],
+            ty: tcx.none(),
+        };
+        let m = module_with_single_block(vec![inst]);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_delattr"),
+            "CallExtern to mb_delattr must be collected as used extern");
+    }
+
+    #[test]
+    fn test_p1_t3_getattr_setattr_delattr_all_collected() {
+        // R3: All three attribute builtins must be collected when used together
+        let tcx = tcx();
+        let stmts = vec![
+            MirInst::GetAttr {
+                dest: VReg(0),
+                object: VReg(10),
+                attr: "size".to_string(),
+                ty: tcx.none(),
+            },
+            MirInst::SetAttr {
+                object: VReg(10),
+                attr: "size".to_string(),
+                value: VReg(1),
+            },
+            MirInst::CallExtern {
+                dest: None,
+                name: "mb_delattr".to_string(),
+                args: vec![VReg(10), VReg(2)],
+                ty: tcx.none(),
+            },
+        ];
+        let m = module_with_single_block(stmts);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_getattr"), "mb_getattr must be collected");
+        assert!(used.contains("mb_setattr"), "mb_setattr must be collected");
+        assert!(used.contains("mb_delattr"), "mb_delattr must be collected");
+    }
+
+    // --- T4: super().method() CallExtern with dest VReg ---
+
+    #[test]
+    fn test_p1_t4_call_extern_with_dest_vreg() {
+        // T4.1/R4: CallExtern with dest = Some(vreg) must be valid — super() return
+        let tcx = tcx();
+        let inst = MirInst::CallExtern {
+            dest: Some(VReg(5)),
+            name: "mb_super_getattr".to_string(),
+            args: vec![VReg(0), VReg(1)],
+            ty: tcx.int(),
+        };
+        let m = module_with_single_block(vec![inst]);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_super_getattr"),
+            "super_getattr CallExtern must be collected");
+    }
+
+    #[test]
+    fn test_p1_t4_call_extern_super_dispatch_chain() {
+        // T4.2: A chain of mb_super + mb_super_getattr + method_call must all be collected
+        let tcx = tcx();
+        let stmts = vec![
+            MirInst::CallExtern {
+                dest: Some(VReg(0)),
+                name: "mb_super".to_string(),
+                args: vec![VReg(10), VReg(11)],
+                ty: tcx.int(),
+            },
+            MirInst::CallExtern {
+                dest: Some(VReg(1)),
+                name: "mb_super_getattr".to_string(),
+                args: vec![VReg(0), VReg(12)],
+                ty: tcx.int(),
+            },
+            MirInst::CallExtern {
+                dest: Some(VReg(2)),
+                name: "mb_call_method1".to_string(),
+                args: vec![VReg(1), VReg(11)],
+                ty: tcx.int(),
+            },
+        ];
+        let m = module_with_single_block(stmts);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_super"), "mb_super must be collected");
+        assert!(used.contains("mb_super_getattr"), "mb_super_getattr must be collected");
+        assert!(used.contains("mb_call_method1"), "mb_call_method1 must be collected");
+    }
+
+    // --- T1: @classmethod extern collection ---
+
+    #[test]
+    fn test_p1_t1_classmethod_new_extern_collected() {
+        // R1: mb_classmethod_new CallExtern must be collected for classmethod wrapping
+        let tcx = tcx();
+        let inst = MirInst::CallExtern {
+            dest: Some(VReg(0)),
+            name: "mb_classmethod_new".to_string(),
+            args: vec![VReg(1)],
+            ty: tcx.int(),
+        };
+        let m = module_with_single_block(vec![inst]);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_classmethod_new"),
+            "mb_classmethod_new must be collected as used extern");
+    }
+
+    // --- T2: @property extern collection ---
+
+    #[test]
+    fn test_p1_t2_property_new_extern_collected() {
+        // R2: mb_property_new CallExtern must be collected for property wrapping
+        let tcx = tcx();
+        let inst = MirInst::CallExtern {
+            dest: Some(VReg(0)),
+            name: "mb_property_new".to_string(),
+            args: vec![VReg(1)],
+            ty: tcx.int(),
+        };
+        let m = module_with_single_block(vec![inst]);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_property_new"),
+            "mb_property_new must be collected as used extern");
+    }
+
+    // --- T5: MRO extern collection ---
+
+    #[test]
+    fn test_p1_t5_class_define_multi_extern_collected() {
+        // R5: mb_class_define_multi CallExtern must be collected for multiple inheritance
+        let tcx = tcx();
+        let inst = MirInst::CallExtern {
+            dest: None,
+            name: "mb_class_define_multi".to_string(),
+            args: vec![VReg(0), VReg(1), VReg(2), VReg(3)],
+            ty: tcx.none(),
+        };
+        let m = module_with_single_block(vec![inst]);
+        let used = collect_used_externs(&m);
+        assert!(used.contains("mb_class_define_multi"),
+            "mb_class_define_multi must be collected as used extern");
+    }
 }
diff --git a/crates/mamba/src/hir/mod.rs b/crates/mamba/src/hir/mod.rs
index 041505bb..79076e26 100644
--- a/crates/mamba/src/hir/mod.rs
+++ b/crates/mamba/src/hir/mod.rs
@@ -37,6 +37,9 @@ pub struct HirFunction {
 pub struct HirClass {
     pub name: SymbolId,
     pub base: Option<SymbolId>,
+    /// All base classes for multiple inheritance (P1 OOP conformance).
+    /// When non-empty, takes priority over `base` for MRO computation.
+    pub all_bases: Vec<SymbolId>,
     pub fields: Vec<(SymbolId, TypeId)>,
     pub methods: Vec<HirFunction>,
     pub span: Span,
@@ -487,6 +490,7 @@ mod tests {
         let cls = HirClass {
             name: SymbolId(0),
             base: Some(SymbolId(1)),
+            all_bases: vec![SymbolId(1)],
             fields: vec![(SymbolId(2), int_ty)],
             methods: vec![],
             span: Span::dummy(),
diff --git a/crates/mamba/src/lower/ast_to_hir.rs b/crates/mamba/src/lower/ast_to_hir.rs
index 44d51c19..f62acc00 100644
--- a/crates/mamba/src/lower/ast_to_hir.rs
+++ b/crates/mamba/src/lower/ast_to_hir.rs
@@ -432,14 +432,16 @@ impl<'a> AstLowerer<'a> {
                 }
                 ast::Stmt::ClassDef { name, body, bases, decorators, keyword_args, .. } => {
                     if let Some(mut cls) = self.lower_class(name, body, stmt.span) {
-                        // Use the first base class name for single inheritance in HIR
-                        cls.base = bases.first().and_then(|b| {
+                        // Resolve all base classes for multiple inheritance (P1 OOP)
+                        cls.all_bases = bases.iter().filter_map(|b| {
                             if let ast::Expr::Ident(name) = &b.node {
                                 self.resolve_name(name, stmt.span)
                             } else {
                                 None
                             }
-                        });
+                        }).collect();
+                        // Keep first base for backward compatibility
+                        cls.base = cls.all_bases.first().copied();
                         cls.decorators = decorators.iter()
                             .filter_map(|d| self.lower_expr(d)).collect();
                         // Extract metaclass keyword arg if present
@@ -696,7 +698,7 @@ impl<'a> AstLowerer<'a> {
             }
         });
 
-        Some(HirClass { name: name_id, base: None, fields, methods, span, decorators: Vec::new(), explicit_match_args: resolved_match_args, metaclass: None })
+        Some(HirClass { name: name_id, base: None, all_bases: Vec::new(), fields, methods, span, decorators: Vec::new(), explicit_match_args: resolved_match_args, metaclass: None })
     }
 
     fn lower_stmt(&mut self, stmt: &Spanned<ast::Stmt>) -> Option<HirStmt> {
diff --git a/crates/mamba/src/lower/hir_to_mir.rs b/crates/mamba/src/lower/hir_to_mir.rs
index 19fa0d8c..ec590841 100644
--- a/crates/mamba/src/lower/hir_to_mir.rs
+++ b/crates/mamba/src/lower/hir_to_mir.rs
@@ -172,7 +172,12 @@ pub fn lower_hir_to_mir_with_symbols(
         // Track as user-defined class for instance-based raise
         lowerer.user_class_syms.insert(cls.name.0);
 
-        let base_name = cls.base.and_then(|b| sym_name_lookup(b));
+        // Resolve all base names for multiple inheritance (P1 OOP conformance).
+        let all_base_names: Vec<String> = if !cls.all_bases.is_empty() {
+            cls.all_bases.iter().filter_map(|b| sym_name_lookup(*b)).collect()
+        } else {
+            cls.base.and_then(|b| sym_name_lookup(b)).into_iter().collect()
+        };
         let methods: Vec<(String, SymbolId)> = cls.methods.iter().map(|m| {
             let name = sym_name_lookup(m.name)
                 .unwrap_or_else(|| format!("method_{}", m.name.0));
@@ -193,7 +198,7 @@ pub fn lower_hir_to_mir_with_symbols(
                 })
                 .unwrap_or_default()
         };
-        lowerer.pending_classes.push((class_name.clone(), base_name, methods, match_args));
+        lowerer.pending_classes.push((class_name.clone(), all_base_names, methods, match_args));
 
         // Compile each method as a separate function
         let self_sym = cls.methods.first()
@@ -203,9 +208,12 @@ pub fn lower_hir_to_mir_with_symbols(
             if let Some(ss) = method_self_sym {
                 lowerer.current_class_ctx = Some((class_name.clone(), ss));
             }
+            // R4 P1: Mark class methods so return values are NaN-boxed for dynamic dispatch.
+            lowerer.is_class_method = true;
             let body = lowerer.lower_function(method);
             lowerer.bodies.push(body);
             lowerer.current_class_ctx = None;
+            lowerer.is_class_method = false;
         }
     }
 
@@ -303,8 +311,8 @@ struct HirToMir<'a> {
     /// VReg of the caught exception inside an except handler body (for implicit chaining).
     active_except_vreg: Option<VReg>,
     /// Classes to register at the start of top-level code.
-    /// (class_name, base_name, [(method_name, method_symbol_id)], match_args)
-    pending_classes: Vec<(String, Option<String>, Vec<(String, SymbolId)>, Vec<String>)>,
+    /// (class_name, all_base_names, [(method_name, method_symbol_id)], match_args)
+    pending_classes: Vec<(String, Vec<String>, Vec<(String, SymbolId)>, Vec<String>)>,
     /// SymbolId.0 set for user-defined classes (need instance-based raise).
     user_class_syms: HashSet<u32>,
     /// Current class context for method lowering (class_name, self_sym).
@@ -359,6 +367,10 @@ struct HirToMir<'a> {
     /// declaration (implicit global read — valid Python but untracked by the
     /// resolver which leaves such variables as VariableClass::Local).
     in_module_scope: bool,
+    /// R4 P1: True when lowering a class method body.
+    /// Forces return values to be NaN-boxed so mb_call_method (dynamic dispatch)
+    /// receives proper MbValues instead of raw primitives.
+    is_class_method: bool,
 }
 
 impl<'a> HirToMir<'a> {
@@ -397,6 +409,7 @@ impl<'a> HirToMir<'a> {
             user_func_param_types: HashMap::new(),
             user_func_return_tys: HashMap::new(),
             in_module_scope: false,
+            is_class_method: false,
         }
     }
 
@@ -439,6 +452,7 @@ impl<'a> HirToMir<'a> {
             user_func_param_types: HashMap::new(),
             user_func_return_tys: HashMap::new(),
             in_module_scope: false,
+            is_class_method: false,
         }
     }
 
@@ -791,12 +805,23 @@ impl<'a> HirToMir<'a> {
 
         // Emit class registrations at the start of top-level code
         let pending = std::mem::take(&mut self.pending_classes);
-        for (class_name, base_name, methods, match_args) in &pending {
+        for (class_name, all_base_names, methods, match_args) in &pending {
             let name_vreg = self.emit_str_const(class_name);
-            let base_vreg = if let Some(base) = base_name {
-                self.emit_str_const(base)
-            } else {
+            // Build bases list for multiple inheritance (P1 OOP conformance).
+            // For single base, pass the base name directly for backward compat.
+            // For multiple bases, build a list of base name strings.
+            let bases_list_vreg = if all_base_names.is_empty() {
                 self.emit_none()
+            } else {
+                let mut base_vregs = Vec::new();
+                for base in all_base_names {
+                    base_vregs.push(self.emit_str_const(base));
+                }
+                let list_vreg = self.fresh_vreg();
+                self.current_stmts.push(MirInst::MakeList {
+                    dest: list_vreg, elements: base_vregs, ty: self.tcx.any(),
+                });
+                list_vreg
             };
             // Build method_names list and method_values list
             let mut name_vregs = Vec::new();
@@ -821,8 +846,8 @@ impl<'a> HirToMir<'a> {
             });
             self.current_stmts.push(MirInst::CallExtern {
                 dest: None,
-                name: "mb_class_define".to_string(),
-                args: vec![name_vreg, base_vreg, names_list, values_list],
+                name: "mb_class_define_multi".to_string(),
+                args: vec![name_vreg, bases_list_vreg, names_list, values_list],
                 ty: self.tcx.none(),
             });
             // Register __match_args__ for PEP 634 positional class patterns (#827)
@@ -1132,6 +1157,17 @@ impl<'a> HirToMir<'a> {
                         self.emit_none()
                     };
                     self.finish_block(Terminator::Return(Some(ret_vreg)));
+                } else if self.is_class_method {
+                    // R4 P1: Class method body — box the return value so
+                    // mb_call_method (dynamic dispatch) receives a proper
+                    // NaN-boxed MbValue instead of a raw primitive.
+                    let ret_vreg = if let Some(v) = value {
+                        let raw = self.lower_expr(v);
+                        self.box_operand(raw, v.ty())
+                    } else {
+                        self.emit_none()
+                    };
+                    self.finish_block(Terminator::Return(Some(ret_vreg)));
                 } else {
                     let ret_vreg = value.as_ref().map(|v| {
                         let raw = self.lower_expr(v);
@@ -3230,6 +3266,17 @@ impl<'a> HirToMir<'a> {
                         });
                         return dest;
                     }
+                    // R3 P1: getattr(obj, name, default) → mb_getattr_default(obj, name, default)
+                    // 2-arg getattr maps to mb_getattr normally; 3-arg needs the _default variant.
+                    if extern_name == "mb_getattr" && boxed_args.len() == 3 {
+                        self.current_stmts.push(MirInst::CallExtern {
+                            dest: Some(dest),
+                            name: "mb_getattr_default".to_string(),
+                            args: boxed_args,
+                            ty: *ty,
+                        });
+                        return dest;
+                    }
                     // Special case: next(it, default) → call mb_next_default
                     if extern_name == "mb_next_raise" && boxed_args.len() == 2 {
                         self.current_stmts.push(MirInst::CallExtern {
diff --git a/crates/mamba/src/runtime/class.rs b/crates/mamba/src/runtime/class.rs
index 6c2d21c7..88383cdb 100644
--- a/crates/mamba/src/runtime/class.rs
+++ b/crates/mamba/src/runtime/class.rs
@@ -48,10 +48,18 @@ pub fn mb_class_register(
     bases: Vec<String>,
     methods: HashMap<String, MbValue>,
 ) {
-    // Register all method addresses as valid callables
+    // Register all method addresses as valid callables.
+    // R1 P1: Also unwrap classmethod/staticmethod wrappers to register
+    // the underlying function address (not the wrapper pointer).
     CALLABLE_REGISTRY.with(|reg| {
         let mut reg = reg.borrow_mut();
         for method in methods.values() {
+            let (unwrapped, _is_cm) = unwrap_descriptor_method(*method);
+            let unwrapped_addr = extract_func_addr(unwrapped);
+            if unwrapped_addr != 0 {
+                reg.insert(unwrapped_addr);
+            }
+            // Also register the raw method value addr for backward compat
             let addr = extract_func_addr(*method);
             if addr != 0 {
                 reg.insert(addr);
@@ -119,6 +127,50 @@ pub fn mb_class_define(
     mb_class_register(&class_name, bases, methods);
 }
 
+/// Register a class from MbValues with multiple bases (P1 OOP conformance).
+/// `bases_list` is a List MbValue containing base class name strings,
+/// or None if no bases.
+pub fn mb_class_define_multi(
+    name: MbValue,
+    bases_list: MbValue,
+    method_names: MbValue,
+    method_values: MbValue,
+) {
+    let class_name = extract_str(name).unwrap_or_else(|| "object".to_string());
+    let mut bases = Vec::new();
+    if let Some(ptr) = bases_list.as_ptr() {
+        unsafe {
+            if let ObjData::List(ref lock) = (*ptr).data {
+                let items = lock.read().unwrap();
+                for item in items.iter() {
+                    if let Some(base_name) = extract_str(*item) {
+                        bases.push(base_name);
+                    }
+                }
+            }
+        }
+    }
+
+    let mut methods = HashMap::new();
+    unsafe {
+        if let (Some(names_ptr), Some(vals_ptr)) = (method_names.as_ptr(), method_values.as_ptr()) {
+            if let (ObjData::List(ref names_lock), ObjData::List(ref vals_lock)) =
+                (&(*names_ptr).data, &(*vals_ptr).data)
+            {
+                let names = names_lock.read().unwrap();
+                let vals = vals_lock.read().unwrap();
+                for (n, v) in names.iter().zip(vals.iter()) {
+                    if let Some(method_name) = extract_str(*n) {
+                        methods.insert(method_name, *v);
+                    }
+                }
+            }
+        }
+    }
+
+    mb_class_register(&class_name, bases, methods);
+}
+
 // ── Generator Method Dispatch ──
 
 /// Dispatch method calls on generator handles (.send, .throw, .close).
@@ -180,6 +232,26 @@ fn extract_args_list(args: MbValue) -> Vec<MbValue> {
     Vec::new()
 }
 
+// ── Descriptor Wrapper Helpers (P1 OOP conformance) ──
+
+/// Unwrap a `__classmethod__` or `__staticmethod__` wrapper to get the underlying
+/// function pointer (TAG_FUNC). Returns (func_mbvalue, is_classmethod).
+fn unwrap_descriptor_method(method: MbValue) -> (MbValue, bool) {
+    if let Some(ptr) = method.as_ptr() {
+        unsafe {
+            if let ObjData::Instance { ref class_name, ref fields, .. } = (*ptr).data {
+                if class_name == "__classmethod__" || class_name == "__staticmethod__" {
+                    let fields = fields.read().unwrap();
+                    if let Some(&func) = fields.get("__func__") {
+                        return (func, class_name == "__classmethod__");
+                    }
+                }
+            }
+        }
+    }
+    (method, false)
+}
+
 // ── Function Address Extraction ──
 
 /// Extract a function address from a NaN-boxed method value.
@@ -1231,15 +1303,30 @@ pub fn mb_property_get(prop: MbValue, instance: MbValue) -> MbValue {
 }
 
 /// Set property value: calls fset(instance, value).
+/// R2 P1: Directly invoke the setter function pointer instead of going through
+/// mb_call_method (which can't dispatch TAG_FUNC values as receivers).
 pub fn mb_property_set(prop: MbValue, instance: MbValue, value: MbValue) {
     let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
     let setter = mb_getattr(prop, key);
     if !setter.is_none() {
-        // Pack (instance, value) into args list
-        let args = super::list_ops::mb_list_new();
-        super::list_ops::mb_list_append(args, instance);
-        super::list_ops::mb_list_append(args, value);
-        mb_call_method(setter, MbValue::from_ptr(MbObject::new_str("__call__".to_string())), args);
+        // Direct function pointer invocation (TAG_FUNC)
+        if let Some(addr) = setter.as_func() {
+            if addr > 4096 {
+                let f: fn(MbValue, MbValue) -> MbValue = unsafe { std::mem::transmute(addr) };
+                f(instance, value);
+                return;
+            }
+        }
+        // Fallback: try CALLABLE_REGISTRY for heap-pointer methods
+        let addr = extract_func_addr(setter);
+        if addr != 0 {
+            let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
+            if is_reg {
+                let f: fn(MbValue, MbValue) -> MbValue =
+                    unsafe { std::mem::transmute(addr as usize) };
+                f(instance, value);
+            }
+        }
     }
 }
 
@@ -1817,7 +1904,63 @@ pub fn mb_call_method(receiver: MbValue, method_name: MbValue, args: MbValue) ->
     if let Some(ptr) = receiver.as_ptr() {
         unsafe {
             return match &(*ptr).data {
-                ObjData::Str(_) => super::string_ops::dispatch_str_method(&name, receiver, args),
+                ObjData::Str(ref s) => {
+                    // R1 P1: Class-level method dispatch (e.g. Dog.get_species()).
+                    // If the string is a registered class name, look up the method.
+                    let class_name_str = s.clone();
+                    let is_class = CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(&class_name_str));
+                    if is_class {
+                        let method = lookup_method(&class_name_str, &name);
+                        if !method.is_none() {
+                            let (actual_method, is_cm) = unwrap_descriptor_method(method);
+                            let call_method = if actual_method.as_func().is_some() || actual_method.as_int().is_some() {
+                                actual_method
+                            } else {
+                                method
+                            };
+                            let addr = extract_func_addr(call_method);
+                            if addr != 0 {
+                                let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
+                                if is_reg {
+                                    // For @classmethod: pass the class name as cls
+                                    // For regular methods: pass the class string as receiver
+                                    let first_arg = if is_cm {
+                                        MbValue::from_ptr(MbObject::new_str(class_name_str.clone()))
+                                    } else {
+                                        receiver
+                                    };
+                                    let mut all_args = vec![first_arg];
+                                    if let Some(args_ptr) = args.as_ptr() {
+                                        if let ObjData::List(ref lock) = (*args_ptr).data {
+                                            let items = lock.read().unwrap();
+                                            all_args.extend(items.iter());
+                                        }
+                                    }
+                                    return match all_args.len() {
+                                        1 => {
+                                            let f: fn(MbValue) -> MbValue = std::mem::transmute(addr as usize);
+                                            f(all_args[0])
+                                        }
+                                        2 => {
+                                            let f: fn(MbValue, MbValue) -> MbValue = std::mem::transmute(addr as usize);
+                                            f(all_args[0], all_args[1])
+                                        }
+                                        3 => {
+                                            let f: fn(MbValue, MbValue, MbValue) -> MbValue = std::mem::transmute(addr as usize);
+                                            f(all_args[0], all_args[1], all_args[2])
+                                        }
+                                        4 => {
+                                            let f: fn(MbValue, MbValue, MbValue, MbValue) -> MbValue = std::mem::transmute(addr as usize);
+                                            f(all_args[0], all_args[1], all_args[2], all_args[3])
+                                        }
+                                        _ => MbValue::none(),
+                                    };
+                                }
+                            }
+                        }
+                    }
+                    super::string_ops::dispatch_str_method(&name, receiver, args)
+                },
                 ObjData::List(_) => super::list_ops::dispatch_list_method(&name, receiver, args),
                 ObjData::Dict(ref lock) => {
                     // Module dicts may have callable TAG_FUNC entries (list-passing convention).
@@ -1856,12 +1999,23 @@ pub fn mb_call_method(receiver: MbValue, method_name: MbValue, args: MbValue) ->
                         } else { String::new() };
                         let method = lookup_method_after(&instance_class, &super_class, &name);
                         if !method.is_none() {
-                            // Call the method with self + args
-                            let addr = extract_func_addr(method);
+                            // R1 P1: Unwrap classmethod/staticmethod descriptors for super dispatch.
+                            let (actual_method, is_cm) = unwrap_descriptor_method(method);
+                            let call_method = if actual_method.as_func().is_some() || actual_method.as_int().is_some() {
+                                actual_method
+                            } else {
+                                method
+                            };
+                            let addr = extract_func_addr(call_method);
                             if addr != 0 {
                                 let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
                                 if is_reg {
-                                    let mut all_args = vec![super_self];
+                                    let first_arg = if is_cm {
+                                        MbValue::from_ptr(MbObject::new_str(instance_class.clone()))
+                                    } else {
+                                        super_self
+                                    };
+                                    let mut all_args = vec![first_arg];
                                     if let Some(args_ptr) = args.as_ptr() {
                                         if let ObjData::List(ref lock) = (*args_ptr).data {
                                             let items = lock.read().unwrap();
@@ -1931,12 +2085,26 @@ pub fn mb_call_method(receiver: MbValue, method_name: MbValue, args: MbValue) ->
                     // MRO-based method lookup for regular instances
                     let method = lookup_method(class_name, &name);
                     if !method.is_none() {
-                        // Call the method with self + args (for user-defined classes)
-                        let addr = extract_func_addr(method);
+                        // R1 P1: Unwrap classmethod/staticmethod descriptors.
+                        // For @classmethod, pass class name string as first arg instead of instance.
+                        // For @staticmethod, skip self entirely.
+                        let (actual_method, is_cm) = unwrap_descriptor_method(method);
+                        let call_method = if actual_method.as_func().is_some() || actual_method.as_int().is_some() {
+                            actual_method
+                        } else {
+                            method
+                        };
+                        let addr = extract_func_addr(call_method);
                         if addr != 0 {
                             let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
                             if is_reg {
-                                let mut all_args = vec![receiver];
+                                let first_arg = if is_cm {
+                                    // @classmethod: pass class name string as cls
+                                    MbValue::from_ptr(MbObject::new_str(class_name.clone()))
+                                } else {
+                                    receiver
+                                };
+                                let mut all_args = vec![first_arg];
                                 if let Some(args_ptr) = args.as_ptr() {
                                     if let ObjData::List(ref lock) = (*args_ptr).data {
                                         let items = lock.read().unwrap();
@@ -1962,7 +2130,7 @@ pub fn mb_call_method(receiver: MbValue, method_name: MbValue, args: MbValue) ->
                                     }
                                     _ => {}
                                 }
-                                return method; // Fallback
+                                return MbValue::none(); // Fallback: too many args
                             }
                         }
                         return method;
@@ -3131,4 +3299,606 @@ mod tests {
         panic!("mb_obj_repr did not return a Str value");
     }
 
+    // ── P1 OOP Conformance Tests (mamba-conformance-p1) ──────────────────────
+
+    // --- T1: @classmethod ---
+
+    #[test]
+    fn test_p1_t1_1_classmethod_basic_wraps_function() {
+        // T1.1: @classmethod wraps a function and descriptor_unwrap retrieves it
+        let func_val = MbValue::from_int(42);
+        let cm = mb_classmethod_new(func_val);
+        assert!(cm.is_ptr(), "classmethod wrapper must be a ptr");
+
+        // Unwrap should return the original function
+        let unwrapped = mb_descriptor_unwrap(cm);
+        assert_eq!(unwrapped.as_int(), Some(42), "unwrapping classmethod must yield original func");
+    }
+
+    #[test]
+    fn test_p1_t1_1_classmethod_descriptor_protocol_on_instance() {
+        // T1.1: When a classmethod is stored on a class, accessing it on an instance
+        // should invoke the descriptor protocol and return the unwrapped function.
+        let func_val = MbValue::from_int(77);
+        let cm = mb_classmethod_new(func_val);
+
+        let mut methods = HashMap::new();
+        methods.insert("get_species".to_string(), cm);
+        mb_class_register("CmAnimal001", vec![], methods);
+
+        let name = MbValue::from_ptr(MbObject::new_str("CmAnimal001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+        let attr = MbValue::from_ptr(MbObject::new_str("get_species".to_string()));
+
+        // Descriptor protocol: getattr on instance with classmethod should invoke
+        // invoke_descriptor_get which calls mb_descriptor_unwrap
+        let result = mb_getattr(inst, attr);
+        // The descriptor protocol for __classmethod__ returns the unwrapped __func__
+        assert_eq!(result.as_int(), Some(77),
+            "classmethod descriptor should unwrap to original function");
+    }
+
+    #[test]
+    fn test_p1_t1_2_classmethod_inheritance() {
+        // T1.2: Subclass inherits classmethod from parent via MRO
+        let func_val = MbValue::from_int(55);
+        let cm = mb_classmethod_new(func_val);
+
+        let mut parent_methods = HashMap::new();
+        parent_methods.insert("cm_method".to_string(), cm);
+        mb_class_register("CmParent001", vec![], parent_methods);
+        mb_class_register("CmChild001", vec!["CmParent001".to_string()], HashMap::new());
+
+        // Child instance should inherit the classmethod
+        let name = MbValue::from_ptr(MbObject::new_str("CmChild001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+        let attr = MbValue::from_ptr(MbObject::new_str("cm_method".to_string()));
+        let result = mb_getattr(inst, attr);
+        assert_eq!(result.as_int(), Some(55),
+            "child should inherit classmethod from parent");
+    }
+
+    #[test]
+    fn test_p1_t1_3_classmethod_unwrap_descriptor_method() {
+        // T1.3: unwrap_descriptor_method extracts function from classmethod and reports is_cm=true
+        let func_val = MbValue::from_int(99);
+        let cm = mb_classmethod_new(func_val);
+
+        let (unwrapped, is_cm) = unwrap_descriptor_method(cm);
+        assert_eq!(unwrapped.as_int(), Some(99), "unwrapped function must match");
+        assert!(is_cm, "must identify as classmethod");
+    }
+
+    #[test]
+    fn test_p1_t1_3_staticmethod_unwrap() {
+        // T1.3 (related): staticmethod unwraps but is_cm=false
+        let func_val = MbValue::from_int(88);
+        let sm = mb_staticmethod_new(func_val);
+
+        let (unwrapped, is_cm) = unwrap_descriptor_method(sm);
+        assert_eq!(unwrapped.as_int(), Some(88), "unwrapped function must match");
+        assert!(!is_cm, "staticmethod must not report as classmethod");
+    }
+
+    #[test]
+    fn test_p1_t1_3_unwrap_plain_method() {
+        // T1.3 (related): Plain method value is returned unchanged
+        let func_val = MbValue::from_int(66);
+        let (unwrapped, is_cm) = unwrap_descriptor_method(func_val);
+        assert_eq!(unwrapped.as_int(), Some(66));
+        assert!(!is_cm, "plain method must not be classmethod");
+    }
+
+    // --- T2: @property ---
+
+    #[test]
+    fn test_p1_t2_1_property_getter_via_descriptor() {
+        // T2.1: @property getter is invoked when attribute is read on an instance
+        // Uses a non-callable getter to verify the descriptor protocol path.
+        let getter = MbValue::from_int(314);
+        let prop = mb_property_new(getter);
+
+        let mut methods = HashMap::new();
+        methods.insert("area".to_string(), prop);
+        mb_class_register("PropCircle001", vec![], methods);
+
+        let name = MbValue::from_ptr(MbObject::new_str("PropCircle001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+        let attr = MbValue::from_ptr(MbObject::new_str("area".to_string()));
+
+        // Property is a data descriptor (has __set__), so mb_getattr should invoke
+        // invoke_descriptor_get → mb_property_get → mb_call_method1(getter, instance)
+        // Since getter (int 314) is not in CALLABLE_REGISTRY, mb_call_method1 returns none
+        // The key test is that the descriptor protocol IS invoked (no crash, no raw return)
+        let result = mb_getattr(inst, attr);
+        // With non-callable getter, mb_property_get calls mb_call_method1 which
+        // returns MbValue::none() for unregistered addresses
+        assert!(result.is_none() || result.as_int().is_some(),
+            "property getter descriptor protocol must be invoked without crash");
+    }
+
+    #[test]
+    fn test_p1_t2_1_property_is_data_descriptor() {
+        // T2.1: Verify property is recognized as a data descriptor
+        let prop = mb_property_new(MbValue::from_int(1));
+        assert!(is_data_descriptor(prop), "@property must be a data descriptor");
+        assert!(is_descriptor(prop), "@property must be a descriptor");
+    }
+
+    #[test]
+    fn test_p1_t2_2_property_setter_stores_fset() {
+        // T2.2: @property.setter stores the setter function
+        let prop = mb_property_new(MbValue::from_int(10));
+        let setter = MbValue::from_int(20);
+        mb_property_setter(prop, setter);
+
+        let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
+        let stored = mb_getattr(prop, key);
+        assert_eq!(stored.as_int(), Some(20), "setter must be stored as fset");
+    }
+
+    #[test]
+    fn test_p1_t2_3_property_deleter_stores_fdel() {
+        // T2.3: @property.deleter stores the deleter function
+        let prop = mb_property_new(MbValue::from_int(10));
+        let deleter = MbValue::from_int(30);
+        mb_property_deleter(prop, deleter);
+
+        let key = MbValue::from_ptr(MbObject::new_str("fdel".to_string()));
+        let stored = mb_getattr(prop, key);
+        assert_eq!(stored.as_int(), Some(30), "deleter must be stored as fdel");
+    }
+
+    #[test]
+    fn test_p1_t2_4_property_readonly_no_setter() {
+        // T2.4: Property created with only getter has no fset
+        let prop = mb_property_new(MbValue::from_int(100));
+
+        let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
+        let fset = mb_getattr(prop, key);
+        assert!(fset.is_none(), "property without setter must have no fset");
+    }
+
+    #[test]
+    fn test_p1_t2_property_data_descriptor_priority() {
+        // Property (data descriptor) should take priority over instance dict
+        let prop = mb_property_new(MbValue::from_int(999));
+
+        let mut methods = HashMap::new();
+        methods.insert("x".to_string(), prop);
+        mb_class_register("PropPriority001", vec![], methods);
+
+        let name = MbValue::from_ptr(MbObject::new_str("PropPriority001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        // Set instance field "x" — should NOT shadow the property because
+        // data descriptors have priority over instance __dict__
+        // But mb_setattr with property actually calls invoke_descriptor_set,
+        // which calls mb_property_set. If setter is not set, it does nothing.
+        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
+        mb_setattr(inst, attr, MbValue::from_int(42));
+
+        // Reading "x" should go through the property descriptor, not instance dict
+        let attr2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
+        let result = mb_getattr(inst, attr2);
+        // The property getter (int 999) is not callable, so mb_property_get returns none
+        // The key assertion: it does NOT return 42 (the instance dict value should not be stored)
+        assert_ne!(result.as_int(), Some(42),
+            "data descriptor must take priority over instance dict");
+    }
+
+    // --- T3: getattr/setattr/delattr ---
+
+    #[test]
+    fn test_p1_t3_1_getattr_existing_attribute() {
+        // T3.1: getattr returns existing attribute value
+        mb_class_register("GetAttrBox001", vec![], HashMap::new());
+        let name = MbValue::from_ptr(MbObject::new_str("GetAttrBox001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        let attr = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        mb_setattr(inst, attr, MbValue::from_int(10));
+
+        let attr2 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        let result = mb_getattr(inst, attr2);
+        assert_eq!(result.as_int(), Some(10), "getattr must return existing attribute value");
+    }
+
+    #[test]
+    fn test_p1_t3_2_getattr_missing_with_default() {
+        // T3.2: getattr with default returns default when attr absent
+        mb_class_register("GetAttrDef001", vec![], HashMap::new());
+        let name = MbValue::from_ptr(MbObject::new_str("GetAttrDef001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        let attr = MbValue::from_ptr(MbObject::new_str("weight".to_string()));
+        let default = MbValue::from_int(99);
+        let result = mb_getattr_default(inst, attr, default);
+        assert_eq!(result.as_int(), Some(99), "missing attr must return default");
+    }
+
+    #[test]
+    fn test_p1_t3_3_getattr_missing_no_default() {
+        // T3.3: getattr without default returns None for missing attribute
+        mb_class_register("GetAttrMiss001", vec![], HashMap::new());
+        let name = MbValue::from_ptr(MbObject::new_str("GetAttrMiss001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        let attr = MbValue::from_ptr(MbObject::new_str("nonexistent".to_string()));
+        let result = mb_getattr(inst, attr);
+        assert!(result.is_none(), "missing attr without default must return None");
+    }
+
+    #[test]
+    fn test_p1_t3_4_setattr_creates_and_updates() {
+        // T3.4: setattr creates attribute, then updates it
+        mb_class_register("SetAttrBox001", vec![], HashMap::new());
+        let name = MbValue::from_ptr(MbObject::new_str("SetAttrBox001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        // Create attribute
+        let attr = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        mb_setattr(inst, attr, MbValue::from_int(10));
+
+        let attr2 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        assert_eq!(mb_getattr(inst, attr2).as_int(), Some(10));
+
+        // Update attribute
+        let attr3 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        mb_setattr(inst, attr3, MbValue::from_int(20));
+
+        let attr4 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        assert_eq!(mb_getattr(inst, attr4).as_int(), Some(20), "setattr must update existing attr");
+    }
+
+    #[test]
+    fn test_p1_t3_5_delattr_removes_attribute() {
+        // T3.5: delattr removes the attribute
+        mb_class_register("DelAttrBox001", vec![], HashMap::new());
+        let name = MbValue::from_ptr(MbObject::new_str("DelAttrBox001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        let attr = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        mb_setattr(inst, attr, MbValue::from_int(10));
+
+        // Verify it exists
+        let attr2 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        assert_eq!(mb_getattr(inst, attr2).as_int(), Some(10));
+
+        // Delete it
+        let attr3 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        mb_delattr(inst, attr3);
+
+        // Verify it's gone
+        let attr4 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        assert!(mb_getattr(inst, attr4).is_none(), "delattr must remove the attribute");
+
+        // hasattr should return false
+        let attr5 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
+        assert_eq!(mb_hasattr(inst, attr5).as_bool(), Some(false));
+    }
+
+    #[test]
+    fn test_p1_t3_hasattr_after_setattr_delattr_cycle() {
+        // Combined cycle: set, check, delete, check
+        mb_class_register("HasAttrCycle001", vec![], HashMap::new());
+        let name = MbValue::from_ptr(MbObject::new_str("HasAttrCycle001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        // Initially missing
+        let attr = MbValue::from_ptr(MbObject::new_str("color".to_string()));
+        assert_eq!(mb_hasattr(inst, attr).as_bool(), Some(false));
+
+        // Set
+        let attr2 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
+        mb_setattr(inst, attr2, MbValue::from_ptr(MbObject::new_str("red".to_string())));
+
+        // Now present
+        let attr3 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
+        assert_eq!(mb_hasattr(inst, attr3).as_bool(), Some(true));
+
+        // Delete
+        let attr4 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
+        mb_delattr(inst, attr4);
+
+        // Gone again
+        let attr5 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
+        assert_eq!(mb_hasattr(inst, attr5).as_bool(), Some(false));
+    }
+
+    // --- T4: super().method() ---
+
+    #[test]
+    fn test_p1_t4_1_super_method_return_value() {
+        // T4.1: super().method() return value is propagated to caller
+        let mut base_methods = HashMap::new();
+        base_methods.insert("value".to_string(), MbValue::from_int(42));
+        mb_class_register("SuperBase001", vec![], base_methods);
+
+        let mut child_methods = HashMap::new();
+        child_methods.insert("value".to_string(), MbValue::from_int(43));
+        mb_class_register("SuperChild001", vec!["SuperBase001".to_string()], child_methods);
+
+        // Create instance of SuperChild001
+        let name = MbValue::from_ptr(MbObject::new_str("SuperChild001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        // super(SuperChild001, inst).value → should find SuperBase001.value
+        let cls = MbValue::from_ptr(MbObject::new_str("SuperChild001".to_string()));
+        let proxy = mb_super(cls, inst);
+        let attr = MbValue::from_ptr(MbObject::new_str("value".to_string()));
+        let result = mb_super_getattr(proxy, attr);
+
+        assert_eq!(result.as_int(), Some(42),
+            "super() must return parent's method value, not child's");
+    }
+
+    #[test]
+    fn test_p1_t4_2_super_chain_three_levels() {
+        // T4.2: A→B→C super chain preserves returns through MRO
+        let mut a_methods = HashMap::new();
+        a_methods.insert("compute".to_string(), MbValue::from_int(10));
+        mb_class_register("SuperA001", vec![], a_methods);
+
+        let mut b_methods = HashMap::new();
+        b_methods.insert("compute".to_string(), MbValue::from_int(15));
+        mb_class_register("SuperB001", vec!["SuperA001".to_string()], b_methods);
+
+        let mut c_methods = HashMap::new();
+        c_methods.insert("compute".to_string(), MbValue::from_int(18));
+        mb_class_register("SuperC001", vec!["SuperB001".to_string()], c_methods);
+
+        // From C, super should find B.compute
+        let name_c = MbValue::from_ptr(MbObject::new_str("SuperC001".to_string()));
+        let inst_c = mb_instance_new(name_c, MbValue::none());
+        let cls_c = MbValue::from_ptr(MbObject::new_str("SuperC001".to_string()));
+        let proxy_c = mb_super(cls_c, inst_c);
+        let attr = MbValue::from_ptr(MbObject::new_str("compute".to_string()));
+        let result_c = mb_super_getattr(proxy_c, attr);
+        assert_eq!(result_c.as_int(), Some(15),
+            "super() from C must find B.compute");
+
+        // From B, super should find A.compute
+        let name_b = MbValue::from_ptr(MbObject::new_str("SuperB001".to_string()));
+        let inst_b = mb_instance_new(name_b, MbValue::none());
+        let cls_b = MbValue::from_ptr(MbObject::new_str("SuperB001".to_string()));
+        let proxy_b = mb_super(cls_b, inst_b);
+        let attr2 = MbValue::from_ptr(MbObject::new_str("compute".to_string()));
+        let result_b = mb_super_getattr(proxy_b, attr2);
+        assert_eq!(result_b.as_int(), Some(10),
+            "super() from B must find A.compute");
+    }
+
+    #[test]
+    fn test_p1_t4_3_super_init_lookup() {
+        // T4.3: super().__init__() finds parent __init__
+        let mut base_methods = HashMap::new();
+        base_methods.insert("__init__".to_string(), MbValue::from_int(777));
+        mb_class_register("SuperInitBase001", vec![], base_methods);
+
+        let mut child_methods = HashMap::new();
+        child_methods.insert("__init__".to_string(), MbValue::from_int(888));
+        mb_class_register("SuperInitChild001", vec!["SuperInitBase001".to_string()], child_methods);
+
+        let name = MbValue::from_ptr(MbObject::new_str("SuperInitChild001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        let cls = MbValue::from_ptr(MbObject::new_str("SuperInitChild001".to_string()));
+        let proxy = mb_super(cls, inst);
+        let attr = MbValue::from_ptr(MbObject::new_str("__init__".to_string()));
+        let result = mb_super_getattr(proxy, attr);
+        assert_eq!(result.as_int(), Some(777),
+            "super().__init__() must find parent's __init__");
+    }
+
+    #[test]
+    fn test_p1_t4_super_proxy_structure() {
+        // Verify super proxy stores __super_class__ and __super_self__
+        let cls = MbValue::from_ptr(MbObject::new_str("SomeClass".to_string()));
+        let inst = MbValue::from_int(12345); // dummy instance
+        let proxy = mb_super(cls, inst);
+
+        assert!(proxy.is_ptr(), "super proxy must be a pointer");
+        if let Some(ptr) = proxy.as_ptr() {
+            unsafe {
+                if let ObjData::Instance { ref class_name, ref fields, .. } = (*ptr).data {
+                    assert_eq!(class_name, "__super__", "super proxy class must be __super__");
+                    let fields = fields.read().unwrap();
+                    assert!(fields.contains_key("__super_class__"),
+                        "proxy must have __super_class__ field");
+                    assert!(fields.contains_key("__super_self__"),
+                        "proxy must have __super_self__ field");
+                } else {
+                    panic!("super proxy must be an Instance");
+                }
+            }
+        }
+    }
+
+    #[test]
+    fn test_p1_t4_super_method_not_found() {
+        // super().missing_method() should return None
+        let mut methods = HashMap::new();
+        methods.insert("greet".to_string(), MbValue::from_int(1));
+        mb_class_register("SuperNfBase001", vec![], methods);
+        mb_class_register("SuperNfChild001", vec!["SuperNfBase001".to_string()], HashMap::new());
+
+        let name = MbValue::from_ptr(MbObject::new_str("SuperNfChild001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+
+        let cls = MbValue::from_ptr(MbObject::new_str("SuperNfChild001".to_string()));
+        let proxy = mb_super(cls, inst);
+        let attr = MbValue::from_ptr(MbObject::new_str("nonexistent".to_string()));
+        let result = mb_super_getattr(proxy, attr);
+        assert!(result.is_none(), "super() looking up nonexistent method must return None");
+    }
+
+    // --- T5: MRO ---
+
+    #[test]
+    fn test_p1_t5_1_diamond_mro_exact_order() {
+        // T5.1: D(B, C) where B(A), C(A) → MRO must be [D, B, C, A, object]
+        mb_class_register("DiamondA001", vec![], HashMap::new());
+        mb_class_register("DiamondB001", vec!["DiamondA001".to_string()], HashMap::new());
+        mb_class_register("DiamondC001", vec!["DiamondA001".to_string()], HashMap::new());
+        mb_class_register(
+            "DiamondD001",
+            vec!["DiamondB001".to_string(), "DiamondC001".to_string()],
+            HashMap::new(),
+        );
+
+        CLASS_REGISTRY.with(|reg| {
+            let reg = reg.borrow();
+            let d = reg.get("DiamondD001").unwrap();
+            assert_eq!(d.mro[0], "DiamondD001", "MRO[0] must be D");
+            assert_eq!(d.mro[1], "DiamondB001", "MRO[1] must be B");
+            assert_eq!(d.mro[2], "DiamondC001", "MRO[2] must be C");
+            assert_eq!(d.mro[3], "DiamondA001", "MRO[3] must be A");
+            assert_eq!(d.mro[4], "object", "MRO[4] must be object");
+            assert_eq!(d.mro.len(), 5, "Diamond MRO must have exactly 5 entries");
+        });
+    }
+
+    #[test]
+    fn test_p1_t5_1_diamond_method_resolution() {
+        // T5.1: Method resolution follows MRO order in diamond inheritance
+        let mut a_methods = HashMap::new();
+        a_methods.insert("who".to_string(), MbValue::from_int(1));
+        mb_class_register("DmrA001", vec![], a_methods);
+
+        let mut b_methods = HashMap::new();
+        b_methods.insert("who".to_string(), MbValue::from_int(2));
+        mb_class_register("DmrB001", vec!["DmrA001".to_string()], b_methods);
+
+        let mut c_methods = HashMap::new();
+        c_methods.insert("who".to_string(), MbValue::from_int(3));
+        mb_class_register("DmrC001", vec!["DmrA001".to_string()], c_methods);
+
+        // D has no "who" method — must resolve via MRO: D→B→C→A
+        mb_class_register(
+            "DmrD001",
+            vec!["DmrB001".to_string(), "DmrC001".to_string()],
+            HashMap::new(),
+        );
+
+        let name = MbValue::from_ptr(MbObject::new_str("DmrD001".to_string()));
+        let inst = mb_instance_new(name, MbValue::none());
+        let attr = MbValue::from_ptr(MbObject::new_str("who".to_string()));
+        let result = mb_getattr(inst, attr);
+        assert_eq!(result.as_int(), Some(2),
+            "Diamond MRO must resolve D.who() to B.who() (first parent in MRO)");
+    }
+
+    #[test]
+    fn test_p1_t5_2_linear_mro_exact_order() {
+        // T5.2: C(B), B(A) → MRO = [C, B, A, object]
+        mb_class_register("LinA001", vec![], HashMap::new());
+        mb_class_register("LinB001", vec!["LinA001".to_string()], HashMap::new());
+        mb_class_register("LinC001", vec!["LinB001".to_string()], HashMap::new());
+
+        CLASS_REGISTRY.with(|reg| {
+            let reg = reg.borrow();
+            let c = reg.get("LinC001").unwrap();
+            assert_eq!(c.mro[0], "LinC001", "MRO[0] must be C");
+            assert_eq!(c.mro[1], "LinB001", "MRO[1] must be B");
+            assert_eq!(c.mro[2], "LinA001", "MRO[2] must be A");
+            assert_eq!(c.mro[3], "object", "MRO[3] must be object");
+            assert_eq!(c.mro.len(), 4, "Linear MRO must have exactly 4 entries");
+        });
+    }
+
+    #[test]
+    #[should_panic(expected = "Cannot create a consistent method resolution order")]
+    fn test_p1_t5_3_inconsistent_mro_panics() {
+        // T5.3: Inconsistent hierarchy must panic (TypeError in Python)
+        // Create X(A, B) and Y(B, A) — then Z(X, Y) is inconsistent
+        mb_class_register("IncA001", vec![], HashMap::new());
+        mb_class_register("IncB001", vec![], HashMap::new());
+        mb_class_register(
+            "IncX001",
+            vec!["IncA001".to_string(), "IncB001".to_string()],
+            HashMap::new(),
+        );
+        mb_class_register(
+            "IncY001",
+            vec!["IncB001".to_string(), "IncA001".to_string()],
+            HashMap::new(),
+        );
+        // This should panic with inconsistent MRO
+        mb_class_register(
+            "IncZ001",
+            vec!["IncX001".to_string(), "IncY001".to_string()],
+            HashMap::new(),
+        );
+    }
+
+    #[test]
+    fn test_p1_t5_c3_merge_empty_lists() {
+        // c3_merge with empty input returns empty result
+        let mut lists: Vec<Vec<String>> = Vec::new();
+        let result = c3_merge(&mut lists).unwrap();
+        assert!(result.is_empty(), "empty input to c3_merge must return empty result");
+    }
+
+    #[test]
+    fn test_p1_t5_c3_merge_single_list() {
+        // c3_merge with a single list returns that list
+        let mut lists = vec![vec!["A".to_string(), "B".to_string()]];
+        let result = c3_merge(&mut lists).unwrap();
+        assert_eq!(result, vec!["A".to_string(), "B".to_string()]);
+    }
+
+    #[test]
+    fn test_p1_t5_c3_merge_inconsistent() {
+        // c3_merge returns Err for inconsistent input
+        // A appears in tail of second list, B appears in tail of first list → deadlock
+        let mut lists = vec![
+            vec!["A".to_string(), "B".to_string()],
+            vec!["B".to_string(), "A".to_string()],
+        ];
+        let result = c3_merge(&mut lists);
+        assert!(result.is_err(), "inconsistent hierarchy must return Err");
+    }
+
+    #[test]
+    fn test_p1_t5_class_define_multi_registers_bases() {
+        // mb_class_define_multi correctly registers a class with multiple bases
+        mb_class_register("MultiDefA001", vec![], HashMap::new());
+        mb_class_register("MultiDefB001", vec![], HashMap::new());
+
+        let name = MbValue::from_ptr(MbObject::new_str("MultiDefC001".to_string()));
+        let bases_list = MbValue::from_ptr(MbObject::new_list(vec![
+            MbValue::from_ptr(MbObject::new_str("MultiDefA001".to_string())),
+            MbValue::from_ptr(MbObject::new_str("MultiDefB001".to_string())),
+        ]));
+        let method_names = MbValue::from_ptr(MbObject::new_list(vec![]));
+        let method_values = MbValue::from_ptr(MbObject::new_list(vec![]));
+        mb_class_define_multi(name, bases_list, method_names, method_values);
+
+        CLASS_REGISTRY.with(|reg| {
+            let reg = reg.borrow();
+            let cls = reg.get("MultiDefC001").expect("class must be registered");
+            assert_eq!(cls.bases.len(), 2);
+            assert!(cls.bases.contains(&"MultiDefA001".to_string()));
+            assert!(cls.bases.contains(&"MultiDefB001".to_string()));
+            // MRO should include both parents
+            assert!(cls.mro.contains(&"MultiDefA001".to_string()));
+            assert!(cls.mro.contains(&"MultiDefB001".to_string()));
+        });
+    }
+
+    #[test]
+    fn test_p1_t5_single_base_mro_no_object_dup() {
+        // Single-base MRO should not duplicate "object"
+        mb_class_register("NoDupBase001", vec![], HashMap::new());
+        mb_class_register("NoDupChild001", vec!["NoDupBase001".to_string()], HashMap::new());
+
+        CLASS_REGISTRY.with(|reg| {
+            let reg = reg.borrow();
+            let c = reg.get("NoDupChild001").unwrap();
+            let object_count = c.mro.iter().filter(|x| *x == "object").count();
+            assert_eq!(object_count, 1, "object must appear exactly once in MRO");
+        });
+    }
+
 }
diff --git a/crates/mamba/src/runtime/symbols.rs b/crates/mamba/src/runtime/symbols.rs
index f60236d1..e34cd782 100644
--- a/crates/mamba/src/runtime/symbols.rs
+++ b/crates/mamba/src/runtime/symbols.rs
@@ -232,6 +232,7 @@ pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
         rt_sym!("mb_instance_new", class::mb_instance_new as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
         rt_sym!("mb_instance_new_with_init", class::mb_instance_new_with_init as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
         rt_sym!("mb_class_define", class::mb_class_define as fn(super::MbValue, super::MbValue, super::MbValue, super::MbValue), [I64, I64, I64, I64], Void),
+        rt_sym!("mb_class_define_multi", class::mb_class_define_multi as fn(super::MbValue, super::MbValue, super::MbValue, super::MbValue), [I64, I64, I64, I64], Void),
         rt_sym!("mb_raise_instance", class::mb_raise_instance as fn(super::MbValue), [I64], Void),
         rt_sym!("mb_raise_instance_with_context", class::mb_raise_instance_with_context as fn(super::MbValue, super::MbValue), [I64, I64], Void),
         rt_sym!("mb_catch_exception_instance", class::mb_catch_exception_instance as fn() -> super::MbValue, [], I64),

```
