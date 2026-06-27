//! operator module — forward wrappers for Python's standard operators.
//!
//! Phase 2 Wave-2 ship #2 (Task #39, #1265). Each public surface entry is a
//! thin wrapper that routes to mamba's existing runtime binop/unop primitives
//! (`mb_add`, `mb_eq`, `mb_neg`, …). The shim contributes no new semantics —
//! correctness piggy-backs on the same fns that codegen of `a + b`, `a == b`
//! etc. already uses, so any future fix to those primitives automatically
//! flows through this module.
//!
//! Pattern mirrors `codecs_mod.rs` / `math_mod.rs`: flat-args ABI dispatchers
//! registered via the `NATIVE_FUNC_ADDRS` tuple-table. Dispatcher fns MUST
//! be named `dispatch_<verb>` so `surface.rs::pick_tuple_dispatcher`
//! recognises them — without the prefix Gate 3 surface score collapses to
//! 0/N (see `[[project_mamba_dispatch_prefix_convention]]`).
//!
//! Carve-out (callback-bound, NOT registered): `itemgetter`, `attrgetter`,
//! `methodcaller`. These three CPython classes return callables that close
//! over their constructor args and apply them at call time. Mamba's stdlib
//! shim ABI cannot today produce a callable that carries closure state
//! across the FFI boundary; tracked at #2100. Stubs below stand as
//! HANDWRITE markers so future codegen can fill them once the closure
//! primitive lands.
//!
//! HANDWRITE-BEGIN reason: stdlib-shim section type (register_module +
//! flat-args dispatch over runtime binops) is not yet emitted by score
//! codegen. Same shape as codecs_mod / math_mod — handwrite during
//! brute-force Phase 2, replace when aw standardize lands the
//! stdlib-shim section type.

use super::super::builtins;
use super::super::class;
use super::super::exception;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

// ── Small shared helpers (self-contained in this module) ──

/// Raise `exc_type(msg)` by name, matching the canonical runtime raise pattern.
fn raise(exc_type: &str, msg: &str) {
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc_type.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// Read a Rust `String` out of an `MbValue` that holds a `Str`. None otherwise.
fn as_str(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
}

/// True when `v` is a Python `str` object.
fn is_str(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
}

/// True when `v` is a Python `complex` object.
fn is_complex(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Complex(_, _)) })
}

/// True when an exception is currently pending.
fn has_exc() -> bool {
    exception::mb_has_exception().as_bool() == Some(true)
}

/// Raise the CPython arity TypeError for an operator function: "<name> expected
/// <n> argument(s), got <got>". CPython's `_operator` C funcs use METH_O /
/// METH_VARARGS arity checks that fail with this shape; the fixtures only catch
/// the type, so the exact wording is not asserted — but we keep it faithful.
fn arity_error(name: &str, want: usize, got: usize) -> MbValue {
    let plural = if want == 1 { "argument" } else { "arguments" };
    raise(
        "TypeError",
        &format!("{name} expected {want} {plural}, got {got}"),
    );
    MbValue::none()
}

/// Best-effort Python type name for error messages.
fn type_name(v: MbValue) -> String {
    if v.is_bool() {
        return "bool".to_string();
    }
    if v.is_int() {
        return "int".to_string();
    }
    if v.is_float() {
        return "float".to_string();
    }
    if v.is_none() {
        return "NoneType".to_string();
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str".to_string(),
                ObjData::List(_) => "list".to_string(),
                ObjData::Tuple(_) => "tuple".to_string(),
                ObjData::Dict(_) => "dict".to_string(),
                ObjData::Set(_) => "set".to_string(),
                ObjData::Bytes(_) => "bytes".to_string(),
                ObjData::ByteArray(_) => "bytearray".to_string(),
                ObjData::Instance { class_name, .. } => class_name.clone(),
                _ => "object".to_string(),
            };
        }
    }
    "object".to_string()
}

/// True when `v` is a user-defined / stdlib class instance (heap Instance).
fn instance_class_name(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, .. } => Some(class_name.clone()),
            _ => None,
        }
    }
}

/// Build a fresh Instance of `class_name` and run `init` over its field map.
fn make_instance<F: FnOnce(&mut super::super::rc::InstanceFields)>(
    class_name: &str,
    init: F,
) -> MbValue {
    let inst = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            init(&mut g);
        }
    }
    MbValue::from_ptr(inst)
}

/// Read a copied field value from an Instance.
fn field(inst: MbValue, name: &str) -> MbValue {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let g = fields.read().unwrap();
                if let Some(v) = g.get(name).copied() {
                    super::super::rc::retain_if_ptr(v);
                    return v;
                }
            }
        }
    }
    MbValue::none()
}

/// Collect the elements of a List/Tuple `MbValue` into a Vec.
fn seq_items(v: MbValue) -> Option<Vec<MbValue>> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            ObjData::Tuple(items) => Some(items.clone()),
            _ => None,
        }
    }
}

/// `obj[key]` honouring list / tuple / str / dict and slice keys, raising the
/// CPython-correct IndexError / KeyError / TypeError. Returns None (with a
/// pending exception) on error.
fn subscript(obj: MbValue, key: MbValue) -> MbValue {
    let ptr = match obj.as_ptr() {
        Some(p) => p,
        None => {
            // range objects are bare iterator handles (ints), not heap ptrs.
            // Support integer indexing into them (CPython: range[i]).
            if super::super::iter::is_range_handle(obj) {
                // Reject non-integer keys (None, str, float) with TypeError.
                if key.is_float() || is_str(key) || key.is_none() {
                    raise("TypeError", "range indices must be integers or slices");
                    return MbValue::none();
                }
                match super::super::iter::range_iter_getitem_value(obj, key) {
                    Some(v) => return v,
                    None => {
                        raise("IndexError", "range object index out of range");
                        return MbValue::none();
                    }
                }
            }
            raise("TypeError", "'NoneType' object is not subscriptable");
            return MbValue::none();
        }
    };
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => {
                let guard = lock.read().unwrap();
                seq_index(&guard, key, "list")
            }
            ObjData::Tuple(items) => seq_index(items, key, "tuple"),
            ObjData::Str(s) => {
                // Index a str by integer code-point position; reject non-int keys.
                let chars: Vec<char> = s.chars().collect();
                if key.is_float() || is_str(key) {
                    raise("TypeError", "string indices must be integers");
                    return MbValue::none();
                }
                let n = chars.len() as i64;
                match key.as_int() {
                    Some(i) => {
                        let r = if i < 0 { i + n } else { i };
                        if r >= 0 && r < n {
                            MbValue::from_ptr(MbObject::new_str(chars[r as usize].to_string()))
                        } else {
                            raise("IndexError", "string index out of range");
                            MbValue::none()
                        }
                    }
                    None => {
                        raise("TypeError", "string indices must be integers");
                        MbValue::none()
                    }
                }
            }
            ObjData::Dict(_) => super::super::dict_ops::mb_dict_getitem(obj, key),
            ObjData::Instance { class_name, .. } if class_name == "slice" => {
                raise("TypeError", "unhashable type: 'slice'");
                MbValue::none()
            }
            ObjData::Instance { class_name, .. } => {
                // Instances with a __getitem__ dunder subscript through it.
                let cls = class_name.clone();
                if !class::lookup_method(&cls, "__getitem__").is_none() {
                    let args = MbValue::from_ptr(MbObject::new_list(vec![key]));
                    return class::mb_call_method(
                        obj,
                        MbValue::from_ptr(MbObject::new_str("__getitem__".to_string())),
                        args,
                    );
                }
                raise("TypeError", &format!("'{cls}' object is not subscriptable"));
                MbValue::none()
            }
            _ => {
                // range / other iterators: best-effort generic subscript (None on miss).
                let r = super::super::list_ops::mb_seq_getitem(obj, key.as_int().unwrap_or(0));
                if r.is_none() {
                    raise(
                        "TypeError",
                        &format!("'{}' object is not subscriptable", type_name(obj)),
                    );
                }
                r
            }
        }
    }
}

/// Index a slice of items (list/tuple backing) honouring negatives and slice
/// keys, raising IndexError / TypeError to match CPython sequence indexing.
fn seq_index(items: &[MbValue], key: MbValue, kind: &str) -> MbValue {
    // slice key → return a tuple/list slice
    if instance_class_name(key).as_deref() == Some("slice") {
        let start = field(key, "start");
        let stop = field(key, "stop");
        let step = field(key, "step");
        let n = items.len() as i64;
        let st = step.as_int().unwrap_or(1);
        if st == 0 {
            raise("ValueError", "slice step cannot be zero");
            return MbValue::none();
        }
        let (mut lo, mut hi) = if st > 0 { (0i64, n) } else { (n - 1, -1i64) };
        if let Some(s) = start.as_int() {
            lo = if s < 0 {
                (s + n).max(if st > 0 { 0 } else { -1 })
            } else {
                s.min(if st > 0 { n } else { n - 1 })
            };
        }
        if let Some(s) = stop.as_int() {
            hi = if s < 0 {
                (s + n).max(if st > 0 { 0 } else { -1 })
            } else {
                s.min(if st > 0 { n } else { n - 1 })
            };
        }
        let mut out = Vec::new();
        let mut i = lo;
        while (st > 0 && i < hi) || (st < 0 && i > hi) {
            if i >= 0 && i < n {
                let v = items[i as usize];
                unsafe { super::super::rc::retain_if_ptr(v) };
                out.push(v);
            }
            i += st;
        }
        return if kind == "tuple" {
            MbValue::from_ptr(MbObject::new_tuple(out))
        } else {
            MbValue::from_ptr(MbObject::new_list(out))
        };
    }
    if is_str(key) {
        raise(
            "TypeError",
            &format!("{kind} indices must be integers or slices, not str"),
        );
        return MbValue::none();
    }
    match key.as_int() {
        Some(i) => {
            let n = items.len() as i64;
            let r = if i < 0 { i + n } else { i };
            if r >= 0 && r < n {
                let v = items[r as usize];
                unsafe { super::super::rc::retain_if_ptr(v) };
                v
            } else {
                raise("IndexError", &format!("{kind} index out of range"));
                MbValue::none()
            }
        }
        None => {
            raise(
                "TypeError",
                &format!("{kind} indices must be integers or slices"),
            );
            MbValue::none()
        }
    }
}

// ── Variadic dispatchers (callable from module-attr context) ──
// NOTE: dispatcher fn names must start with `dispatch_` so the surface walker
// (projects/mamba/src/surface.rs::pick_tuple_dispatcher) recognises them.

macro_rules! disp_unary {
    ($disp:ident, $name:literal, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            // CPython's unary operator funcs are METH_O: exactly one argument.
            if nargs != 1 {
                return arity_error($name, 1, nargs);
            }
            $fn(unsafe { *args_ptr })
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $name:literal, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            // CPython's binary operator funcs take exactly two arguments.
            if nargs != 2 {
                return arity_error($name, 2, nargs);
            }
            unsafe { $fn(*args_ptr, *args_ptr.add(1)) }
        }
    };
}

// ── Arithmetic (binary) ──
disp_binary!(dispatch_add, "add", mb_operator_add);
disp_binary!(dispatch_sub, "sub", mb_operator_sub);
disp_binary!(dispatch_mul, "mul", mb_operator_mul);
disp_binary!(dispatch_truediv, "truediv", mb_operator_truediv);
disp_binary!(dispatch_floordiv, "floordiv", mb_operator_floordiv);
disp_binary!(dispatch_mod, "mod", mb_operator_mod);
disp_binary!(dispatch_pow, "pow", mb_operator_pow);
disp_binary!(dispatch_matmul, "matmul", mb_operator_matmul);

// ── Bitwise (binary) ──
disp_binary!(dispatch_and_, "and_", mb_operator_and);
disp_binary!(dispatch_or_, "or_", mb_operator_or);
disp_binary!(dispatch_xor, "xor", mb_operator_xor);
disp_binary!(dispatch_lshift, "lshift", mb_operator_lshift);
disp_binary!(dispatch_rshift, "rshift", mb_operator_rshift);

// ── Unary ──
disp_unary!(dispatch_neg, "neg", mb_operator_neg);
disp_unary!(dispatch_pos, "pos", mb_operator_pos);
disp_unary!(dispatch_not_, "not_", mb_operator_not);
disp_unary!(dispatch_abs, "abs", mb_operator_abs);
disp_unary!(dispatch_invert, "invert", mb_operator_invert);
disp_unary!(dispatch_truth, "truth", mb_operator_truth);
disp_unary!(dispatch_index, "index", mb_operator_index);

// length_hint(obj, default=0) — two-arg form needs its own dispatcher.
unsafe extern "C" fn dispatch_length_hint(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 || nargs > 2 {
        raise("TypeError", "length_hint expected at most 2 arguments");
        return MbValue::none();
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a[0];
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    operator_length_hint(obj, default)
}

// ── Comparison (binary) ──
disp_binary!(dispatch_eq, "eq", mb_operator_eq);
disp_binary!(dispatch_ne, "ne", mb_operator_ne);
disp_binary!(dispatch_lt, "lt", mb_operator_lt);
disp_binary!(dispatch_le, "le", mb_operator_le);
disp_binary!(dispatch_gt, "gt", mb_operator_gt);
disp_binary!(dispatch_ge, "ge", mb_operator_ge);

// ── Identity / membership ──
disp_binary!(dispatch_is_, "is_", mb_operator_is);
disp_binary!(dispatch_is_not, "is_not", mb_operator_is_not);
disp_binary!(dispatch_contains, "contains", mb_operator_contains);
disp_binary!(dispatch_countOf, "countOf", mb_operator_count_of);
disp_binary!(dispatch_indexOf, "indexOf", mb_operator_index_of);

// ── Sequence: getitem / setitem / delitem / concat ──
disp_binary!(dispatch_getitem, "getitem", mb_operator_getitem);
// setitem(obj, key, value) — three positional args.
unsafe extern "C" fn dispatch_setitem(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs != 3 {
        return arity_error("setitem", 3, nargs);
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    operator_setitem(a[0], a[1], a[2])
}
// delitem(obj, key) — two positional args.
unsafe extern "C" fn dispatch_delitem(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs != 2 {
        return arity_error("delitem", 2, nargs);
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    operator_delitem(a[0], a[1])
}
disp_binary!(dispatch_concat, "concat", mb_operator_concat);

// ── In-place ops ──
//
// CPython's pure-Python operator.iADD(a, b) is `a += b`, which prefers the
// `__iADD__` dunder and falls back to the binary form. For built-in numbers
// there is no in-place dunder so the binary op result is returned.
macro_rules! disp_inplace {
    ($disp:ident, $name:literal, $dunder:literal, $binop:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            if nargs != 2 {
                return arity_error($name, 2, nargs);
            }
            let a = unsafe { *args_ptr };
            let b = unsafe { *args_ptr.add(1) };
            inplace_dispatch($dunder, a, b, $binop)
        }
    };
}

/// Apply an in-place operator: try the `__iOP__` dunder on `a`, else the binary
/// fallback (which itself dispatches `__OP__`/`__rOP__`).
fn inplace_dispatch(
    dunder: &str,
    a: MbValue,
    b: MbValue,
    binop: fn(MbValue, MbValue) -> MbValue,
) -> MbValue {
    if let Some(cls) = instance_class_name(a) {
        if !class::lookup_method(&cls, dunder).is_none() {
            let args = MbValue::from_ptr(MbObject::new_list(vec![b]));
            return class::mb_call_method(
                a,
                MbValue::from_ptr(MbObject::new_str(dunder.to_string())),
                args,
            );
        }
    }
    binop(a, b)
}

disp_inplace!(dispatch_iadd, "iadd", "__iadd__", mb_operator_add);
disp_inplace!(dispatch_isub, "isub", "__isub__", mb_operator_sub);
disp_inplace!(dispatch_imul, "imul", "__imul__", mb_operator_mul);
disp_inplace!(
    dispatch_itruediv,
    "itruediv",
    "__itruediv__",
    mb_operator_truediv
);
disp_inplace!(
    dispatch_ifloordiv,
    "ifloordiv",
    "__ifloordiv__",
    mb_operator_floordiv
);
disp_inplace!(dispatch_imod, "imod", "__imod__", mb_operator_mod);
disp_inplace!(dispatch_ipow, "ipow", "__ipow__", mb_operator_pow);
disp_inplace!(
    dispatch_imatmul,
    "imatmul",
    "__imatmul__",
    mb_operator_matmul
);
disp_inplace!(dispatch_iand, "iand", "__iand__", mb_operator_and);
disp_inplace!(dispatch_ior, "ior", "__ior__", mb_operator_or);
disp_inplace!(dispatch_ixor, "ixor", "__ixor__", mb_operator_xor);
disp_inplace!(
    dispatch_ilshift,
    "ilshift",
    "__ilshift__",
    mb_operator_lshift
);
disp_inplace!(
    dispatch_irshift,
    "irshift",
    "__irshift__",
    mb_operator_rshift
);

// iconcat(a, b) == `a += b` for sequences. The first arg MUST be a sequence
// (CPython raises TypeError otherwise); iconcat falls back to __iadd__ when no
// __iconcat__ exists.
unsafe extern "C" fn dispatch_iconcat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs != 2 {
        return arity_error("iconcat", 2, nargs);
    }
    let a = unsafe { *args_ptr };
    let b = unsafe { *args_ptr.add(1) };
    mb_operator_iconcat(a, b)
}

fn mb_operator_iconcat(a: MbValue, b: MbValue) -> MbValue {
    // Instances: prefer __iconcat__, then __iadd__.
    if let Some(cls) = instance_class_name(a) {
        for dunder in ["__iconcat__", "__iadd__"] {
            if !class::lookup_method(&cls, dunder).is_none() {
                let args = MbValue::from_ptr(MbObject::new_list(vec![b]));
                return class::mb_call_method(
                    a,
                    MbValue::from_ptr(MbObject::new_str(dunder.to_string())),
                    args,
                );
            }
        }
    }
    // Built-in sequences only.
    let is_seq = a.as_ptr().is_some_and(|p| unsafe {
        matches!(
            (*p).data,
            ObjData::List(_) | ObjData::Tuple(_) | ObjData::Str(_) | ObjData::Bytes(_)
        )
    });
    if !is_seq {
        raise(
            "TypeError",
            &format!("'{}' object can't be concatenated", type_name(a)),
        );
        return MbValue::none();
    }
    builtins::mb_add(a, b)
}

// ── Generic call (3.11+) ──
unsafe extern "C" fn dispatch_call(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    // operator.call(obj, /, *args, **kwargs) → obj(*args, **kwargs).
    // Forward the callable and the remaining positional args (plus any trailing
    // kwargs dict, which mb_call_spread already understands).
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let func = match a.first().copied() {
        Some(f) => f,
        None => {
            raise("TypeError", "call expected at least 1 argument, got 0");
            return MbValue::none();
        }
    };
    // The call-lowering appends keyword args as a trailing dict. Forward them
    // as real kwargs (mb_call_spread would bind the dict positionally, so
    // `operator.call(f, a=2)` wrongly produced `(({'a':2},), {})`).
    let mut rest: Vec<MbValue> = a[1..].to_vec();
    let trailing_kwargs = rest.last().copied().filter(|v| {
        v.as_ptr().is_some_and(|p| unsafe {
            matches!((*p).data, super::super::rc::ObjData::Dict(_))
        })
    });
    if let Some(kw) = trailing_kwargs {
        rest.pop();
        let pos_list = MbValue::from_ptr(MbObject::new_list(rest));
        return builtins::mb_call_spread_kwargs(func, pos_list, kw);
    }
    let args_list = MbValue::from_ptr(MbObject::new_list(rest));
    builtins::mb_call_spread(func, args_list)
}

// ── Callback-bound classes: itemgetter / attrgetter / methodcaller ──
//
// Each constructor returns an Instance of a dedicated class whose `__call__`
// method (registered in `register`) closes over the constructor arguments held
// in the instance fields. The runtime's generic `__call__` dispatch
// (`builtins::mb_call_spread` → `class::mb_call_method`) invokes these as
// `extern "C" fn(self, args_list)` because the `__call__` methods are
// registered variadic, so any call arity is delivered as a single list.

unsafe extern "C" fn dispatch_itemgetter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if a.is_empty() {
        raise("TypeError", "itemgetter expected 1 argument, got 0");
        return MbValue::none();
    }
    let keys: Vec<MbValue> = a.to_vec();
    for k in &keys {
        unsafe { super::super::rc::retain_if_ptr(*k) };
    }
    let single = a.len() == 1;
    make_instance("operator.itemgetter", |f| {
        f.insert("_single".to_string(), MbValue::from_bool(single));
        f.insert(
            "_keys".to_string(),
            MbValue::from_ptr(MbObject::new_tuple(keys)),
        );
    })
}

unsafe extern "C" fn dispatch_attrgetter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if a.is_empty() {
        raise("TypeError", "attrgetter expected 1 argument, got 0");
        return MbValue::none();
    }
    // Every name argument must be a str (CPython raises TypeError otherwise).
    for v in a {
        if !is_str(*v) {
            raise("TypeError", "attribute name must be a string");
            return MbValue::none();
        }
    }
    let names: Vec<MbValue> = a.to_vec();
    for k in &names {
        unsafe { super::super::rc::retain_if_ptr(*k) };
    }
    let single = a.len() == 1;
    make_instance("operator.attrgetter", |f| {
        f.insert("_single".to_string(), MbValue::from_bool(single));
        f.insert(
            "_names".to_string(),
            MbValue::from_ptr(MbObject::new_tuple(names)),
        );
    })
}

unsafe extern "C" fn dispatch_methodcaller(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if a.is_empty() {
        raise(
            "TypeError",
            "methodcaller needs at least one argument, the method name",
        );
        return MbValue::none();
    }
    let name = a[0];
    if !is_str(name) {
        raise("TypeError", "method name must be a string");
        return MbValue::none();
    }
    // Trailing dict (kwargs lowering) is bound as keyword args; the rest are
    // positional bound args applied ahead of the call-time target.
    let mut bound: Vec<MbValue> = a[1..].to_vec();
    let mut kwargs = MbValue::none();
    if let Some(last) = bound.last().copied() {
        if last
            .as_ptr()
            .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        {
            kwargs = last;
            bound.pop();
        }
    }
    unsafe { super::super::rc::retain_if_ptr(name) };
    for v in &bound {
        unsafe { super::super::rc::retain_if_ptr(*v) };
    }
    unsafe { super::super::rc::retain_if_ptr(kwargs) };
    make_instance("operator.methodcaller", |f| {
        f.insert("_name".to_string(), name);
        f.insert(
            "_args".to_string(),
            MbValue::from_ptr(MbObject::new_list(bound)),
        );
        f.insert("_kwargs".to_string(), kwargs);
    })
}

// ── `__call__` bodies for the three callable classes (variadic: self, args) ──

extern "C" fn op_itemgetter_call(self_inst: MbValue, args_list: MbValue) -> MbValue {
    let items = seq_items(args_list).unwrap_or_default();
    if items.len() != 1 {
        raise("TypeError", "itemgetter expects exactly one call argument");
        return MbValue::none();
    }
    let target = items[0];
    let single = field(self_inst, "_single").as_bool() == Some(true);
    let keys = seq_items(field(self_inst, "_keys")).unwrap_or_default();
    if single {
        return subscript(target, keys[0]);
    }
    let mut out = Vec::with_capacity(keys.len());
    for k in keys {
        let v = subscript(target, k);
        if exception::mb_has_exception().as_bool() == Some(true) {
            return MbValue::none();
        }
        out.push(v);
    }
    MbValue::from_ptr(MbObject::new_tuple(out))
}

extern "C" fn op_attrgetter_call(self_inst: MbValue, args_list: MbValue) -> MbValue {
    let items = seq_items(args_list).unwrap_or_default();
    if items.len() != 1 {
        raise("TypeError", "attrgetter expects exactly one call argument");
        return MbValue::none();
    }
    let target = items[0];
    let single = field(self_inst, "_single").as_bool() == Some(true);
    let names = seq_items(field(self_inst, "_names")).unwrap_or_default();
    let resolve = |obj: MbValue, dotted: &str| -> MbValue {
        let mut cur = obj;
        for seg in dotted.split('.') {
            if seg.is_empty() {
                // Empty path segment ('a.' or '.b') is not a valid attribute.
                raise("AttributeError", "empty attribute name");
                return MbValue::none();
            }
            let attr = MbValue::from_ptr(MbObject::new_str(seg.to_string()));
            // getattr(cur, seg): mb_getattr raises AttributeError on a missing
            // attribute AND dispatches __getattr__ (whose own exceptions, e.g.
            // SyntaxError, must propagate). Only synthesize an AttributeError
            // when mb_getattr returned None *without* setting an exception.
            let next = class::mb_getattr(cur, attr);
            if has_exc() {
                return MbValue::none();
            }
            if next.is_none() && class::mb_hasattr(cur, attr).as_bool() != Some(true) {
                raise(
                    "AttributeError",
                    &format!("'{}' object has no attribute '{seg}'", type_name(cur)),
                );
                return MbValue::none();
            }
            cur = next;
        }
        cur
    };
    if single {
        let n = as_str(names[0]).unwrap_or_default();
        return resolve(target, &n);
    }
    let mut out = Vec::with_capacity(names.len());
    for nm in names {
        let n = as_str(nm).unwrap_or_default();
        let v = resolve(target, &n);
        if exception::mb_has_exception().as_bool() == Some(true) {
            return MbValue::none();
        }
        out.push(v);
    }
    MbValue::from_ptr(MbObject::new_tuple(out))
}

extern "C" fn op_methodcaller_call(self_inst: MbValue, args_list: MbValue) -> MbValue {
    let items = seq_items(args_list).unwrap_or_default();
    if items.is_empty() {
        raise(
            "TypeError",
            "methodcaller needs at least one argument, the target object",
        );
        return MbValue::none();
    }
    let target = items[0];
    let name = field(self_inst, "_name");
    let call_args = seq_items(field(self_inst, "_args")).unwrap_or_default();
    let kwargs = field(self_inst, "_kwargs");
    let args_val = MbValue::from_ptr(MbObject::new_list(call_args));
    if kwargs.is_none() {
        class::mb_call_method(target, name, args_val)
    } else {
        class::mb_call_method_kwargs(target, name, args_val, kwargs)
    }
}

/// Register the operator module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        // Arithmetic
        ("add", dispatch_add as *const () as usize),
        ("sub", dispatch_sub as *const () as usize),
        ("mul", dispatch_mul as *const () as usize),
        ("truediv", dispatch_truediv as *const () as usize),
        ("floordiv", dispatch_floordiv as *const () as usize),
        ("mod", dispatch_mod as *const () as usize),
        ("pow", dispatch_pow as *const () as usize),
        ("matmul", dispatch_matmul as *const () as usize),
        // Bitwise
        ("and_", dispatch_and_ as *const () as usize),
        ("or_", dispatch_or_ as *const () as usize),
        ("xor", dispatch_xor as *const () as usize),
        ("lshift", dispatch_lshift as *const () as usize),
        ("rshift", dispatch_rshift as *const () as usize),
        // Unary
        ("neg", dispatch_neg as *const () as usize),
        ("pos", dispatch_pos as *const () as usize),
        ("not_", dispatch_not_ as *const () as usize),
        ("abs", dispatch_abs as *const () as usize),
        ("inv", dispatch_invert as *const () as usize),
        ("invert", dispatch_invert as *const () as usize),
        ("truth", dispatch_truth as *const () as usize),
        ("index", dispatch_index as *const () as usize),
        ("length_hint", dispatch_length_hint as *const () as usize),
        // Comparison
        ("eq", dispatch_eq as *const () as usize),
        ("ne", dispatch_ne as *const () as usize),
        ("lt", dispatch_lt as *const () as usize),
        ("le", dispatch_le as *const () as usize),
        ("gt", dispatch_gt as *const () as usize),
        ("ge", dispatch_ge as *const () as usize),
        // Identity / membership
        ("is_", dispatch_is_ as *const () as usize),
        ("is_not", dispatch_is_not as *const () as usize),
        ("contains", dispatch_contains as *const () as usize),
        ("countOf", dispatch_countOf as *const () as usize),
        ("indexOf", dispatch_indexOf as *const () as usize),
        // Sequence
        ("getitem", dispatch_getitem as *const () as usize),
        ("setitem", dispatch_setitem as *const () as usize),
        ("delitem", dispatch_delitem as *const () as usize),
        ("concat", dispatch_concat as *const () as usize),
        // In-place
        ("iadd", dispatch_iadd as *const () as usize),
        ("isub", dispatch_isub as *const () as usize),
        ("imul", dispatch_imul as *const () as usize),
        ("itruediv", dispatch_itruediv as *const () as usize),
        ("ifloordiv", dispatch_ifloordiv as *const () as usize),
        ("imod", dispatch_imod as *const () as usize),
        ("ipow", dispatch_ipow as *const () as usize),
        ("imatmul", dispatch_imatmul as *const () as usize),
        ("iand", dispatch_iand as *const () as usize),
        ("ior", dispatch_ior as *const () as usize),
        ("ixor", dispatch_ixor as *const () as usize),
        ("ilshift", dispatch_ilshift as *const () as usize),
        ("irshift", dispatch_irshift as *const () as usize),
        ("iconcat", dispatch_iconcat as *const () as usize),
        // 3.11+ generic call
        ("call", dispatch_call as *const () as usize),
        // Callable-class ctors — surface-only stubs (#2100 closes the loop)
        ("itemgetter", dispatch_itemgetter as *const () as usize),
        ("attrgetter", dispatch_attrgetter as *const () as usize),
        ("methodcaller", dispatch_methodcaller as *const () as usize),
    ];
    // Public names that have a dunder twin in CPython's operator module.
    // `operator.add is operator.__add__`, etc. The twin must be the *same*
    // object (identical func addr) so `is`-identity holds.
    const DUNDER_TWINS: &[&str] = &[
        "abs",
        "add",
        "and_",
        "concat",
        "contains",
        "delitem",
        "eq",
        "floordiv",
        "ge",
        "getitem",
        "gt",
        "iadd",
        "iand",
        "iconcat",
        "ifloordiv",
        "ilshift",
        "imatmul",
        "imod",
        "imul",
        "index",
        "inv",
        "invert",
        "ior",
        "ipow",
        "irshift",
        "isub",
        "itruediv",
        "ixor",
        "le",
        "lshift",
        "lt",
        "matmul",
        "mod",
        "mul",
        "ne",
        "neg",
        "not_",
        "or_",
        "pos",
        "pow",
        "rshift",
        "setitem",
        "sub",
        "truediv",
        "xor",
    ];

    let mut aliases: Vec<(String, usize)> = Vec::new();
    for (name, addr) in &dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
        if DUNDER_TWINS.contains(name) {
            // `and_`/`or_`/`not_`/`is_` strip trailing underscores in the dunder.
            let core = name.trim_end_matches('_');
            aliases.push((format!("__{core}__"), *addr));
        }
    }
    for (dunder, addr) in aliases {
        attrs.insert(dunder, MbValue::from_func(addr));
        // addr already in NATIVE_FUNC_ADDRS via the loop above.
    }

    // Register the three callable classes so their instances are callable via
    // the runtime's generic `__call__` dispatch. The `__call__` methods are
    // variadic (self, args_list) — register them as variadic so the dispatcher
    // delivers any call arity as a single list (mirrors mb_call_method's
    // is_variadic branch) and avoids per-call-arity ABI mismatch.
    register_callable_class(
        "operator.itemgetter",
        op_itemgetter_call as *const () as usize,
    );
    register_callable_class(
        "operator.attrgetter",
        op_attrgetter_call as *const () as usize,
    );
    register_callable_class(
        "operator.methodcaller",
        op_methodcaller_call as *const () as usize,
    );

    super::register_module("operator", attrs);
}

/// Register a class `name` whose only method is `__call__` → `call_addr`,
/// marking that method variadic so it receives `(self, args_list)`.
fn register_callable_class(name: &str, call_addr: usize) {
    super::super::module::register_variadic_func(call_addr as u64);
    let mut methods = HashMap::new();
    methods.insert("__call__".to_string(), MbValue::from_func(call_addr));
    class::mb_class_register(name, vec![], methods);
}

// ── Arithmetic ──

#[inline]
pub fn mb_operator_add(a: MbValue, b: MbValue) -> MbValue {
    let r = builtins::mb_add(a, b);
    // A None result with no pending exception means the operand types are
    // unsupported (e.g. `1 + "a"`). `a + b` never legitimately yields None,
    // so surface the CPython TypeError here.
    if r.is_none() && exception::mb_has_exception().as_bool() != Some(true) {
        raise(
            "TypeError",
            &format!(
                "unsupported operand type(s) for +: '{}' and '{}'",
                type_name(a),
                type_name(b)
            ),
        );
    }
    r
}
/// Surface a CPython "unsupported operand type(s)" TypeError when a runtime
/// binop returned None with no pending exception — meaning the operand types
/// are unsupported (`a OP b` never legitimately yields None).
#[inline]
fn binop_result(r: MbValue, sym: &str, a: MbValue, b: MbValue) -> MbValue {
    if r.is_none() && !has_exc() {
        raise(
            "TypeError",
            &format!(
                "unsupported operand type(s) for {sym}: '{}' and '{}'",
                type_name(a),
                type_name(b)
            ),
        );
    }
    r
}

#[inline]
pub fn mb_operator_sub(a: MbValue, b: MbValue) -> MbValue {
    binop_result(builtins::mb_sub(a, b), "-", a, b)
}
#[inline]
pub fn mb_operator_mul(a: MbValue, b: MbValue) -> MbValue {
    binop_result(builtins::mb_mul(a, b), "*", a, b)
}
#[inline]
pub fn mb_operator_truediv(a: MbValue, b: MbValue) -> MbValue {
    binop_result(builtins::mb_div(a, b), "/", a, b)
}
#[inline]
pub fn mb_operator_floordiv(a: MbValue, b: MbValue) -> MbValue {
    binop_result(builtins::mb_floordiv(a, b), "//", a, b)
}
#[inline]
pub fn mb_operator_mod(a: MbValue, b: MbValue) -> MbValue {
    binop_result(builtins::mb_mod(a, b), "%", a, b)
}
#[inline]
pub fn mb_operator_pow(a: MbValue, b: MbValue) -> MbValue {
    binop_result(builtins::mb_pow(a, b), "** or pow()", a, b)
}
pub fn mb_operator_matmul(a: MbValue, b: MbValue) -> MbValue {
    // No native matmul on numbers — `a @ b` dispatches to __matmul__ when the
    // left operand defines it, else raises TypeError (matching CPython, where
    // `int @ int` is unsupported).
    if let Some(cls) = instance_class_name(a) {
        if !class::lookup_method(&cls, "__matmul__").is_none() {
            let args = MbValue::from_ptr(MbObject::new_list(vec![b]));
            return class::mb_call_method(
                a,
                MbValue::from_ptr(MbObject::new_str("__matmul__".to_string())),
                args,
            );
        }
    }
    raise(
        "TypeError",
        &format!(
            "unsupported operand type(s) for @: '{}' and '{}'",
            type_name(a),
            type_name(b)
        ),
    );
    MbValue::none()
}

// ── Bitwise ──

/// True when `v` can participate in a bitwise op: int / bool / set / frozenset /
/// dict-view, or an instance carrying the matching dunder. None / float / str
/// etc. are rejected (CPython raises TypeError). `None | None` must NOT be
/// treated as a PEP-604 type union here.
fn supports_bitwise(v: MbValue, dunder: &str) -> bool {
    if v.is_int() || v.is_bool() {
        return true;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::BigInt(_) | ObjData::Set(_) | ObjData::FrozenSet(_) => return true,
                ObjData::Instance { class_name, .. } => {
                    return !class::lookup_method(class_name, dunder).is_none();
                }
                _ => return false,
            }
        }
    }
    false
}

fn bitwise_guard(a: MbValue, b: MbValue, sym: &str, dunder: &str) -> bool {
    // Either operand may supply the dunder (reflected form), so require at least
    // one side to support it AND neither side to be an unsupported scalar.
    let ok = (supports_bitwise(a, dunder) && supports_bitwise(b, dunder))
        || instance_class_name(a).is_some()
        || instance_class_name(b).is_some();
    if !ok {
        raise(
            "TypeError",
            &format!(
                "unsupported operand type(s) for {sym}: '{}' and '{}'",
                type_name(a),
                type_name(b)
            ),
        );
        return false;
    }
    true
}

pub fn mb_operator_and(a: MbValue, b: MbValue) -> MbValue {
    if !bitwise_guard(a, b, "&", "__and__") {
        return MbValue::none();
    }
    binop_result(builtins::mb_bitand(a, b), "&", a, b)
}
pub fn mb_operator_or(a: MbValue, b: MbValue) -> MbValue {
    if !bitwise_guard(a, b, "|", "__or__") {
        return MbValue::none();
    }
    binop_result(builtins::mb_bitor(a, b), "|", a, b)
}
pub fn mb_operator_xor(a: MbValue, b: MbValue) -> MbValue {
    if !bitwise_guard(a, b, "^", "__xor__") {
        return MbValue::none();
    }
    binop_result(builtins::mb_bitxor(a, b), "^", a, b)
}
pub fn mb_operator_lshift(a: MbValue, b: MbValue) -> MbValue {
    // Shift only defined for ints (and bool). Non-int operands → TypeError.
    // Negative shift counts → ValueError, matching CPython.
    match (shift_int(a), shift_int(b)) {
        (Some(_), Some(y)) if y < 0 => {
            raise("ValueError", "negative shift count");
            MbValue::none()
        }
        (Some(x), Some(y)) if y < 64 => MbValue::from_int(x.wrapping_shl(y as u32)),
        (Some(x), Some(_)) => MbValue::from_int(if x < 0 { -1 } else { 0 }),
        _ => {
            raise(
                "TypeError",
                &format!(
                    "unsupported operand type(s) for <<: '{}' and '{}'",
                    type_name(a),
                    type_name(b)
                ),
            );
            MbValue::none()
        }
    }
}
pub fn mb_operator_rshift(a: MbValue, b: MbValue) -> MbValue {
    match (shift_int(a), shift_int(b)) {
        (Some(_), Some(y)) if y < 0 => {
            raise("ValueError", "negative shift count");
            MbValue::none()
        }
        (Some(x), Some(y)) if y < 64 => MbValue::from_int(x.wrapping_shr(y as u32)),
        (Some(x), Some(_)) => MbValue::from_int(if x < 0 { -1 } else { 0 }),
        _ => {
            raise(
                "TypeError",
                &format!(
                    "unsupported operand type(s) for >>: '{}' and '{}'",
                    type_name(a),
                    type_name(b)
                ),
            );
            MbValue::none()
        }
    }
}

/// Integer view of a shift operand: true ints and bools only (NOT float/str).
fn shift_int(v: MbValue) -> Option<i64> {
    if v.is_bool() {
        return Some(if v.as_bool() == Some(true) { 1 } else { 0 });
    }
    if v.is_int() {
        return v.as_int();
    }
    None
}

// ── Unary ──

/// True when `v` is a numeric value (int / bool / float / complex / bigint) or a
/// class instance carrying the named arithmetic dunder. Used to gate unary ops
/// so `operator.neg(None)` / `abs(None)` raise TypeError like CPython.
fn supports_unary(v: MbValue, dunder: &str) -> bool {
    if v.is_int() || v.is_bool() || v.is_float() {
        return true;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::BigInt(_) | ObjData::Complex(_, _) => return true,
                ObjData::Instance { class_name, .. } => {
                    return !class::lookup_method(class_name, dunder).is_none();
                }
                _ => return false,
            }
        }
    }
    false
}

pub fn mb_operator_neg(a: MbValue) -> MbValue {
    if !supports_unary(a, "__neg__") {
        raise(
            "TypeError",
            &format!("bad operand type for unary -: '{}'", type_name(a)),
        );
        return MbValue::none();
    }
    builtins::mb_neg(a)
}
pub fn mb_operator_pos(a: MbValue) -> MbValue {
    if !supports_unary(a, "__pos__") {
        raise(
            "TypeError",
            &format!("bad operand type for unary +: '{}'", type_name(a)),
        );
        return MbValue::none();
    }
    // +x is identity on numerics; dispatch to __pos__ for instances.
    if let Some(cls) = instance_class_name(a) {
        if !class::lookup_method(&cls, "__pos__").is_none() {
            return class::mb_call_method(
                a,
                MbValue::from_ptr(MbObject::new_str("__pos__".to_string())),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
    }
    a
}
pub fn mb_operator_not(a: MbValue) -> MbValue {
    builtins::mb_not(a)
}
pub fn mb_operator_abs(a: MbValue) -> MbValue {
    if !supports_unary(a, "__abs__") {
        raise(
            "TypeError",
            &format!("bad operand type for abs(): '{}'", type_name(a)),
        );
        return MbValue::none();
    }
    builtins::mb_abs(a)
}
pub fn mb_operator_invert(a: MbValue) -> MbValue {
    // ~x is defined for ints/bools and instances with __invert__.
    if a.is_bool() {
        let x = if a.as_bool() == Some(true) { 1 } else { 0 };
        return MbValue::from_int(!x);
    }
    if let Some(x) = a.as_int() {
        return MbValue::from_int(!x);
    }
    if let Some(cls) = instance_class_name(a) {
        if !class::lookup_method(&cls, "__invert__").is_none() {
            return class::mb_call_method(
                a,
                MbValue::from_ptr(MbObject::new_str("__invert__".to_string())),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
    }
    raise(
        "TypeError",
        &format!("bad operand type for unary ~: '{}'", type_name(a)),
    );
    MbValue::none()
}
pub fn mb_operator_truth(a: MbValue) -> MbValue {
    builtins::mb_bool(a)
}
pub fn mb_operator_index(a: MbValue) -> MbValue {
    // operator.index(x) == x.__index__(). True ints (incl. bool) pass through;
    // floats and other non-integers raise TypeError; instances dispatch to
    // their __index__ dunder.
    if a.is_bool() {
        return MbValue::from_int(if a.as_bool() == Some(true) { 1 } else { 0 });
    }
    if a.is_int() {
        return a;
    }
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::BigInt(_) = (*ptr).data {
                return a;
            }
        }
    }
    if let Some(cls) = instance_class_name(a) {
        if !class::lookup_method(&cls, "__index__").is_none() {
            let r = class::mb_call_method(
                a,
                MbValue::from_ptr(MbObject::new_str("__index__".to_string())),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
            return r;
        }
    }
    raise(
        "TypeError",
        &format!(
            "'{}' object cannot be interpreted as an integer",
            type_name(a)
        ),
    );
    MbValue::none()
}

/// The `default` value to return when no length information is available —
/// 0 when the caller omitted it (default is None).
fn length_hint_default(default: MbValue) -> MbValue {
    if default.is_none() {
        MbValue::from_int(0)
    } else {
        default
    }
}

/// operator.length_hint(obj, default=0).
fn operator_length_hint(obj: MbValue, default: MbValue) -> MbValue {
    // Validate the default first (must be an int) — CPython checks this eagerly.
    if !default.is_none() && !default.is_int() && !default.is_bool() {
        raise(
            "TypeError",
            &format!(
                "'{}' object cannot be interpreted as an integer",
                type_name(default)
            ),
        );
        return MbValue::none();
    }
    // For user/stdlib instances, consult the dunders explicitly: mb_len returns
    // 0 for instances that define neither __len__ nor __length_hint__, which
    // would mask the proper fallback, so we do not trust it for instances.
    if let Some(cls) = instance_class_name(obj) {
        if !class::lookup_method(&cls, "__len__").is_none() {
            return class::mb_call_method(
                obj,
                MbValue::from_ptr(MbObject::new_str("__len__".to_string())),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
        if !class::lookup_method(&cls, "__length_hint__").is_none() {
            let r = class::mb_call_method(
                obj,
                MbValue::from_ptr(MbObject::new_str("__length_hint__".to_string())),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
            // CPython's PyObject_LengthHint: a TypeError raised inside
            // __length_hint__ is swallowed and the default is returned; any
            // other exception propagates.
            if has_exc() {
                // A TypeError (or subclass) is swallowed → default; else propagate.
                if exception::current_exception_type().as_deref() == Some("TypeError") {
                    exception::mb_clear_exception();
                    return length_hint_default(default);
                }
                return MbValue::none();
            }
            // NotImplemented → fall back to the supplied default.
            if r.is_not_implemented() {
                return length_hint_default(default);
            }
            // Result must be a non-negative integer.
            match r.as_int() {
                Some(n) if n < 0 => {
                    raise("ValueError", "__length_hint__() should return >= 0");
                    return MbValue::none();
                }
                Some(_) => return r,
                None => {
                    raise(
                        "TypeError",
                        &format!("__length_hint__ must be an integer, not {}", type_name(r)),
                    );
                    return MbValue::none();
                }
            }
        }
        // No length information on this instance → supplied default.
        return length_hint_default(default);
    }
    // Iterator handles are bare ints (no __len__). Consult the iterator's
    // __length_hint__ (remaining-element count) first — mb_len returns a
    // misleading 0 for these.
    if obj.as_ptr().is_none() {
        if let Some(n) = super::super::iter::mb_iter_length_hint(obj) {
            return MbValue::from_int(n);
        }
        return if default.is_none() {
            MbValue::from_int(0)
        } else {
            default
        };
    }
    // Built-in sized containers: __len__ wins.
    let l = builtins::mb_len(obj);
    if !l.is_none() && l.as_int().is_some() {
        return l;
    }
    // Iterators expose a __length_hint__ — the count of remaining elements.
    if let Some(n) = super::super::iter::mb_iter_length_hint(obj) {
        return MbValue::from_int(n);
    }
    // Fall back to the supplied default (0 when omitted).
    if default.is_none() {
        MbValue::from_int(0)
    } else {
        default
    }
}

pub fn mb_operator_length_hint(a: MbValue) -> MbValue {
    operator_length_hint(a, MbValue::none())
}

// ── Comparison ──

/// Dispatch a comparison dunder (`__eq__`/`__ne__`/…) on an instance when it
/// defines that method. Returns Some(result) — possibly NotImplemented — when
/// the dunder ran (so the caller honours exceptions raised inside it); None when
/// `a` is not an instance carrying that dunder.
fn instance_richcmp(a: MbValue, b: MbValue, dunder: &str) -> Option<MbValue> {
    let cls = instance_class_name(a)?;
    if class::lookup_method(&cls, dunder).is_none() {
        return None;
    }
    let args = MbValue::from_ptr(MbObject::new_list(vec![b]));
    let r = class::mb_call_method(
        a,
        MbValue::from_ptr(MbObject::new_str(dunder.to_string())),
        args,
    );
    Some(r)
}

#[inline]
pub fn mb_operator_eq(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_eq(a, b)
}
#[inline]
pub fn mb_operator_ne(a: MbValue, b: MbValue) -> MbValue {
    // CPython's `!=` prefers `__ne__`; mamba's mb_ne ignores instance dunders, so
    // dispatch `__ne__` explicitly (exceptions inside it propagate). Fall back to
    // negating `__eq__`, else the structural mb_ne.
    if let Some(r) = instance_richcmp(a, b, "__ne__") {
        if has_exc() {
            return MbValue::none();
        }
        if !r.is_not_implemented() {
            return r;
        }
    }
    if instance_class_name(a).is_some() {
        let eq = builtins::mb_eq(a, b);
        if has_exc() {
            return MbValue::none();
        }
        if let Some(t) = eq.as_bool() {
            return MbValue::from_bool(!t);
        }
    }
    builtins::mb_ne(a, b)
}
/// Ordered comparisons are undefined on complex numbers — CPython raises
/// `TypeError: '<' not supported between instances of 'complex' and 'complex'`.
/// mamba's runtime `mb_lt` etc. silently return False for complex, so gate here.
fn ordered_complex_guard(sym: &str, a: MbValue, b: MbValue) -> bool {
    if is_complex(a) || is_complex(b) {
        raise(
            "TypeError",
            &format!(
                "'{sym}' not supported between instances of '{}' and '{}'",
                type_name(a),
                type_name(b)
            ),
        );
        return true;
    }
    false
}

#[inline]
pub fn mb_operator_lt(a: MbValue, b: MbValue) -> MbValue {
    if ordered_complex_guard("<", a, b) {
        return MbValue::none();
    }
    builtins::mb_lt(a, b)
}
#[inline]
pub fn mb_operator_le(a: MbValue, b: MbValue) -> MbValue {
    if ordered_complex_guard("<=", a, b) {
        return MbValue::none();
    }
    builtins::mb_le(a, b)
}
#[inline]
pub fn mb_operator_gt(a: MbValue, b: MbValue) -> MbValue {
    if ordered_complex_guard(">", a, b) {
        return MbValue::none();
    }
    builtins::mb_gt(a, b)
}
#[inline]
pub fn mb_operator_ge(a: MbValue, b: MbValue) -> MbValue {
    if ordered_complex_guard(">=", a, b) {
        return MbValue::none();
    }
    builtins::mb_ge(a, b)
}

// ── Identity / membership ──

pub fn mb_operator_is(a: MbValue, b: MbValue) -> MbValue {
    // Compare raw value bits; matches CPython's `is` for primitive payloads
    // and ptr-eq for heap objects. MbValue's PartialEq compares u64 bits.
    MbValue::from_bool(a == b)
}
pub fn mb_operator_is_not(a: MbValue, b: MbValue) -> MbValue {
    MbValue::from_bool(a != b)
}
/// CPython's `PyObject_RichCompareBool(v, item, Py_EQ)` used by sequence search:
/// identical objects compare equal *without* calling __eq__ (so `nan in [nan]`
/// is True), otherwise fall back to `==`. Returns None on a pending exception.
fn search_eq(v: MbValue, item: MbValue) -> Option<bool> {
    if v == item {
        return Some(true);
    }
    let eq = builtins::mb_eq(v, item);
    if has_exc() {
        return None;
    }
    Some(eq.as_bool() == Some(true))
}

pub fn mb_operator_contains(container: MbValue, item: MbValue) -> MbValue {
    // `item in container` — str / dict / set use their own membership test
    // (substring / key / hash). Everything else iterates via the iterator
    // protocol, matching CPython's `_PySequence_IterSearch`.
    if let Some(ptr) = container.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => {
                    return super::super::string_ops::mb_str_contains(container, item)
                }
                ObjData::Dict(_) => {
                    return super::super::dict_ops::mb_dict_contains(container, item)
                }
                ObjData::Set(_) | ObjData::FrozenSet(_) => {
                    return super::super::set_ops::mb_set_contains(container, item)
                }
                _ => {}
            }
        }
    }
    // Iterate; True on first match. None / exhausted → False. Iteration errors
    // (BadIterable, non-iterable) propagate as a pending exception.
    let iter = super::super::iter::mb_iter(container);
    if iter.is_none() {
        return MbValue::none();
    }
    loop {
        let v = super::super::iter::mb_next_or_stop(iter);
        if v.is_stop_iter_sentinel() {
            return MbValue::from_bool(false);
        }
        if has_exc() {
            return MbValue::none();
        }
        match search_eq(v, item) {
            None => return MbValue::none(),
            Some(true) => return MbValue::from_bool(true),
            Some(false) => {}
        }
    }
}
pub fn mb_operator_count_of(container: MbValue, item: MbValue) -> MbValue {
    let iter = super::super::iter::mb_iter(container);
    if iter.is_none() {
        return MbValue::none();
    }
    let mut count = 0i64;
    loop {
        let v = super::super::iter::mb_next_or_stop(iter);
        if v.is_stop_iter_sentinel() {
            return MbValue::from_int(count);
        }
        if has_exc() {
            return MbValue::none();
        }
        match search_eq(v, item) {
            None => return MbValue::none(),
            Some(true) => count += 1,
            Some(false) => {}
        }
    }
}
pub fn mb_operator_index_of(container: MbValue, item: MbValue) -> MbValue {
    // indexOf(a, b) returns the index of the first b in a, else raises
    // ValueError. When `a` is already an iterator, the search leaves it
    // positioned just after the match (CPython iterator-search semantics).
    let iter = super::super::iter::mb_iter(container);
    if iter.is_none() {
        return MbValue::none();
    }
    let mut i = 0i64;
    loop {
        let v = super::super::iter::mb_next_or_stop(iter);
        if v.is_stop_iter_sentinel() {
            raise("ValueError", "sequence.index(x): x not in sequence");
            return MbValue::none();
        }
        if has_exc() {
            return MbValue::none();
        }
        match search_eq(v, item) {
            None => return MbValue::none(),
            Some(true) => return MbValue::from_int(i),
            Some(false) => {}
        }
        i += 1;
    }
}

// ── Sequence subscript / concat ──

pub fn mb_operator_getitem(container: MbValue, key: MbValue) -> MbValue {
    // Full subscript semantics (list/tuple/str/dict/slice/negative) with the
    // CPython-correct IndexError / KeyError / TypeError.
    subscript(container, key)
}

/// operator.setitem(obj, key, value) → `obj[key] = value`; returns None.
fn operator_setitem(obj: MbValue, key: MbValue, value: MbValue) -> MbValue {
    let ptr = match obj.as_ptr() {
        Some(p) => p,
        None => {
            raise(
                "TypeError",
                "'NoneType' object does not support item assignment",
            );
            return MbValue::none();
        }
    };
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => {
                if key.is_float() || is_str(key) {
                    raise("TypeError", "list indices must be integers or slices");
                    return MbValue::none();
                }
                match key.as_int() {
                    Some(i) => {
                        let mut guard = lock.write().unwrap();
                        let n = guard.len() as i64;
                        let r = if i < 0 { i + n } else { i };
                        if r >= 0 && r < n {
                            let old = guard[r as usize];
                            super::super::rc::retain_if_ptr(value);
                            guard[r as usize] = value;
                            drop(guard);
                            super::super::rc::release_if_ptr(old);
                        } else {
                            raise("IndexError", "list assignment index out of range");
                        }
                    }
                    None => {
                        raise("TypeError", "list indices must be integers or slices");
                    }
                }
            }
            ObjData::Dict(_) => {
                super::super::dict_ops::mb_dict_setitem(obj, key, value);
            }
            _ => {
                raise(
                    "TypeError",
                    &format!(
                        "'{}' object does not support item assignment",
                        type_name(obj)
                    ),
                );
            }
        }
    }
    MbValue::none()
}

/// operator.delitem(obj, key) → `del obj[key]`; returns None.
fn operator_delitem(obj: MbValue, key: MbValue) -> MbValue {
    let ptr = match obj.as_ptr() {
        Some(p) => p,
        None => {
            raise(
                "TypeError",
                "'NoneType' object doesn't support item deletion",
            );
            return MbValue::none();
        }
    };
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => {
                if key.is_float() || is_str(key) || key.is_none() {
                    raise("TypeError", "list indices must be integers or slices");
                    return MbValue::none();
                }
                match key.as_int() {
                    Some(i) => {
                        let mut guard = lock.write().unwrap();
                        let n = guard.len() as i64;
                        let r = if i < 0 { i + n } else { i };
                        if r >= 0 && r < n {
                            let old = guard.remove(r as usize);
                            drop(guard);
                            super::super::rc::release_if_ptr(old);
                        } else {
                            raise("IndexError", "list assignment index out of range");
                        }
                    }
                    None => {
                        raise("TypeError", "list indices must be integers or slices");
                    }
                }
            }
            ObjData::Dict(lock) => {
                let dk = super::super::dict_ops::to_dict_key(key);
                let removed = {
                    let mut guard = lock.write().unwrap();
                    guard.shift_remove(&dk)
                };
                match removed {
                    Some(old) => super::super::rc::release_if_ptr(old),
                    None => {
                        let repr = super::super::dict_ops::dict_key_raw_str(&dk);
                        raise("KeyError", &repr);
                    }
                }
            }
            _ => {
                raise(
                    "TypeError",
                    &format!("'{}' object doesn't support item deletion", type_name(obj)),
                );
            }
        }
    }
    MbValue::none()
}

pub fn mb_operator_concat(a: MbValue, b: MbValue) -> MbValue {
    // concat(a, b) == a + b but ONLY for sequences. Numbers raise TypeError even
    // though `+` would add them (CPython: "'int' object can't be concatenated").
    // Instances dispatch through __add__/__concat__.
    if let Some(cls) = instance_class_name(a) {
        // Instances delegate to __add__ (mb_add does NOT dispatch generic
        // instance dunders — only specific built-in pairs). Try __add__, then
        // __radd__ on b, else TypeError.
        if !class::lookup_method(&cls, "__add__").is_none() {
            let args = MbValue::from_ptr(MbObject::new_list(vec![b]));
            let r = class::mb_call_method(
                a,
                MbValue::from_ptr(MbObject::new_str("__add__".to_string())),
                args,
            );
            if has_exc() {
                return MbValue::none();
            }
            if !r.is_not_implemented() {
                return r;
            }
        }
        raise(
            "TypeError",
            &format!("'{}' object can't be concatenated", type_name(a)),
        );
        return MbValue::none();
    }
    let is_seq = a.as_ptr().is_some_and(|p| unsafe {
        matches!(
            (*p).data,
            ObjData::List(_)
                | ObjData::Tuple(_)
                | ObjData::Str(_)
                | ObjData::Bytes(_)
                | ObjData::ByteArray(_)
        )
    });
    if !is_seq {
        raise(
            "TypeError",
            &format!("'{}' object can't be concatenated", type_name(a)),
        );
        return MbValue::none();
    }
    let r = builtins::mb_add(a, b);
    if r.is_none() && !has_exc() {
        raise(
            "TypeError",
            &format!("can only concatenate {} to {}", type_name(b), type_name(a)),
        );
    }
    r
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_forward() {
        let a = MbValue::from_int(10);
        let b = MbValue::from_int(3);
        assert_eq!(mb_operator_add(a, b).as_int(), Some(13));
        assert_eq!(mb_operator_sub(a, b).as_int(), Some(7));
        assert_eq!(mb_operator_mul(a, b).as_int(), Some(30));
        assert_eq!(mb_operator_floordiv(a, b).as_int(), Some(3));
        assert_eq!(mb_operator_mod(a, b).as_int(), Some(1));
        assert_eq!(
            mb_operator_pow(MbValue::from_int(2), MbValue::from_int(10)).as_int(),
            Some(1024)
        );
    }

    #[test]
    fn bitwise_forward() {
        let a = MbValue::from_int(0b1100);
        let b = MbValue::from_int(0b1010);
        assert_eq!(mb_operator_and(a, b).as_int(), Some(0b1000));
        assert_eq!(mb_operator_or(a, b).as_int(), Some(0b1110));
        assert_eq!(mb_operator_xor(a, b).as_int(), Some(0b0110));
        assert_eq!(
            mb_operator_lshift(MbValue::from_int(1), MbValue::from_int(4)).as_int(),
            Some(16)
        );
        assert_eq!(
            mb_operator_rshift(MbValue::from_int(16), MbValue::from_int(2)).as_int(),
            Some(4)
        );
        assert_eq!(mb_operator_invert(MbValue::from_int(0)).as_int(), Some(-1));
    }

    #[test]
    fn unary_forward() {
        assert_eq!(mb_operator_neg(MbValue::from_int(7)).as_int(), Some(-7));
        assert_eq!(mb_operator_pos(MbValue::from_int(7)).as_int(), Some(7));
        assert_eq!(mb_operator_abs(MbValue::from_int(-5)).as_int(), Some(5));
        assert_eq!(mb_operator_not(MbValue::from_int(0)).as_bool(), Some(true));
        assert_eq!(
            mb_operator_truth(MbValue::from_int(0)).as_bool(),
            Some(false)
        );
        assert_eq!(
            mb_operator_truth(MbValue::from_int(42)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn comparison_forward() {
        let a = MbValue::from_int(3);
        let b = MbValue::from_int(5);
        assert_eq!(mb_operator_lt(a, b).as_bool(), Some(true));
        assert_eq!(mb_operator_gt(a, b).as_bool(), Some(false));
        assert_eq!(mb_operator_eq(a, a).as_bool(), Some(true));
        assert_eq!(mb_operator_ne(a, b).as_bool(), Some(true));
        assert_eq!(mb_operator_le(a, b).as_bool(), Some(true));
        assert_eq!(mb_operator_ge(b, a).as_bool(), Some(true));
    }

    #[test]
    fn identity_forward() {
        let a = MbValue::from_int(42);
        assert_eq!(mb_operator_is(a, a).as_bool(), Some(true));
        assert_eq!(
            mb_operator_is_not(MbValue::from_int(1), MbValue::from_int(2)).as_bool(),
            Some(true)
        );
    }
}
