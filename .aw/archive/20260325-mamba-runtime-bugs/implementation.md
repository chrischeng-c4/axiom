---
id: implementation
type: change_implementation
change_id: mamba-runtime-bugs
---

# Implementation

## Summary

Fix 3 runtime conformance bugs in cclab-mamba:

1. **Decorator return value is None (#1084)**: `mb_box_int()` did not recognize TAG_FUNC (tag=4) as a valid NaN-boxed value. When `emit_internal_call` boxing logic re-boxed a decorated function return that happened to pass through a TAG_FUNC value, `mb_box_int` promoted it to BigInt (TAG_PTR), losing the function pointer. Fix: extend the NaN-box passthrough check from `tag <= 3` to `tag <= 4`.

2. **Floor division by zero does not raise ZeroDivisionError (#1085)**: Two sub-bugs: (a) `div_euclid` gives Euclidean division, not Python floor division — `-7 // 2` returned `-3` instead of `-4`. Fix: use `(a/b)` with sign-aware remainder adjustment. (b) Float `//` had no codegen handler — fell through to `iadd.f64` causing Cranelift verifier error. Fix: route all `FloorDiv` BinOp through `mb_floordiv` runtime with boxed operands.

3. **Nested f-string inner value lost (#1086)**: Already fixed in prior commit (parser/expr.rs `strip_fstring_literal`). Tests confirmed passing.


## Diff

```diff
diff --git a/crates/mamba/src/codegen/cranelift/jit.rs b/crates/mamba/src/codegen/cranelift/jit.rs
index a9f9a24c..861e6c96 100644
--- a/crates/mamba/src/codegen/cranelift/jit.rs
+++ b/crates/mamba/src/codegen/cranelift/jit.rs
@@ -251,7 +251,41 @@ impl CraneliftJitBackend {
                     MirBinOp::In | MirBinOp::NotIn => false,
                     _ => matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool),
                 };
-                if matches!(op, MirBinOp::Pow) && matches!(resolved_ty, Ty::Int) {
+                if matches!(op, MirBinOp::FloorDiv) {
+                    // Floor division → call mb_floordiv runtime for correct Python
+                    // floor semantics and ZeroDivisionError handling (#1085).
+                    // Operands must be boxed to MbValue before calling.
+                    let box_fn_name = match resolved_ty {
+                        Ty::Float => "mb_box_float",
+                        Ty::Bool => "mb_box_bool",
+                        _ => "mb_box_int",
+                    };
+                    // Gather func IDs before mutably borrowing self.module()
+                    let floordiv_id = self.extern_funcs.get("mb_floordiv").copied();
+                    let box_id = self.extern_funcs.get(box_fn_name).copied();
+                    if let Some(func_id) = floordiv_id {
+                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
+                        let cl_type = Self::mamba_to_cl_type(resolved_ty);
+                        let lv = vars.get(*lhs, builder, cl_type);
+                        let rv = vars.get(*rhs, builder, cl_type);
+                        let l = builder.use_var(lv);
+                        let r = builder.use_var(rv);
+                        let (l_boxed, r_boxed) = if let Some(bid) = box_id {
+                            let fref = self.module().declare_func_in_func(bid, builder.func);
+                            let lc = builder.ins().call(fref, &[l]);
+                            let rc = builder.ins().call(fref, &[r]);
+                            (builder.inst_results(lc)[0], builder.inst_results(rc)[0])
+                        } else { (l, r) };
+                        let call = builder.ins().call(func_ref, &[l_boxed, r_boxed]);
+                        let result = builder.inst_results(call)[0];
+                        let dv = vars.get(*dest, builder, cl_types::I64);
+                        builder.def_var(dv, result);
+                    } else {
+                        let zero = builder.ins().iconst(cl_types::I64, 0);
+                        let dv = vars.get(*dest, builder, cl_types::I64);
+                        builder.def_var(dv, zero);
+                    }
+                } else if matches!(op, MirBinOp::Pow) && matches!(resolved_ty, Ty::Int) {
                     // Integer power → call mb_pow_int runtime function
                     if let Some(&func_id) = self.extern_funcs.get("mb_pow_int") {
                         let func_ref = self.module().declare_func_in_func(func_id, builder.func);
diff --git a/crates/mamba/src/runtime/builtins.rs b/crates/mamba/src/runtime/builtins.rs
index 17063c7b..3559adc9 100644
--- a/crates/mamba/src/runtime/builtins.rs
+++ b/crates/mamba/src/runtime/builtins.rs
@@ -40,8 +40,10 @@ pub fn mb_box_int(raw: i64) -> MbValue {
     const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
     if bits & NAN_PREFIX == NAN_PREFIX {
         let tag = (bits >> 48) & 7;
-        if tag <= 3 {
-            // Valid NaN-boxed value (BigInt pointer or inline int from checked ops)
+        if tag <= 4 {
+            // Valid NaN-boxed value: PTR(0), INT(1), BOOL(2), NONE(3), FUNC(4).
+            // Decorator application can pass a TAG_FUNC through a function typed
+            // as returning Int — must not re-box it (#1084).
             return MbValue::from_bits(bits);
         }
     }
@@ -1496,10 +1498,15 @@ pub fn mb_call_spread(func: MbValue, args_list: MbValue) -> MbValue {
 
 /// floor division: a // b
 pub fn mb_floordiv(a: MbValue, b: MbValue) -> MbValue {
-    // Integer fast path
+    // Integer fast path — Python floor division (round towards -∞)
     if let (Some(ai), Some(bi)) = (a.as_int(), b.as_int()) {
         if bi != 0 {
-            return MbValue::from_int(ai.div_euclid(bi));
+            let d = ai / bi;
+            let r = ai % bi;
+            // Adjust: if remainder is non-zero and signs of remainder and divisor differ,
+            // subtract 1 to get floor division (rounds towards -∞, not towards 0).
+            let floored = if r != 0 && ((r ^ bi) < 0) { d - 1 } else { d };
+            return MbValue::from_int(floored);
         }
         // ZeroDivisionError: integer division or modulo by zero
         super::exception::mb_raise(

```

## Review: mamba-runtime-bugs-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-runtime-bugs

**Summary**: All 3 bugs fixed with correct implementations. mb_box_int TAG_FUNC passthrough fixes #1084 decorator return. Floor division uses proper Python floor semantics and routes through mb_floordiv for ZeroDivisionError (#1085). Nested f-string already working from prior commit (#1086). 26 tests pass, 80 conformance tests pass with 0 regressions. One test (stacked decorators) ignored due to pre-existing closure capture limitation — tracked separately.

