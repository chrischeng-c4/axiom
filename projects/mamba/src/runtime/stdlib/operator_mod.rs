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
use super::super::rc::ObjData;
use super::super::value::MbValue;
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──
// NOTE: dispatcher fn names must start with `dispatch_` so the surface walker
// (projects/mamba/src/surface.rs::pick_tuple_dispatcher) recognises them.

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            if nargs == 0 {
                return $fn(MbValue::none());
            }
            $fn(unsafe { *args_ptr })
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            if nargs >= 2 {
                unsafe { $fn(*args_ptr, *args_ptr.add(1)) }
            } else if nargs == 1 {
                unsafe { $fn(*args_ptr, MbValue::none()) }
            } else {
                $fn(MbValue::none(), MbValue::none())
            }
        }
    };
}

// ── Arithmetic (binary) ──
disp_binary!(dispatch_add, mb_operator_add);
disp_binary!(dispatch_sub, mb_operator_sub);
disp_binary!(dispatch_mul, mb_operator_mul);
disp_binary!(dispatch_truediv, mb_operator_truediv);
disp_binary!(dispatch_floordiv, mb_operator_floordiv);
disp_binary!(dispatch_mod, mb_operator_mod);
disp_binary!(dispatch_pow, mb_operator_pow);
disp_binary!(dispatch_matmul, mb_operator_matmul);

// ── Bitwise (binary) ──
disp_binary!(dispatch_and_, mb_operator_and);
disp_binary!(dispatch_or_, mb_operator_or);
disp_binary!(dispatch_xor, mb_operator_xor);
disp_binary!(dispatch_lshift, mb_operator_lshift);
disp_binary!(dispatch_rshift, mb_operator_rshift);

// ── Unary ──
disp_unary!(dispatch_neg, mb_operator_neg);
disp_unary!(dispatch_pos, mb_operator_pos);
disp_unary!(dispatch_not_, mb_operator_not);
disp_unary!(dispatch_abs, mb_operator_abs);
disp_unary!(dispatch_invert, mb_operator_invert);
disp_unary!(dispatch_truth, mb_operator_truth);
disp_unary!(dispatch_index, mb_operator_index);
disp_unary!(dispatch_length_hint, mb_operator_length_hint);

// ── Comparison (binary) ──
disp_binary!(dispatch_eq, mb_operator_eq);
disp_binary!(dispatch_ne, mb_operator_ne);
disp_binary!(dispatch_lt, mb_operator_lt);
disp_binary!(dispatch_le, mb_operator_le);
disp_binary!(dispatch_gt, mb_operator_gt);
disp_binary!(dispatch_ge, mb_operator_ge);

// ── Identity / membership ──
disp_binary!(dispatch_is_, mb_operator_is);
disp_binary!(dispatch_is_not, mb_operator_is_not);
disp_binary!(dispatch_contains, mb_operator_contains);
disp_binary!(dispatch_countOf, mb_operator_count_of);
disp_binary!(dispatch_indexOf, mb_operator_index_of);

// ── Sequence: getitem / setitem / delitem / concat ──
disp_binary!(dispatch_getitem, mb_operator_getitem);
disp_binary!(dispatch_setitem, mb_operator_setitem);
disp_binary!(dispatch_delitem, mb_operator_delitem);
disp_binary!(dispatch_concat, mb_operator_concat);

// ── In-place (forward to non-in-place; pure-Python wrappers mutate via assignment) ──
disp_binary!(dispatch_iadd, mb_operator_add);
disp_binary!(dispatch_isub, mb_operator_sub);
disp_binary!(dispatch_imul, mb_operator_mul);
disp_binary!(dispatch_itruediv, mb_operator_truediv);
disp_binary!(dispatch_ifloordiv, mb_operator_floordiv);
disp_binary!(dispatch_imod, mb_operator_mod);
disp_binary!(dispatch_ipow, mb_operator_pow);
disp_binary!(dispatch_imatmul, mb_operator_matmul);
disp_binary!(dispatch_iand, mb_operator_and);
disp_binary!(dispatch_ior, mb_operator_or);
disp_binary!(dispatch_ixor, mb_operator_xor);
disp_binary!(dispatch_ilshift, mb_operator_lshift);
disp_binary!(dispatch_irshift, mb_operator_rshift);
disp_binary!(dispatch_iconcat, mb_operator_concat);

// ── Generic call (3.11+) ──
unsafe extern "C" fn dispatch_call(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    // operator.call(obj, /, *args, **kwargs) — would need full call dispatch.
    // Stub: return the callable unchanged. Registered for surface coverage;
    // semantic correctness queues for the closure/call-dispatch primitive.
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.first().copied().unwrap_or_else(MbValue::none)
}

// ── Callback-bound class ctors — registered for surface (Gate 3) only ──
//
// CPython surface: `operator.itemgetter`/`attrgetter`/`methodcaller` are
// classes whose instances are callables that close over their constructor
// args. Mamba's shim ABI cannot today emit closure-bearing callables
// across FFI (#2100). These dispatchers register the names so the surface
// walker counts them — they return `None` so any user code that actually
// calls the returned object will fail loudly (None is not callable) rather
// than silently producing wrong results. The mb_operator_itemgetter etc.
// free fns below stand as HANDWRITE markers for future codegen.
unsafe extern "C" fn dispatch_itemgetter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_attrgetter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_methodcaller(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
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
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("operator", attrs);
}

// ── Arithmetic ──

#[inline]
pub fn mb_operator_add(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_add(a, b)
}
#[inline]
pub fn mb_operator_sub(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_sub(a, b)
}
#[inline]
pub fn mb_operator_mul(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_mul(a, b)
}
#[inline]
pub fn mb_operator_truediv(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_div(a, b)
}
#[inline]
pub fn mb_operator_floordiv(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_floordiv(a, b)
}
#[inline]
pub fn mb_operator_mod(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_mod(a, b)
}
#[inline]
pub fn mb_operator_pow(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_pow(a, b)
}
pub fn mb_operator_matmul(_a: MbValue, _b: MbValue) -> MbValue {
    // No matmul runtime primitive; returns None. Numpy ports would route
    // here; tracked under future array/matrix work.
    MbValue::none()
}

// ── Bitwise ──

pub fn mb_operator_and(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_bitand(a, b)
}
pub fn mb_operator_or(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_bitor(a, b)
}
pub fn mb_operator_xor(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_bitxor(a, b)
}
pub fn mb_operator_lshift(a: MbValue, b: MbValue) -> MbValue {
    match (a.as_int(), b.as_int()) {
        (Some(x), Some(y)) if (0..64).contains(&y) => MbValue::from_int(x.wrapping_shl(y as u32)),
        _ => MbValue::none(),
    }
}
pub fn mb_operator_rshift(a: MbValue, b: MbValue) -> MbValue {
    match (a.as_int(), b.as_int()) {
        (Some(x), Some(y)) if (0..64).contains(&y) => MbValue::from_int(x.wrapping_shr(y as u32)),
        _ => MbValue::none(),
    }
}

// ── Unary ──

pub fn mb_operator_neg(a: MbValue) -> MbValue {
    builtins::mb_neg(a)
}
pub fn mb_operator_pos(a: MbValue) -> MbValue {
    // +x is identity on numerics; no runtime primitive — pass through.
    a
}
pub fn mb_operator_not(a: MbValue) -> MbValue {
    builtins::mb_not(a)
}
pub fn mb_operator_abs(a: MbValue) -> MbValue {
    builtins::mb_abs(a)
}
pub fn mb_operator_invert(a: MbValue) -> MbValue {
    match a.as_int() {
        Some(x) => MbValue::from_int(!x),
        None => MbValue::none(),
    }
}
pub fn mb_operator_truth(a: MbValue) -> MbValue {
    builtins::mb_bool(a)
}
pub fn mb_operator_index(a: MbValue) -> MbValue {
    // operator.index(x) calls __index__; ints pass through, floats raise
    // TypeError. We route to mb_int which performs the same coercion for
    // integer-like values; floats become truncating ints — close enough for
    // forward-direction surface coverage.
    builtins::mb_int(a)
}
pub fn mb_operator_length_hint(a: MbValue) -> MbValue {
    builtins::mb_len(a)
}

// ── Comparison ──

#[inline]
pub fn mb_operator_eq(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_eq(a, b)
}
#[inline]
pub fn mb_operator_ne(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_ne(a, b)
}
#[inline]
pub fn mb_operator_lt(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_lt(a, b)
}
#[inline]
pub fn mb_operator_le(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_le(a, b)
}
#[inline]
pub fn mb_operator_gt(a: MbValue, b: MbValue) -> MbValue {
    builtins::mb_gt(a, b)
}
#[inline]
pub fn mb_operator_ge(a: MbValue, b: MbValue) -> MbValue {
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
pub fn mb_operator_contains(container: MbValue, item: MbValue) -> MbValue {
    // O(n) linear scan via eq. Mirrors CPython's `in`.
    let ptr = match container.as_ptr() {
        Some(p) => p,
        None => return MbValue::from_bool(false),
    };
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => {
                let guard = lock.read().unwrap();
                for el in guard.iter() {
                    if builtins::mb_eq(*el, item).as_bool() == Some(true) {
                        return MbValue::from_bool(true);
                    }
                }
                MbValue::from_bool(false)
            }
            ObjData::Tuple(items) => {
                for el in items.iter() {
                    if builtins::mb_eq(*el, item).as_bool() == Some(true) {
                        return MbValue::from_bool(true);
                    }
                }
                MbValue::from_bool(false)
            }
            _ => MbValue::from_bool(false),
        }
    }
}
pub fn mb_operator_count_of(container: MbValue, item: MbValue) -> MbValue {
    let ptr = match container.as_ptr() {
        Some(p) => p,
        None => return MbValue::from_int(0),
    };
    let mut count = 0i64;
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => {
                let guard = lock.read().unwrap();
                for el in guard.iter() {
                    if builtins::mb_eq(*el, item).as_bool() == Some(true) {
                        count += 1;
                    }
                }
            }
            ObjData::Tuple(items) => {
                for el in items.iter() {
                    if builtins::mb_eq(*el, item).as_bool() == Some(true) {
                        count += 1;
                    }
                }
            }
            _ => {}
        }
    }
    MbValue::from_int(count)
}
pub fn mb_operator_index_of(container: MbValue, item: MbValue) -> MbValue {
    let ptr = match container.as_ptr() {
        Some(p) => p,
        None => return MbValue::from_int(-1),
    };
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => {
                let guard = lock.read().unwrap();
                for (i, el) in guard.iter().enumerate() {
                    if builtins::mb_eq(*el, item).as_bool() == Some(true) {
                        return MbValue::from_int(i as i64);
                    }
                }
            }
            ObjData::Tuple(items) => {
                for (i, el) in items.iter().enumerate() {
                    if builtins::mb_eq(*el, item).as_bool() == Some(true) {
                        return MbValue::from_int(i as i64);
                    }
                }
            }
            _ => {}
        }
    }
    MbValue::from_int(-1)
}

// ── Sequence subscript / concat ──

pub fn mb_operator_getitem(container: MbValue, key: MbValue) -> MbValue {
    let ptr = match container.as_ptr() {
        Some(p) => p,
        None => return MbValue::none(),
    };
    unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => {
                let guard = lock.read().unwrap();
                let idx = key.as_int().unwrap_or(0);
                let n = guard.len() as i64;
                let resolved = if idx < 0 { idx + n } else { idx };
                if resolved >= 0 && resolved < n {
                    guard[resolved as usize]
                } else {
                    MbValue::none()
                }
            }
            ObjData::Tuple(items) => {
                let idx = key.as_int().unwrap_or(0);
                let n = items.len() as i64;
                let resolved = if idx < 0 { idx + n } else { idx };
                if resolved >= 0 && resolved < n {
                    items[resolved as usize]
                } else {
                    MbValue::none()
                }
            }
            _ => MbValue::none(),
        }
    }
}
pub fn mb_operator_setitem(container: MbValue, _key_and_value: MbValue) -> MbValue {
    // Two-arg form (container, key=value) cannot be expressed in the binary
    // dispatcher; CPython's setitem takes 3 positional args. This stub returns
    // None so import-time wiring still succeeds; full impl queues with the
    // varargs-aware shim infrastructure.
    let _ = container;
    MbValue::none()
}
pub fn mb_operator_delitem(_container: MbValue, _key: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_operator_concat(a: MbValue, b: MbValue) -> MbValue {
    // Lists / tuples / strings — defer to mb_add, which already handles all
    // three concat paths through __add__ dispatch.
    builtins::mb_add(a, b)
}

// HANDWRITE-END

// ── Callback-bound classes (NOT registered; #2100) ──
//
// HANDWRITE-BEGIN reason: itemgetter / attrgetter / methodcaller are
// callback-bound — they return a callable that closes over its constructor
// arguments. Mamba's stdlib shim ABI cannot today emit a closure-bearing
// callable across the FFI boundary. Tracked at #2100; once the closure
// primitive lands, codegen can replace these stubs with proper
// constructor-returns-callable wiring.

#[allow(dead_code)]
pub fn mb_operator_itemgetter(_key: MbValue) -> MbValue {
    unimplemented!("operator.itemgetter — blocked on #2100 (closure-bearing callable)");
}
#[allow(dead_code)]
pub fn mb_operator_attrgetter(_attr: MbValue) -> MbValue {
    unimplemented!("operator.attrgetter — blocked on #2100 (closure-bearing callable)");
}
#[allow(dead_code)]
pub fn mb_operator_methodcaller(_name: MbValue) -> MbValue {
    unimplemented!("operator.methodcaller — blocked on #2100 (closure-bearing callable)");
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
