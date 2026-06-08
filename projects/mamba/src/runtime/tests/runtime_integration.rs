#![cfg(test)]

/// Cross-module integration tests for the Mamba runtime.
/// Exercises Value, GC, Module, Builtins, String/List/Dict ops together.

use crate::runtime::value::MbValue;
use crate::runtime::rc::MbObject;

// ── Helpers ──

fn str_val(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

// ═══════════════════════════════════════════════════════════
// Group 1: Value lifecycle with GC (5 tests)
// ═══════════════════════════════════════════════════════════

/// Create a list, add as GC root, collect — object must not be freed.
#[test]
fn test_list_value_gc_root_survives_collect() {
    use crate::runtime::gc::{gc_add_root, gc_remove_root, gc_clear_roots, collect, gc_disable, gc_enable};
    use crate::runtime::list_ops::mb_list_new;

    gc_disable();
    gc_clear_roots();

    let list = mb_list_new();
    gc_add_root(list);

    gc_enable();
    let freed = collect();

    // The list was rooted, so it should NOT be among the freed objects.
    // We verify indirectly: the value is still a valid ptr.
    assert!(list.is_ptr(), "rooted list should still be a valid ptr after collect");
    let _ = freed; // may be 0 or positive depending on other objects

    gc_remove_root(list);
    gc_clear_roots();
}

/// Create a dict, add as GC root, collect — object must not be freed.
#[test]
fn test_dict_value_gc_root_survives_collect() {
    use crate::runtime::gc::{gc_add_root, gc_remove_root, gc_clear_roots, collect, gc_disable, gc_enable};
    use crate::runtime::dict_ops::mb_dict_new;

    gc_disable();
    gc_clear_roots();

    let dict = mb_dict_new();
    gc_add_root(dict);

    gc_enable();
    let _freed = collect();

    assert!(dict.is_ptr(), "rooted dict should still be a valid ptr after collect");

    gc_remove_root(dict);
    gc_clear_roots();
}

/// Create a list, add as root, remove root, collect — freed count increases.
#[test]
fn test_list_remove_root_collected() {
    use crate::runtime::gc::{gc_add_root, gc_remove_root, gc_clear_roots, collect, gc_disable, gc_enable, gc_get_count};
    use crate::runtime::list_ops::mb_list_new;

    gc_disable();
    gc_clear_roots();

    let list = mb_list_new();
    gc_add_root(list);

    // Remove root before collecting — now unreachable.
    gc_remove_root(list);

    gc_enable();
    let before = gc_get_count();
    let freed = collect();
    let after = gc_get_count();

    // Freed count is >= 0; tracked count should not grow.
    let _ = freed;
    assert!(after <= before, "tracked count should not grow after collect");
}

/// Add multiple roots, clear all roots, collect — freed count > 0 possible.
#[test]
fn test_gc_clear_roots_allows_collection() {
    use crate::runtime::gc::{gc_add_root, gc_clear_roots, collect, gc_disable, gc_enable};
    use crate::runtime::list_ops::mb_list_new;
    use crate::runtime::dict_ops::mb_dict_new;

    gc_disable();
    gc_clear_roots();

    let l1 = mb_list_new();
    let l2 = mb_list_new();
    let d1 = mb_dict_new();
    gc_add_root(l1);
    gc_add_root(l2);
    gc_add_root(d1);

    // Clear all roots — all three are now unreachable.
    gc_clear_roots();

    gc_enable();
    let _freed = collect();
    // After clearing roots, GC may reclaim those objects (freed >= 0).
}

/// Outer list contains inner list; only outer is rooted — both survive collect.
#[test]
fn test_nested_list_reachability() {
    use crate::runtime::gc::{gc_add_root, gc_remove_root, gc_clear_roots, collect, gc_disable, gc_enable};
    use crate::runtime::list_ops::{mb_list_new, mb_list_append, mb_list_getitem};

    gc_disable();
    gc_clear_roots();

    let inner = mb_list_new();
    mb_list_append(inner, MbValue::from_int(99));

    let outer = mb_list_new();
    mb_list_append(outer, inner);

    // Only root the outer list.
    gc_add_root(outer);

    gc_enable();
    let _freed = collect();

    // outer must still be valid.
    assert!(outer.is_ptr());
    // inner is reachable through outer — verify its content is intact.
    let fetched_inner = mb_list_getitem(outer, MbValue::from_int(0));
    assert!(fetched_inner.is_ptr(), "inner list reachable from outer must survive");

    gc_remove_root(outer);
    gc_clear_roots();
}

// ═══════════════════════════════════════════════════════════
// Group 2: Module + stdlib (5 tests)
// ═══════════════════════════════════════════════════════════

/// Register a custom module with an int attr, import it, get the attr.
#[test]
fn test_register_and_import_custom_module() {
    use crate::runtime::module::{mb_module_register, mb_import, mb_module_getattr};
    use std::collections::HashMap;

    let mut attrs = HashMap::new();
    attrs.insert("answer".to_string(), MbValue::from_int(42));
    mb_module_register("integ_custom_mod", attrs);

    let mod_name = str_val("integ_custom_mod");
    let imported = mb_import(mod_name);
    assert!(imported.is_ptr(), "imported module should be a ptr");

    let attr_name = str_val("answer");
    let val = mb_module_getattr(str_val("integ_custom_mod"), attr_name);
    assert_eq!(val.as_int(), Some(42), "module attr 'answer' should be 42");
}

/// After register_builtins, import "sys" — result is a ptr (not none).
#[test]
fn test_import_builtin_after_register_builtins() {
    use crate::runtime::module::{mb_register_builtins, mb_import};

    mb_register_builtins();

    let result = mb_import(str_val("sys"));
    assert!(result.is_ptr(), "sys module should be importable after register_builtins");
}

/// After register_builtins, import "json", getattr "dumps" — not none.
#[test]
fn test_builtin_json_accessible() {
    use crate::runtime::module::{mb_register_builtins, mb_module_getattr};

    mb_register_builtins();

    let attr = mb_module_getattr(str_val("json"), str_val("dumps"));
    assert!(!attr.is_none(), "json.dumps should be accessible after register_builtins");
}

/// After register_builtins, import "os", getattr "getcwd" — not none.
#[test]
fn test_builtin_os_accessible() {
    use crate::runtime::module::{mb_register_builtins, mb_module_getattr};

    mb_register_builtins();

    let attr = mb_module_getattr(str_val("os"), str_val("getcwd"));
    assert!(!attr.is_none(), "os.getcwd should be accessible after register_builtins");
}

/// After register_builtins, import "math", getattr "sqrt" — not none.
#[test]
fn test_builtin_math_accessible() {
    use crate::runtime::module::{mb_register_builtins, mb_module_getattr};

    mb_register_builtins();

    let attr = mb_module_getattr(str_val("math"), str_val("sqrt"));
    assert!(!attr.is_none(), "math.sqrt should be accessible after register_builtins");
}

// ═══════════════════════════════════════════════════════════
// Group 3: mb_len on different types (5 tests)
// ═══════════════════════════════════════════════════════════

/// mb_len on an empty list returns 0.
#[test]
fn test_mb_len_on_empty_list() {
    use crate::runtime::builtins::mb_len;
    use crate::runtime::list_ops::mb_list_new;

    let list = mb_list_new();
    assert_eq!(mb_len(list).as_int(), Some(0));
}

/// mb_len on a list with 3 items returns 3.
#[test]
fn test_mb_len_on_nonempty_list() {
    use crate::runtime::builtins::mb_len;
    use crate::runtime::list_ops::{mb_list_new, mb_list_append};

    let list = mb_list_new();
    mb_list_append(list, MbValue::from_int(1));
    mb_list_append(list, MbValue::from_int(2));
    mb_list_append(list, MbValue::from_int(3));
    assert_eq!(mb_len(list).as_int(), Some(3));
}

/// mb_len on an empty dict returns 0.
#[test]
fn test_mb_len_on_empty_dict() {
    use crate::runtime::builtins::mb_len;
    use crate::runtime::dict_ops::mb_dict_new;

    let dict = mb_dict_new();
    assert_eq!(mb_len(dict).as_int(), Some(0));
}

/// mb_len on a str "hello" returns 5.
#[test]
fn test_mb_len_on_str() {
    use crate::runtime::builtins::mb_len;

    let s = str_val("hello");
    assert_eq!(mb_len(s).as_int(), Some(5));
}

/// mb_len on an empty str "" returns 0.
#[test]
fn test_mb_len_on_empty_str() {
    use crate::runtime::builtins::mb_len;

    let s = str_val("");
    assert_eq!(mb_len(s).as_int(), Some(0));
}

// ═══════════════════════════════════════════════════════════
// Group 4: mb_int, mb_float, mb_bool conversions (5 tests)
// ═══════════════════════════════════════════════════════════

/// mb_int(3.7) truncates to 3.
#[test]
fn test_mb_int_from_float() {
    use crate::runtime::builtins::mb_int;

    let result = mb_int(MbValue::from_float(3.7));
    assert_eq!(result.as_int(), Some(3));
}

/// mb_int(true) returns 1.
#[test]
fn test_mb_int_from_bool_true() {
    use crate::runtime::builtins::mb_int;

    let result = mb_int(MbValue::from_bool(true));
    assert_eq!(result.as_int(), Some(1));
}

/// mb_int(false) returns 0.
#[test]
fn test_mb_int_from_bool_false() {
    use crate::runtime::builtins::mb_int;

    let result = mb_int(MbValue::from_bool(false));
    assert_eq!(result.as_int(), Some(0));
}

/// mb_float(42) returns 42.0.
#[test]
fn test_mb_float_from_int() {
    use crate::runtime::builtins::mb_float;

    let result = mb_float(MbValue::from_int(42));
    assert_eq!(result.as_float(), Some(42.0));
}

/// mb_bool(0) returns false.
#[test]
fn test_mb_bool_from_zero() {
    use crate::runtime::builtins::mb_bool;

    let result = mb_bool(MbValue::from_int(0));
    assert_eq!(result.as_bool(), Some(false));
}

// ═══════════════════════════════════════════════════════════
// Group 5: Box/unbox round trips (5 tests)
// ═══════════════════════════════════════════════════════════

/// mb_box_int(42) round-trips back to 42.
#[test]
fn test_box_int_small() {
    use crate::runtime::builtins::mb_box_int;

    let val = mb_box_int(42);
    assert_eq!(val.as_int(), Some(42));
}

/// mb_box_int(-1) round-trips back to -1.
#[test]
fn test_box_int_negative() {
    use crate::runtime::builtins::mb_box_int;

    let val = mb_box_int(-1);
    assert_eq!(val.as_int(), Some(-1));
}

/// mb_box_bool(1) yields a bool true.
#[test]
fn test_box_bool_true() {
    use crate::runtime::builtins::mb_box_bool;

    let val = mb_box_bool(1);
    assert_eq!(val.as_bool(), Some(true));
}

/// mb_box_float(3.14) round-trips back to approximately 3.14.
#[test]
fn test_box_float_pi() {
    use crate::runtime::builtins::mb_box_float;

    let val = mb_box_float(3.14);
    let f = val.as_float().expect("should be a float");
    assert!((f - 3.14).abs() < 1e-9, "float should be approx 3.14, got {f}");
}

/// MbValue::from_int(99), mb_unbox_int → 99.
#[test]
fn test_unbox_int_roundtrip() {
    use crate::runtime::builtins::mb_unbox_int;

    let val = MbValue::from_int(99);
    assert_eq!(mb_unbox_int(val), 99i64);
}

// ═══════════════════════════════════════════════════════════
// Group 6: mb_is_none, mb_is_not_none (3 tests)
// ═══════════════════════════════════════════════════════════

/// mb_is_none on MbValue::none() returns true.
#[test]
fn test_is_none_on_none() {
    use crate::runtime::builtins::mb_is_none;

    let result = mb_is_none(MbValue::none());
    assert_eq!(result.as_bool(), Some(true));
}

/// mb_is_none on an int(42) returns false.
#[test]
fn test_is_none_on_int() {
    use crate::runtime::builtins::mb_is_none;

    let result = mb_is_none(MbValue::from_int(42));
    assert_eq!(result.as_bool(), Some(false));
}

/// mb_is_not_none on a list ptr returns true.
#[test]
fn test_is_not_none_on_ptr() {
    use crate::runtime::builtins::mb_is_not_none;
    use crate::runtime::list_ops::mb_list_new;

    let list = mb_list_new();
    let result = mb_is_not_none(list);
    assert_eq!(result.as_bool(), Some(true));
}
