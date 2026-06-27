#![cfg(test)]

use crate::runtime::rc::MbObject;
/// Integration tests for the Mamba runtime modules.
/// Tests string_ops, list_ops, dict_ops, tuple_ops, exception, class, iter,
/// generator, closure, module, and async_rt.
use crate::runtime::value::MbValue;

// ── String Operations (#284) ──

#[test]
fn test_string_concat() {
    use crate::runtime::string_ops::*;
    let a = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
    let b = MbValue::from_ptr(MbObject::new_str(" world".to_string()));
    let result = mb_str_concat(a, b);
    assert!(result.is_ptr());
}

#[test]
fn test_string_upper_lower() {
    use crate::runtime::string_ops::*;
    let s = MbValue::from_ptr(MbObject::new_str("Hello".to_string()));
    let upper = mb_str_upper(s);
    let lower = mb_str_lower(s);
    assert!(upper.is_ptr());
    assert!(lower.is_ptr());
}

#[test]
fn test_string_split_join() {
    use crate::runtime::string_ops::*;
    let s = MbValue::from_ptr(MbObject::new_str("a,b,c".to_string()));
    let sep = MbValue::from_ptr(MbObject::new_str(",".to_string()));
    let parts = mb_str_split(s, sep, MbValue::none());
    assert!(parts.is_ptr());
}

#[test]
fn test_string_replace() {
    use crate::runtime::string_ops::*;
    let s = MbValue::from_ptr(MbObject::new_str("hello world".to_string()));
    let old = MbValue::from_ptr(MbObject::new_str("world".to_string()));
    let new = MbValue::from_ptr(MbObject::new_str("rust".to_string()));
    let result = mb_str_replace(s, old, new, MbValue::none());
    assert!(result.is_ptr());
}

// ── List Operations (#285) ──

#[test]
fn test_list_operations() {
    use crate::runtime::list_ops::*;
    let list = mb_list_new();
    mb_list_append(list, MbValue::from_int(1));
    mb_list_append(list, MbValue::from_int(2));
    mb_list_append(list, MbValue::from_int(3));
    assert_eq!(mb_list_len(list).as_int(), Some(3));
    assert_eq!(
        mb_list_getitem(list, MbValue::from_int(0)).as_int(),
        Some(1)
    );
    assert_eq!(
        mb_list_getitem(list, MbValue::from_int(2)).as_int(),
        Some(3)
    );
}

#[test]
fn test_list_pop_and_remove() {
    use crate::runtime::list_ops::*;
    let list = mb_list_new();
    mb_list_append(list, MbValue::from_int(10));
    mb_list_append(list, MbValue::from_int(20));
    mb_list_append(list, MbValue::from_int(30));
    let popped = mb_list_pop(list);
    assert_eq!(popped.as_int(), Some(30));
    assert_eq!(mb_list_len(list).as_int(), Some(2));
}

#[test]
fn test_list_contains() {
    use crate::runtime::list_ops::*;
    let list = mb_list_new();
    mb_list_append(list, MbValue::from_int(42));
    assert_eq!(
        mb_list_contains(list, MbValue::from_int(42)).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_list_contains(list, MbValue::from_int(99)).as_bool(),
        Some(false)
    );
}

#[test]
fn test_list_sort() {
    use crate::runtime::list_ops::*;
    let list = mb_list_new();
    mb_list_append(list, MbValue::from_int(3));
    mb_list_append(list, MbValue::from_int(1));
    mb_list_append(list, MbValue::from_int(2));
    mb_list_sort(list);
    assert_eq!(
        mb_list_getitem(list, MbValue::from_int(0)).as_int(),
        Some(1)
    );
    assert_eq!(
        mb_list_getitem(list, MbValue::from_int(1)).as_int(),
        Some(2)
    );
    assert_eq!(
        mb_list_getitem(list, MbValue::from_int(2)).as_int(),
        Some(3)
    );
}

// ── Dict Operations (#285) ──

#[test]
fn test_dict_operations() {
    use crate::runtime::dict_ops::*;
    let d = mb_dict_new();
    let key = MbValue::from_ptr(MbObject::new_str("name".to_string()));
    let val = MbValue::from_ptr(MbObject::new_str("mamba".to_string()));
    mb_dict_setitem(d, key, val);
    assert_eq!(mb_dict_len(d).as_int(), Some(1));
    let key2 = MbValue::from_ptr(MbObject::new_str("name".to_string()));
    assert_eq!(mb_dict_contains(d, key2).as_bool(), Some(true));
}

#[test]
fn test_dict_pop() {
    use crate::runtime::dict_ops::*;
    let d = mb_dict_new();
    let key = MbValue::from_ptr(MbObject::new_str("x".to_string()));
    mb_dict_setitem(d, key, MbValue::from_int(42));
    let key2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
    let result = mb_dict_pop(d, key2, MbValue::from_int(-1));
    assert_eq!(result.as_int(), Some(42));
    assert_eq!(mb_dict_len(d).as_int(), Some(0));
}

// ── Tuple Operations (#285) ──

#[test]
fn test_tuple_operations() {
    use crate::runtime::tuple_ops::*;
    let t = mb_tuple_from(vec![
        MbValue::from_int(1),
        MbValue::from_int(2),
        MbValue::from_int(3),
    ]);
    assert_eq!(mb_tuple_len(t).as_int(), Some(3));
    assert_eq!(mb_tuple_getitem(t, MbValue::from_int(0)).as_int(), Some(1));
    assert_eq!(
        mb_tuple_contains(t, MbValue::from_int(2)).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_tuple_contains(t, MbValue::from_int(9)).as_bool(),
        Some(false)
    );
}

// ── Exception System (#283) ──

#[test]
fn test_exception_create_and_raise() {
    use crate::runtime::exception::*;
    mb_clear_exception();
    assert_eq!(mb_has_exception().as_bool(), Some(false));

    let exc_type = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
    let msg = MbValue::from_ptr(MbObject::new_str("bad value".to_string()));
    let _exc = mb_exception_new(exc_type, msg);
    mb_raise(exc_type, msg);
    assert_eq!(mb_has_exception().as_bool(), Some(true));

    let caught = mb_catch_exception();
    assert!(caught.is_ptr());
    mb_clear_exception();
    assert_eq!(mb_has_exception().as_bool(), Some(false));
}

#[test]
fn test_exception_matches() {
    use crate::runtime::exception::*;
    mb_clear_exception();

    let ve_type = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
    let ve_msg = MbValue::from_ptr(MbObject::new_str("test".to_string()));
    mb_raise(ve_type, ve_msg);

    let caught = mb_catch_exception();
    let target = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
    assert_eq!(mb_exception_matches(caught, target).as_bool(), Some(true));

    // Re-raise and check wrong type
    let ve_type2 = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
    let ve_msg2 = MbValue::from_ptr(MbObject::new_str("test2".to_string()));
    mb_raise(ve_type2, ve_msg2);
    let caught2 = mb_catch_exception();
    let wrong = MbValue::from_ptr(MbObject::new_str("TypeError".to_string()));
    assert_eq!(mb_exception_matches(caught2, wrong).as_bool(), Some(false));

    mb_clear_exception();
}

// ── Class System (#287, #288) ──

#[test]
fn test_class_instance_and_attrs() {
    use crate::runtime::class::*;
    use std::collections::HashMap;

    mb_class_register("TestAnimal", vec![], HashMap::new());
    mb_class_register("TestDog", vec!["TestAnimal".to_string()], HashMap::new());

    let name = MbValue::from_ptr(MbObject::new_str("TestDog".to_string()));
    let inst = mb_instance_new(name, MbValue::none());
    assert!(inst.is_ptr());

    // Set and get attribute
    let attr = MbValue::from_ptr(MbObject::new_str("breed".to_string()));
    let val = MbValue::from_ptr(MbObject::new_str("labrador".to_string()));
    mb_setattr(inst, attr, val);

    let attr2 = MbValue::from_ptr(MbObject::new_str("breed".to_string()));
    let result = mb_getattr(inst, attr2);
    assert!(result.is_ptr());
}

#[test]
fn test_isinstance_with_inheritance() {
    use crate::runtime::class::*;
    use std::collections::HashMap;

    mb_class_register("Base2", vec![], HashMap::new());
    mb_class_register("Child2", vec!["Base2".to_string()], HashMap::new());

    let child_name = MbValue::from_ptr(MbObject::new_str("Child2".to_string()));
    let inst = mb_instance_new(child_name, MbValue::none());

    let base = MbValue::from_ptr(MbObject::new_str("Base2".to_string()));
    let child = MbValue::from_ptr(MbObject::new_str("Child2".to_string()));
    let other = MbValue::from_ptr(MbObject::new_str("Other".to_string()));

    assert_eq!(mb_isinstance(inst, child).as_bool(), Some(true));
    assert_eq!(mb_isinstance(inst, base).as_bool(), Some(true));
    assert_eq!(mb_isinstance(inst, other).as_bool(), Some(false));
}

// ── Iterator Protocol (#286) ──

#[test]
fn test_iterator_protocol() {
    use crate::runtime::iter::*;
    let list = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(10),
        MbValue::from_int(20),
        MbValue::from_int(30),
    ]));
    let iter = mb_iter(list);
    assert_eq!(mb_next(iter).as_int(), Some(10));
    assert_eq!(mb_next(iter).as_int(), Some(20));
    assert_eq!(mb_next(iter).as_int(), Some(30));
    // After consuming all 3 items, the next call triggers exhaustion
    assert!(mb_next(iter).is_none());
    assert_eq!(mb_has_next(iter).as_bool(), Some(false));
    mb_iter_release(iter);
}

#[test]
fn test_range_iterator() {
    use crate::runtime::iter::*;
    let iter = mb_range_iter(
        MbValue::from_int(0),
        MbValue::from_int(5),
        MbValue::from_int(2),
    );
    assert_eq!(mb_next(iter).as_int(), Some(0));
    assert_eq!(mb_next(iter).as_int(), Some(2));
    assert_eq!(mb_next(iter).as_int(), Some(4));
    // After consuming all items, the next call triggers exhaustion
    assert!(mb_next(iter).is_none());
    assert_eq!(mb_has_next(iter).as_bool(), Some(false));
    mb_iter_release(iter);
}

// ── Generator (#290) ──

#[test]
fn test_generator_lifecycle() {
    use crate::runtime::generator::*;
    let name = MbValue::from_ptr(MbObject::new_str("test_gen".to_string()));
    // Create a generator with a dummy body function (we won't start it)
    let body_fn = MbValue::none();
    let gen = mb_generator_create(name, body_fn, MbValue::none());

    assert_eq!(mb_generator_is_exhausted(gen).as_bool(), Some(false));

    // Store an argument
    mb_generator_store_arg(gen, MbValue::from_int(42));

    // Close the generator (since body_fn is None, close just marks exhausted)
    mb_generator_close(gen);
    assert_eq!(mb_generator_is_exhausted(gen).as_bool(), Some(true));
    mb_generator_release(gen);
}

// ── Closure (#289) ──

#[test]
fn test_closure_captures() {
    use crate::runtime::closure::*;
    let name = MbValue::from_ptr(MbObject::new_str("test_closure".to_string()));
    let func = MbValue::from_int(0); // placeholder
    let captures = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(10),
        MbValue::from_int(20),
    ]));
    let closure = mb_closure_new(name, func, captures);

    assert_eq!(
        mb_closure_get_capture(closure, MbValue::from_int(0)).as_int(),
        Some(10)
    );
    assert_eq!(
        mb_closure_get_capture(closure, MbValue::from_int(1)).as_int(),
        Some(20)
    );

    mb_closure_set_capture(closure, MbValue::from_int(0), MbValue::from_int(99));
    assert_eq!(
        mb_closure_get_capture(closure, MbValue::from_int(0)).as_int(),
        Some(99)
    );

    mb_closure_release(closure);
}

// ── Module Import (#292) ──

#[test]
fn test_module_register_and_import() {
    use crate::runtime::module::*;
    use std::collections::HashMap;

    let mut attrs = HashMap::new();
    attrs.insert("test_val".to_string(), MbValue::from_int(99));
    mb_module_register("test_module_rt", attrs);

    let mod_name = MbValue::from_ptr(MbObject::new_str("test_module_rt".to_string()));
    let result = mb_import(mod_name);
    assert!(result.is_ptr());
}

#[test]
fn test_module_getattr() {
    use crate::runtime::module::*;
    use std::collections::HashMap;

    let mut attrs = HashMap::new();
    attrs.insert("pi".to_string(), MbValue::from_float(3.14));
    mb_module_register("mymath_rt", attrs);

    let mod_name = MbValue::from_ptr(MbObject::new_str("mymath_rt".to_string()));
    let attr = MbValue::from_ptr(MbObject::new_str("pi".to_string()));
    let val = mb_module_getattr(mod_name, attr);
    assert_eq!(val.as_float(), Some(3.14));
}

// ── Async Runtime (#293) ──

#[test]
fn test_coroutine_lifecycle() {
    use crate::runtime::async_rt::*;
    let name = MbValue::from_ptr(MbObject::new_str("test_coro".to_string()));
    let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
    let coro = mb_coroutine_new(name, locals);

    mb_coroutine_set_state(coro, 1);
    assert_eq!(mb_coroutine_get_state(coro), 1);

    mb_coroutine_complete(coro, MbValue::from_int(42));
    mb_coroutine_release(coro);
}

// ── Primitive isinstance (#288) ──

#[test]
fn test_isinstance_primitives() {
    use crate::runtime::class::*;
    let int_type = MbValue::from_ptr(MbObject::new_str("int".to_string()));
    let float_type = MbValue::from_ptr(MbObject::new_str("float".to_string()));
    let bool_type = MbValue::from_ptr(MbObject::new_str("bool".to_string()));

    assert_eq!(
        mb_isinstance(MbValue::from_int(42), int_type).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_isinstance(MbValue::from_float(3.14), float_type).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_isinstance(MbValue::from_bool(true), bool_type).as_bool(),
        Some(true)
    );
}

// ═══════════════════════════════════════════════════════════════
// P0 Runtime Tests (#375-#381)
// ═══════════════════════════════════════════════════════════════

fn s(val: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(val.to_string()))
}

unsafe fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| {
        if let crate::runtime::rc::ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

unsafe fn extract_list(val: MbValue) -> Vec<MbValue> {
    val.as_ptr()
        .map(|ptr| {
            if let crate::runtime::rc::ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                vec![]
            }
        })
        .unwrap_or_default()
}

// ── mb_call_method dispatch (#380) ──

#[test]
fn test_call_method_str_upper() {
    use crate::runtime::class::mb_call_method;
    let result = mb_call_method(s("hello"), s("upper"), MbValue::none());
    unsafe {
        assert_eq!(extract_str(result), Some("HELLO".to_string()));
    }
}

#[test]
fn test_call_method_str_split() {
    use crate::runtime::class::mb_call_method;
    let args = MbValue::from_ptr(MbObject::new_list(vec![s(",")]));
    let result = mb_call_method(s("a,b,c"), s("split"), args);
    unsafe {
        let items = extract_list(result);
        assert_eq!(items.len(), 3);
        assert_eq!(extract_str(items[0]), Some("a".to_string()));
    }
}

#[test]
fn test_call_method_list_append() {
    use crate::runtime::class::mb_call_method;
    use crate::runtime::list_ops::{mb_list_len, mb_list_new};
    let list = mb_list_new();
    let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(42)]));
    mb_call_method(list, s("append"), args);
    assert_eq!(mb_list_len(list).as_int(), Some(1));
}

#[test]
fn test_call_method_dict_keys() {
    use crate::runtime::class::mb_call_method;
    use crate::runtime::dict_ops::{mb_dict_new, mb_dict_setitem};
    let d = mb_dict_new();
    mb_dict_setitem(d, s("a"), MbValue::from_int(1));
    let result = mb_call_method(d, s("keys"), MbValue::none());
    unsafe {
        let items = extract_list(result);
        assert_eq!(items.len(), 1);
    }
}

#[test]
fn test_call_method_primitive_raises_attr_error() {
    use crate::runtime::class::mb_call_method;
    use crate::runtime::exception;
    exception::mb_clear_exception();
    let result = mb_call_method(MbValue::from_int(42), s("upper"), MbValue::none());
    assert!(result.is_none());
    assert_eq!(exception::mb_has_exception().as_bool(), Some(true));
    exception::mb_clear_exception();
}

// ── Builtins (#378) ──

#[test]
fn test_builtin_min_max() {
    use crate::runtime::builtins::{mb_max, mb_min};
    let list = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(3),
        MbValue::from_int(1),
        MbValue::from_int(2),
    ]));
    assert_eq!(mb_min(list).as_int(), Some(1));
    assert_eq!(mb_max(list).as_int(), Some(3));
}

#[test]
fn test_builtin_sum() {
    use crate::runtime::builtins::mb_sum;
    let list = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(1),
        MbValue::from_int(2),
        MbValue::from_int(3),
    ]));
    assert_eq!(mb_sum(list).as_int(), Some(6));
}

#[test]
fn test_builtin_sorted() {
    use crate::runtime::builtins::mb_sorted;
    let list = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(3),
        MbValue::from_int(1),
        MbValue::from_int(2),
    ]));
    unsafe {
        let items = extract_list(mb_sorted(list, MbValue::none()));
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[1].as_int(), Some(2));
        assert_eq!(items[2].as_int(), Some(3));
    }
}

#[test]
fn test_builtin_repr() {
    use crate::runtime::builtins::mb_repr;
    unsafe {
        assert_eq!(
            extract_str(mb_repr(MbValue::from_int(42))),
            Some("42".to_string())
        );
        assert_eq!(
            extract_str(mb_repr(s("hello"))),
            Some("'hello'".to_string())
        );
    }
}

#[test]
fn test_builtin_chr_ord() {
    use crate::runtime::builtins::{mb_chr, mb_ord};
    unsafe {
        assert_eq!(
            extract_str(mb_chr(MbValue::from_int(65))),
            Some("A".to_string())
        );
    }
    assert_eq!(mb_ord(s("A")).as_int(), Some(65));
}

#[test]
fn test_builtin_hex_oct_bin() {
    use crate::runtime::builtins::{mb_bin, mb_hex, mb_oct};
    unsafe {
        assert_eq!(
            extract_str(mb_hex(MbValue::from_int(255))),
            Some("0xff".to_string())
        );
        assert_eq!(
            extract_str(mb_oct(MbValue::from_int(8))),
            Some("0o10".to_string())
        );
        assert_eq!(
            extract_str(mb_bin(MbValue::from_int(10))),
            Some("0b1010".to_string())
        );
    }
}

#[test]
fn test_builtin_pow() {
    use crate::runtime::builtins::mb_pow;
    assert_eq!(
        mb_pow(MbValue::from_int(2), MbValue::from_int(10)).as_int(),
        Some(1024)
    );
}

#[test]
fn test_builtin_floordiv() {
    use crate::runtime::builtins::mb_floordiv;
    assert_eq!(
        mb_floordiv(MbValue::from_int(7), MbValue::from_int(2)).as_int(),
        Some(3)
    );
    assert_eq!(
        mb_floordiv(MbValue::from_int(-7), MbValue::from_int(2)).as_int(),
        Some(-4)
    );
}

#[test]
fn test_builtin_comparisons() {
    use crate::runtime::builtins::{mb_ge, mb_gt, mb_le, mb_ne};
    assert_eq!(
        mb_gt(MbValue::from_int(5), MbValue::from_int(3)).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_le(MbValue::from_int(3), MbValue::from_int(5)).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_le(MbValue::from_int(5), MbValue::from_int(5)).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_ge(MbValue::from_int(5), MbValue::from_int(5)).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_ne(MbValue::from_int(1), MbValue::from_int(2)).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_ne(MbValue::from_int(1), MbValue::from_int(1)).as_bool(),
        Some(false)
    );
}

// ── Exception hierarchy (#381) ──

#[test]
fn test_exception_hierarchy_matching() {
    use crate::runtime::exception::mb_exception_matches;
    let exc = crate::runtime::exception::mb_exception_new(s("IndexError"), s("out of range"));
    assert_eq!(
        mb_exception_matches(exc, s("IndexError")).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_exception_matches(exc, s("LookupError")).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_exception_matches(exc, s("Exception")).as_bool(),
        Some(true)
    );
    assert_eq!(
        mb_exception_matches(exc, s("ValueError")).as_bool(),
        Some(false)
    );
}

#[test]
fn test_exception_zero_division() {
    use crate::runtime::exception;
    let exc = exception::mb_zero_division_error();
    assert_eq!(
        exception::mb_exception_matches(exc, s("ZeroDivisionError")).as_bool(),
        Some(true)
    );
    assert_eq!(
        exception::mb_exception_matches(exc, s("ArithmeticError")).as_bool(),
        Some(true)
    );
}

// ── Iterator builtins (#378 R1) ──

#[test]
fn test_enumerate_iterator() {
    use crate::runtime::iter::{mb_enumerate, mb_iter_release, mb_next};
    use crate::runtime::rc::ObjData;
    let list = MbValue::from_ptr(MbObject::new_list(vec![s("a"), s("b")]));
    let it = mb_enumerate(list, MbValue::from_int(0));
    let first = mb_next(it);
    unsafe {
        if let ObjData::Tuple(ref items) = (*first.as_ptr().unwrap()).data {
            assert_eq!(items[0].as_int(), Some(0));
        }
    }
    mb_iter_release(it);
}

#[test]
fn test_zip_iterator() {
    use crate::runtime::iter::{mb_iter_release, mb_next, mb_zip};
    use crate::runtime::rc::ObjData;
    let a = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(1),
        MbValue::from_int(2),
    ]));
    let b = MbValue::from_ptr(MbObject::new_list(vec![s("a"), s("b")]));
    let it = mb_zip(a, b);
    let first = mb_next(it);
    unsafe {
        if let ObjData::Tuple(ref items) = (*first.as_ptr().unwrap()).data {
            assert_eq!(items[0].as_int(), Some(1));
            assert_eq!(extract_str(items[1]), Some("a".to_string()));
        }
    }
    mb_iter_release(it);
}

#[test]
fn test_reversed_iterator() {
    use crate::runtime::iter::{mb_iter_release, mb_next, mb_reversed};
    let list = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(1),
        MbValue::from_int(2),
        MbValue::from_int(3),
    ]));
    let it = mb_reversed(list);
    assert_eq!(mb_next(it).as_int(), Some(3));
    assert_eq!(mb_next(it).as_int(), Some(2));
    assert_eq!(mb_next(it).as_int(), Some(1));
    assert!(mb_next(it).is_none());
    mb_iter_release(it);
}

// ── File I/O (#379) ──

#[test]
fn test_file_io_write_and_read() {
    use crate::runtime::file_io;
    let tmp = std::env::temp_dir().join("mamba_p0_test.txt");
    let path_str = tmp.to_string_lossy().to_string();

    // Write
    let fh = file_io::mb_open(s(&path_str), s("w"));
    assert!(fh.as_int().is_some());
    let written = file_io::mb_file_write(fh, s("hello\nworld\n"));
    assert_eq!(written.as_int(), Some(12));
    file_io::mb_file_close(fh);

    // Read
    let fh2 = file_io::mb_open(s(&path_str), s("r"));
    let content = file_io::mb_file_read(fh2);
    unsafe {
        assert_eq!(extract_str(content), Some("hello\nworld\n".to_string()));
    }
    file_io::mb_file_close(fh2);

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_file_io_not_found() {
    use crate::runtime::{exception, file_io};
    exception::mb_clear_exception();
    let result = file_io::mb_open(s("/nonexistent/path.txt"), s("r"));
    assert!(result.is_none());
    assert_eq!(exception::mb_has_exception().as_bool(), Some(true));
    exception::mb_clear_exception();
}

#[test]
fn test_file_io_read_after_close() {
    use crate::runtime::{exception, file_io};
    let tmp = std::env::temp_dir().join("mamba_p0_close_test.txt");
    let path_str = tmp.to_string_lossy().to_string();

    let fh = file_io::mb_open(s(&path_str), s("w"));
    file_io::mb_file_write(fh, s("test"));
    file_io::mb_file_close(fh);

    let fh2 = file_io::mb_open(s(&path_str), s("r"));
    file_io::mb_file_close(fh2);

    exception::mb_clear_exception();
    let result = file_io::mb_file_read(fh2);
    assert!(result.is_none());

    let _ = std::fs::remove_file(&tmp);
}

// ── String dispatch method tests (#375) ──

#[test]
fn test_str_dispatch_find_replace() {
    use crate::runtime::string_ops::dispatch_str_method;
    let args_find = MbValue::from_ptr(MbObject::new_list(vec![s("world")]));
    assert_eq!(
        dispatch_str_method("find", s("hello world"), args_find).as_int(),
        Some(6)
    );

    let args_replace = MbValue::from_ptr(MbObject::new_list(vec![s("world"), s("rust")]));
    let result = dispatch_str_method("replace", s("hello world"), args_replace);
    unsafe {
        assert_eq!(extract_str(result), Some("hello rust".to_string()));
    }
}

#[test]
fn test_str_dispatch_predicates() {
    use crate::runtime::string_ops::dispatch_str_method;
    assert_eq!(
        dispatch_str_method("isdigit", s("123"), MbValue::none()).as_bool(),
        Some(true)
    );
    assert_eq!(
        dispatch_str_method("isalpha", s("abc"), MbValue::none()).as_bool(),
        Some(true)
    );
    assert_eq!(
        dispatch_str_method("isupper", s("ABC"), MbValue::none()).as_bool(),
        Some(true)
    );
    assert_eq!(
        dispatch_str_method("islower", s("abc"), MbValue::none()).as_bool(),
        Some(true)
    );
}

// ── List dispatch method tests (#376) ──

#[test]
fn test_list_dispatch_sort_and_reverse() {
    use crate::runtime::list_ops::*;
    let list = mb_list_from(vec![
        MbValue::from_int(3),
        MbValue::from_int(1),
        MbValue::from_int(2),
    ]);
    dispatch_list_method("sort", list, MbValue::none());
    assert_eq!(
        mb_list_getitem(list, MbValue::from_int(0)).as_int(),
        Some(1)
    );
    dispatch_list_method("reverse", list, MbValue::none());
    assert_eq!(
        mb_list_getitem(list, MbValue::from_int(0)).as_int(),
        Some(3)
    );
}

// ── Dict dispatch method tests (#377) ──

#[test]
fn test_dict_dispatch_setdefault() {
    use crate::runtime::dict_ops::*;
    let d = mb_dict_new();
    let args = MbValue::from_ptr(MbObject::new_list(vec![s("x"), MbValue::from_int(42)]));
    let result = dispatch_dict_method("setdefault", d, args);
    assert_eq!(result.as_int(), Some(42));
    // Second call should return existing value
    let args2 = MbValue::from_ptr(MbObject::new_list(vec![s("x"), MbValue::from_int(99)]));
    let result2 = dispatch_dict_method("setdefault", d, args2);
    assert_eq!(result2.as_int(), Some(42));
}

// ── String concatenation: content verification (string-ops) ──

#[test]
fn test_string_concat_content() {
    // mb_str_concat must produce the correct concatenated string
    use crate::runtime::string_ops::mb_str_concat;
    let result = mb_str_concat(s("hello"), s(" world"));
    assert_eq!(
        unsafe { extract_str(result) },
        Some("hello world".to_string())
    );
}

#[test]
fn test_string_concat_empty_left() {
    // "" + "world" == "world"
    use crate::runtime::string_ops::mb_str_concat;
    let result = mb_str_concat(s(""), s("world"));
    assert_eq!(unsafe { extract_str(result) }, Some("world".to_string()));
}

#[test]
fn test_string_concat_empty_right() {
    // "hello" + "" == "hello"
    use crate::runtime::string_ops::mb_str_concat;
    let result = mb_str_concat(s("hello"), s(""));
    assert_eq!(unsafe { extract_str(result) }, Some("hello".to_string()));
}

#[test]
fn test_string_concat_both_empty() {
    // "" + "" == ""
    use crate::runtime::string_ops::mb_str_concat;
    let result = mb_str_concat(s(""), s(""));
    assert_eq!(unsafe { extract_str(result) }, Some(String::new()));
}
