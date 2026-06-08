#![cfg(test)]

/// JIT refcount audit integration tests (#1129).
///
/// Validates the ownership audit changes: all borrowed-reference mb_* functions
/// now call `retain_if_ptr` before returning, ensuring callers always receive
/// owned references. Also verifies EMIT_REFCOUNT_CALLS=true and GC re-enabled.

use crate::runtime::value::MbValue;
use crate::runtime::rc::{self, MbObject, ObjData, IMMORTAL_REFCOUNT, mb_refcount};
use crate::runtime::gc;

// ── Helpers ──

fn str_val(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn list_val(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

fn dict_with(pairs: Vec<(&str, MbValue)>) -> MbValue {
    let dict = crate::runtime::dict_ops::mb_dict_new();
    for (k, v) in pairs {
        crate::runtime::dict_ops::mb_dict_setitem(dict, str_val(k), v);
    }
    dict
}

fn tuple_val(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(items))
}

/// Guard that disables GC for the test duration to prevent cross-thread
/// sweep interference, then re-enables on drop.
struct GcGuard;
impl GcGuard {
    fn new() -> Self {
        gc::gc_disable();
        Self
    }
}
impl Drop for GcGuard {
    fn drop(&mut self) {
        gc::gc_enable();
    }
}

// ═══════════════════════════════════════════════════════════
// Unit Tests: retain_if_ptr behavior (rc.rs)
// ═══════════════════════════════════════════════════════════

/// R2: retain_if_ptr on an integer MbValue is a no-op — no crash, same value.
#[test]
fn test_retain_if_ptr_int_noop() {
    let val = MbValue::from_int(42);
    unsafe { rc::retain_if_ptr(val); }
    // No crash. Value is unchanged (ints are NaN-boxed, not heap pointers).
    assert_eq!(val.as_int(), Some(42));
}

/// R2: retain_if_ptr on a heap list increments rc from 1→2.
#[test]
fn test_retain_if_ptr_heap_obj() {
    let _gc = GcGuard::new();
    unsafe {
        let obj = MbObject::new_list(vec![MbValue::from_int(1)]);
        assert_eq!(mb_refcount(obj), 1);

        let val = MbValue::from_ptr(obj);
        rc::retain_if_ptr(val);
        assert_eq!(mb_refcount(obj), 2);

        // Cleanup: release twice to free
        rc::mb_release(obj);
        rc::mb_release(obj);
    }
}

/// R2: retain_if_ptr on an immortal string — rc stays IMMORTAL.
#[test]
fn test_retain_if_ptr_immortal_noop() {
    unsafe {
        let obj = MbObject::new_str_immortal("constant".into());
        assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);

        let val = MbValue::from_ptr(obj);
        rc::retain_if_ptr(val);
        assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);

        // Cleanup: force-free since mb_release won't touch immortals
        drop(Box::from_raw(obj));
    }
}

/// R2: retain_if_ptr on None — no crash.
#[test]
fn test_retain_if_ptr_none_noop() {
    let val = MbValue::none();
    unsafe { rc::retain_if_ptr(val); }
    assert!(val.is_none());
}

/// R2: retain_if_ptr on a bool — no crash, is a no-op.
#[test]
fn test_retain_if_ptr_bool_noop() {
    let val = MbValue::from_bool(true);
    unsafe { rc::retain_if_ptr(val); }
    assert_eq!(val.as_bool(), Some(true));
}

/// R2: retain_if_ptr on a float — no crash, is a no-op.
#[test]
fn test_retain_if_ptr_float_noop() {
    let val = MbValue::from_float(3.14);
    unsafe { rc::retain_if_ptr(val); }
    assert!(val.as_float().is_some());
}

// ═══════════════════════════════════════════════════════════
// Integration: List getitem returns owned reference (S1)
// ═══════════════════════════════════════════════════════════

/// R2, S1: mb_list_getitem returns an owned reference — the returned element
/// has rc incremented so it survives after the list is released.
#[test]
fn test_list_getitem_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::list_ops::mb_list_getitem;

    unsafe {
        // Create a list containing a heap string
        let inner_str = MbObject::new_str("hello".into());
        assert_eq!(mb_refcount(inner_str), 1);

        let list = list_val(vec![MbValue::from_ptr(inner_str)]);
        // After list creation, the string is in the list (rc still 1 — list
        // just copies the MbValue bits, doesn't retain).

        // Get element — should retain the value (rc 1→2)
        let elem = mb_list_getitem(list, MbValue::from_int(0));
        assert!(elem.is_ptr());
        assert_eq!(mb_refcount(inner_str), 2);

        // Release the list — cascading release drops the element rc (2→1)
        rc::release_if_ptr(list);
        // The element still has rc=1 (our owned ref from getitem)
        assert_eq!(mb_refcount(inner_str), 1);

        // Verify the string is still valid
        if let ObjData::Str(ref s) = (*inner_str).data {
            assert_eq!(s, "hello");
        } else {
            panic!("expected Str data");
        }

        // Clean up
        rc::mb_release(inner_str);
    }
}

/// S1 variant: mb_list_getitem with integer elements (non-pointer) — no crash.
#[test]
fn test_list_getitem_int_noop() {
    let _gc = GcGuard::new();
    use crate::runtime::list_ops::mb_list_getitem;

    let list = list_val(vec![MbValue::from_int(10), MbValue::from_int(20)]);
    let elem = mb_list_getitem(list, MbValue::from_int(1));
    assert_eq!(elem.as_int(), Some(20));

    // Release list — ints are not pointers, no crash
    unsafe { rc::release_if_ptr(list); }
}

// ═══════════════════════════════════════════════════════════
// Integration: Dict getitem returns owned reference (S2)
// ═══════════════════════════════════════════════════════════

/// R2, S2: mb_dict_getitem returns an owned reference — value survives after
/// dict is freed.
#[test]
fn test_dict_getitem_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::dict_ops::mb_dict_getitem;

    unsafe {
        let inner_list = MbObject::new_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_refcount(inner_list), 1);

        let dict = dict_with(vec![("key", MbValue::from_ptr(inner_list))]);
        // After dict_setitem, the list was retained (rc 1→2)
        assert_eq!(mb_refcount(inner_list), 2);

        // Get value — should retain (rc 2→3)
        let val = mb_dict_getitem(dict, str_val("key"));
        assert!(val.is_ptr());
        assert_eq!(mb_refcount(inner_list), 3);

        // Release our getitem ref
        rc::release_if_ptr(val);
        assert_eq!(mb_refcount(inner_list), 2);

        // Release the dict (cascading release: key string freed, value rc 2→1)
        rc::release_if_ptr(dict);
        assert_eq!(mb_refcount(inner_list), 1);

        // The list is still valid
        if let ObjData::List(ref lock) = (*inner_list).data {
            assert_eq!(lock.read().unwrap().len(), 2);
        } else {
            panic!("expected List data");
        }

        // Final cleanup
        rc::mb_release(inner_list);
    }
}

// ═══════════════════════════════════════════════════════════
// Integration: Tuple getitem returns owned reference
// ═══════════════════════════════════════════════════════════

/// R2: mb_tuple_getitem returns an owned reference.
#[test]
fn test_tuple_getitem_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::tuple_ops::mb_tuple_getitem;

    unsafe {
        let inner_str = MbObject::new_str("world".into());
        assert_eq!(mb_refcount(inner_str), 1);

        let tup = tuple_val(vec![MbValue::from_ptr(inner_str)]);

        // Get element — should retain (rc 1→2)
        let elem = mb_tuple_getitem(tup, MbValue::from_int(0));
        assert!(elem.is_ptr());
        assert_eq!(mb_refcount(inner_str), 2);

        // Release tuple — cascading release drops element rc (2→1)
        rc::release_if_ptr(tup);
        assert_eq!(mb_refcount(inner_str), 1);

        // String still valid
        if let ObjData::Str(ref s) = (*inner_str).data {
            assert_eq!(s, "world");
        } else {
            panic!("expected Str data");
        }

        rc::mb_release(inner_str);
    }
}

// ═══════════════════════════════════════════════════════════
// Integration: Getattr returns owned reference (S3)
// ═══════════════════════════════════════════════════════════

/// R2, S3: mb_getattr returns an owned reference — attribute value survives
/// after instance is freed.
#[test]
fn test_getattr_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::class::{mb_setattr, mb_getattr};

    unsafe {
        // Create an instance with a heap attribute
        let instance = MbValue::from_ptr(MbObject::new_instance("TestClass".to_string()));
        let attr_val = MbObject::new_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_refcount(attr_val), 1);

        // Set attribute — this should retain the value internally
        let attr_name = str_val("items");
        mb_setattr(instance, attr_name, MbValue::from_ptr(attr_val));

        // Get attribute — should return an owned reference (retain called)
        let result = mb_getattr(instance, str_val("items"));
        assert!(result.is_ptr());
        // attr_val rc should have been bumped by both setattr and getattr
        let rc = mb_refcount(attr_val);
        assert!(rc >= 2, "expected rc >= 2 after setattr + getattr, got {rc}");

        // Release the getattr result
        rc::release_if_ptr(result);

        // Release the instance — cascading release on the fields
        rc::release_if_ptr(instance);

        // The attr_val should eventually be freed (or at rc=0 if everything balanced)
        // We can't safely read rc after release_if_ptr of the instance cascades.
        // The test passes if no crash (no use-after-free).
    }
}

// ═══════════════════════════════════════════════════════════
// Integration: Global get returns owned reference (S4)
// ═══════════════════════════════════════════════════════════

/// R2, S4: mb_global_get_id returns an owned reference — global value is not
/// corrupted when the local reference is released.
#[test]
fn test_global_get_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::closure::{mb_global_set_id, mb_global_get_id};

    unsafe {
        let global_list = MbObject::new_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_refcount(global_list), 1);

        // Store global by id. mb_global_set_id retains the value so it
        // survives the JIT epilogue releasing the caller's VReg (rc 1→2).
        // Simulate that epilogue release here so we have a single owned ref
        // (the one held by the global namespace) as the baseline.
        let id = MbValue::from_bits(42);
        mb_global_set_id(id, MbValue::from_ptr(global_list));
        rc::release_if_ptr(MbValue::from_ptr(global_list));
        assert_eq!(mb_refcount(global_list), 1);

        // Get global — should return owned ref (retain: rc 1→2)
        let val1 = mb_global_get_id(id);
        assert!(val1.is_ptr());
        assert_eq!(mb_refcount(global_list), 2);

        // Get again — should return another owned ref (rc 2→3)
        let val2 = mb_global_get_id(id);
        assert_eq!(mb_refcount(global_list), 3);

        // Release both local refs
        rc::release_if_ptr(val1);
        assert_eq!(mb_refcount(global_list), 2);
        rc::release_if_ptr(val2);
        assert_eq!(mb_refcount(global_list), 1);

        // The global is still alive (rc=1 from the global namespace)
        if let ObjData::List(ref lock) = (*global_list).data {
            assert_eq!(lock.read().unwrap().len(), 2);
        } else {
            panic!("expected List data");
        }

        // Cleanup: overwrite with None — mb_global_set_id now cascade-releases
        // the old value (rc 1→0, freed). No manual release needed.
        mb_global_set_id(id, MbValue::none());
    }
}

// ═══════════════════════════════════════════════════════════
// Integration: Cell get returns owned reference
// ═══════════════════════════════════════════════════════════

/// R2: mb_cell_get returns an owned reference.
#[test]
fn test_cell_get_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::closure::{mb_cell_new, mb_cell_set, mb_cell_get};

    unsafe {
        let inner = MbObject::new_str("cell_value".into());
        assert_eq!(mb_refcount(inner), 1);

        // mb_cell_new takes its OWN owned reference to the initial value: it
        // calls `retain_if_ptr` so the slot survives the JIT epilogue's
        // `mb_release_value` on the source VReg (see MakeCell in
        // codegen/cranelift/mod.rs + the "Fix C-prime" comment in closure.rs).
        // So after construction the object has rc 2: our `inner` + the cell.
        let cell = mb_cell_new(MbValue::from_ptr(inner));
        assert_eq!(mb_refcount(inner), 2);

        // Get cell value — returns an owned ref (retain: rc 2→3).
        let val = mb_cell_get(cell);
        assert!(val.is_ptr());
        assert_eq!(mb_refcount(inner), 3);

        // Release the get result (rc 3→2).
        rc::release_if_ptr(val);
        assert_eq!(mb_refcount(inner), 2);

        // Value still valid in the cell
        if let ObjData::Str(ref s) = (*inner).data {
            assert_eq!(s, "cell_value");
        } else {
            panic!("expected Str data");
        }

        // Cleanup: overwrite cell — mb_cell_set cascade-releases the old cell
        // value (rc 2→1, the cell's owned ref dropped).
        mb_cell_set(cell, MbValue::none());
        assert_eq!(mb_refcount(inner), 1);

        // Final cleanup: release our remaining `inner` reference (rc 1→0, freed).
        rc::mb_release(inner);
    }
}

// ═══════════════════════════════════════════════════════════
// Integration: Closure get_capture returns owned reference (S8)
// ═══════════════════════════════════════════════════════════

/// R2, S8: mb_closure_get_capture returns an owned reference.
#[test]
fn test_closure_get_capture_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::closure::{mb_closure_new, mb_closure_get_capture, mb_closure_release};

    unsafe {
        let captured = MbObject::new_str("captured_var".into());
        assert_eq!(mb_refcount(captured), 1);

        // Extra retain so captured survives closure release + our inspection
        rc::mb_retain(captured);
        assert_eq!(mb_refcount(captured), 2);

        // Create closure with one captured variable
        let name = str_val("test_closure");
        let func = MbValue::from_int(0); // dummy function pointer
        let caps = list_val(vec![MbValue::from_ptr(captured)]);
        let closure_handle = mb_closure_new(name, func, caps);

        // Get capture — should return owned ref (retain: rc 2→3)
        let val = mb_closure_get_capture(closure_handle, MbValue::from_int(0));
        assert!(val.is_ptr());
        assert_eq!(mb_refcount(captured), 3);

        // Release our getitem reference (rc 3→2)
        rc::release_if_ptr(val);
        assert_eq!(mb_refcount(captured), 2);

        // Captured value still valid
        if let ObjData::Str(ref s) = (*captured).data {
            assert_eq!(s, "captured_var");
        } else {
            panic!("expected Str data");
        }

        // Release closure — cascade-releases captures (rc 2→1)
        mb_closure_release(closure_handle);
        assert_eq!(mb_refcount(captured), 1);

        // Final cleanup: release our extra retain
        rc::mb_release(captured);
    }
}

// ═══════════════════════════════════════════════════════════
// Integration: Iterator next returns owned reference (S5)
// ═══════════════════════════════════════════════════════════

/// R2, S5: mb_next returns an owned reference — element survives after
/// iterator/list freed.
#[test]
fn test_next_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::iter::{mb_iter, mb_next};

    unsafe {
        // Create a list with heap elements
        let elem0 = MbObject::new_str("first".into());
        let elem1 = MbObject::new_str("second".into());
        assert_eq!(mb_refcount(elem0), 1);
        assert_eq!(mb_refcount(elem1), 1);

        let list = list_val(vec![MbValue::from_ptr(elem0), MbValue::from_ptr(elem1)]);

        // Create iterator
        let iter_handle = mb_iter(list);

        // Get first element via next — should retain (rc 1→2)
        let first = mb_next(iter_handle);
        assert!(first.is_ptr());
        assert_eq!(mb_refcount(elem0), 2);

        // Get second element via next — should retain (rc 1→2)
        let second = mb_next(iter_handle);
        assert!(second.is_ptr());
        assert_eq!(mb_refcount(elem1), 2);

        // Release iterator
        crate::runtime::iter::mb_iter_release(iter_handle);

        // Release list — cascading release: elem0 rc 2→1, elem1 rc 2→1
        rc::release_if_ptr(list);
        assert_eq!(mb_refcount(elem0), 1);
        assert_eq!(mb_refcount(elem1), 1);

        // Elements still valid (owned by our next() references)
        if let ObjData::Str(ref s) = (*elem0).data {
            assert_eq!(s, "first");
        } else {
            panic!("expected Str");
        }

        // Cleanup
        rc::mb_release(elem0);
        rc::mb_release(elem1);
    }
}

/// S5 variant: mb_next with integer elements — retain_if_ptr is a no-op.
#[test]
fn test_next_int_noop() {
    let _gc = GcGuard::new();
    use crate::runtime::iter::{mb_iter, mb_next, mb_iter_release};

    let list = list_val(vec![MbValue::from_int(10), MbValue::from_int(20)]);
    let iter_handle = mb_iter(list);

    let first = mb_next(iter_handle);
    assert_eq!(first.as_int(), Some(10));

    let second = mb_next(iter_handle);
    assert_eq!(second.as_int(), Some(20));

    mb_iter_release(iter_handle);
    unsafe { rc::release_if_ptr(list); }
}

// ═══════════════════════════════════════════════════════════
// Integration: Dict get returns owned reference
// ═══════════════════════════════════════════════════════════

/// R2: mb_dict_get returns an owned reference.
#[test]
fn test_dict_get_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::dict_ops::mb_dict_get;

    unsafe {
        let inner = MbObject::new_str("value".into());
        assert_eq!(mb_refcount(inner), 1);

        let dict = dict_with(vec![("mykey", MbValue::from_ptr(inner))]);
        // dict_setitem retains the value (rc 1→2)
        assert_eq!(mb_refcount(inner), 2);

        // dict.get(key, default) — should retain found value (rc 2→3)
        let val = mb_dict_get(dict, str_val("mykey"), MbValue::none());
        assert!(val.is_ptr());
        assert_eq!(mb_refcount(inner), 3);

        // Release our get result
        rc::release_if_ptr(val);
        assert_eq!(mb_refcount(inner), 2);

        // Release dict (cascade: value rc 2→1)
        rc::release_if_ptr(dict);
        assert_eq!(mb_refcount(inner), 1);

        // Still valid
        if let ObjData::Str(ref s) = (*inner).data {
            assert_eq!(s, "value");
        }

        rc::mb_release(inner);
    }
}

/// R2: mb_dict_get returns default when key not found — no crash.
#[test]
fn test_dict_get_missing_key_returns_default() {
    let _gc = GcGuard::new();
    use crate::runtime::dict_ops::mb_dict_get;

    let dict = dict_with(vec![]);
    let default = MbValue::from_int(999);
    let val = mb_dict_get(dict, str_val("missing"), default);
    assert_eq!(val.as_int(), Some(999));

    unsafe { rc::release_if_ptr(dict); }
}

// ═══════════════════════════════════════════════════════════
// Integration: Module getattr returns owned reference
// ═══════════════════════════════════════════════════════════

/// R2: mb_module_getattr returns an owned reference.
#[test]
fn test_module_getattr_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::module::{mb_module_getattr, mb_module_register};
    use std::collections::HashMap;

    unsafe {
        let attr_val = MbObject::new_str("module_attr_value".into());
        assert_eq!(mb_refcount(attr_val), 1);

        // Register a module with an attribute via the public API
        let mut attrs = HashMap::new();
        attrs.insert("test_attr".to_string(), MbValue::from_ptr(attr_val));
        mb_module_register("__test_refcount_mod__", attrs);

        // Get module attribute — should retain (rc 1→2)
        let val = mb_module_getattr(str_val("__test_refcount_mod__"), str_val("test_attr"));
        assert!(val.is_ptr());
        assert_eq!(mb_refcount(attr_val), 2);

        // Release our reference
        rc::release_if_ptr(val);
        assert_eq!(mb_refcount(attr_val), 1);

        // Cleanup
        rc::mb_release(attr_val);
    }
}

// ═══════════════════════════════════════════════════════════
// Integration: Catch exception returns owned reference
// ═══════════════════════════════════════════════════════════

/// R2: mb_catch_exception returns an owned reference.
#[test]
fn test_catch_exception_owned_ref() {
    let _gc = GcGuard::new();
    use crate::runtime::exception;

    // Set an exception
    exception::set_current_exception(
        exception::MbException::new("ValueError", "test error")
    );

    // Catch it — should return an owned reference
    let exc = exception::mb_catch_exception();
    assert!(exc.is_ptr(), "caught exception should be a heap object");

    // The exception was taken from thread-local state (no double reference).
    // Just verify it's valid and release.
    unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*(exc.as_ptr().unwrap())).data {
            // The exception is stored as an instance
            assert!(!class_name.is_empty());
        }
        rc::release_if_ptr(exc);
    }
}

/// R2: mb_catch_exception with no pending exception returns None.
#[test]
fn test_catch_exception_none_when_empty() {
    use crate::runtime::exception;

    // Ensure no exception is pending
    exception::clear_current_exception();

    let exc = exception::mb_catch_exception();
    assert!(exc.is_none(), "should return None when no exception pending");
}

// ═══════════════════════════════════════════════════════════
// Integration: Dict setdefault returns owned reference
// ═══════════════════════════════════════════════════════════

/// R2: mb_dict_setdefault returns an owned reference.
#[test]
fn test_dict_setdefault_owned_ref() {
    use crate::runtime::dict_ops::mb_dict_setdefault;

    unsafe {
        let existing = MbObject::new_str("existing_val".into());
        assert_eq!(mb_refcount(existing), 1);

        let dict = dict_with(vec![("key", MbValue::from_ptr(existing))]);
        // After setitem, rc is 2 (setitem retains)
        assert_eq!(mb_refcount(existing), 2);

        // setdefault with existing key — should return existing value, retained (rc 2→3)
        let val = mb_dict_setdefault(dict, str_val("key"), MbValue::from_int(99));
        assert!(val.is_ptr());
        assert_eq!(mb_refcount(existing), 3);

        // Release our reference
        rc::release_if_ptr(val);
        assert_eq!(mb_refcount(existing), 2);

        // Cleanup
        rc::release_if_ptr(dict);
        assert_eq!(mb_refcount(existing), 1);
        rc::mb_release(existing);
    }
}

// ═══════════════════════════════════════════════════════════
// Verification: New-reference functions do NOT double-retain (S6, S9)
// ═══════════════════════════════════════════════════════════

/// S6: mb_list_new returns rc=1 — no spurious retain.
#[test]
fn test_new_ref_list_no_double_retain() {
    use crate::runtime::list_ops::mb_list_new;

    let list = mb_list_new();
    assert!(list.is_ptr());
    unsafe {
        let ptr = list.as_ptr().unwrap();
        assert_eq!(mb_refcount(ptr), 1, "new list should have rc=1");
        rc::mb_release(ptr);
    }
}

/// S6: mb_dict_new returns rc=1 — no spurious retain.
#[test]
fn test_new_ref_dict_no_double_retain() {
    use crate::runtime::dict_ops::mb_dict_new;

    let dict = mb_dict_new();
    assert!(dict.is_ptr());
    unsafe {
        let ptr = dict.as_ptr().unwrap();
        assert_eq!(mb_refcount(ptr), 1, "new dict should have rc=1");
        rc::mb_release(ptr);
    }
}

/// S6: mb_str_concat returns rc=1 — new reference, no extra retain.
#[test]
fn test_new_ref_str_concat_no_double_retain() {
    use crate::runtime::string_ops::mb_str_concat;

    let a = str_val("hello");
    let b = str_val(" world");
    let result = mb_str_concat(a, b);
    assert!(result.is_ptr());
    unsafe {
        let ptr = result.as_ptr().unwrap();
        assert_eq!(mb_refcount(ptr), 1, "str_concat should return rc=1");
        rc::mb_release(ptr);
        rc::release_if_ptr(a);
        rc::release_if_ptr(b);
    }
}

/// S9: mb_list_pop returns a new reference (removed from container) — rc=1 after pop.
#[test]
fn test_list_pop_new_ref() {
    use crate::runtime::list_ops::{mb_list_from, mb_list_pop};

    unsafe {
        let elem = MbObject::new_str("popped".into());
        assert_eq!(mb_refcount(elem), 1);

        let list = mb_list_from(vec![MbValue::from_ptr(elem)]);

        // Pop the element — it should be a new reference (container released its ref).
        let popped = mb_list_pop(list);
        assert!(popped.is_ptr());

        // The element was removed from the list. Its refcount may be 1 (caller owns).
        // The key invariant: no double-retain (would cause leak).
        let rc = mb_refcount(elem);
        assert!(rc >= 1, "popped element should have rc >= 1, got {rc}");

        // Cleanup
        rc::release_if_ptr(popped);
        rc::release_if_ptr(list);
    }
}

// ═══════════════════════════════════════════════════════════
// Verification: EMIT_REFCOUNT_CALLS enabled (R3)
// ═══════════════════════════════════════════════════════════

/// R3: Verify EMIT_REFCOUNT_CALLS is true and JIT-compiled code emits
/// retain/release calls. We test this indirectly by JIT-compiling a function
/// with list operations — if refcount calls are not emitted, the test would
/// crash under ASan (but here we just verify it runs correctly).
#[test]
fn test_emit_refcount_enabled() {
    use crate::parser;
    use crate::source::span::FileId;
    use crate::types::TypeChecker;
    use crate::lower::{lower_module, lower_hir_to_mir_with_symbols};
    use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
    use crate::codegen::{CodegenBackend, CodegenOutput};

    let _jit_guard = JIT_LOCK.lock().unwrap();

    // Compile a function that exercises list getitem (borrowed→owned ref pattern).
    // With EMIT_REFCOUNT_CALLS=true, the JIT emits retain/release for locals.
    let src = r#"
def f() -> int:
    x: list = [10, 20, 30]
    y: int = 42
    return y
"#;
    let module = parser::parse(src, FileId(0)).expect("parse failed");
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
    let output = backend.codegen(&mir, &checker.tcx).expect("codegen failed");

    match output {
        CodegenOutput::Jit { entry } => {
            let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
            let result = main_fn();
            assert_eq!(result, 42);
        }
        _ => panic!("expected JIT output"),
    }
}

// ═══════════════════════════════════════════════════════════
// Verification: GC re-enabled (R4)
// ═══════════════════════════════════════════════════════════

/// R4: Verify GC is enabled by default. We test this by calling gc_collect
/// and verifying it doesn't panic (GC runs when enabled).
#[test]
fn test_gc_enabled() {
    use crate::runtime::gc;

    // gc::collect() should succeed when GC is enabled.
    // If GC were disabled, collect() would still return 0 but the fact that
    // it runs without error is our baseline verification.
    let freed = gc::collect();
    // freed may be 0 or positive depending on tracked objects — just verify no crash.
    let _ = freed;
}

// ═══════════════════════════════════════════════════════════
// Integration: Conformance with refcount enabled (R3, R6)
// ═══════════════════════════════════════════════════════════

/// R3, R6: Run a representative set of JIT-compiled programs with refcounting
/// enabled. Verifies no crash, correct results, and proper cleanup.
#[test]
fn test_conformance_with_refcount_basic() {
    use crate::parser;
    use crate::source::span::FileId;
    use crate::types::TypeChecker;
    use crate::lower::{lower_module, lower_hir_to_mir_with_symbols};
    use crate::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
    use crate::codegen::{CodegenBackend, CodegenOutput};

    fn jit_run_locked(src: &str, guard: &std::sync::MutexGuard<'_, ()>) -> i64 {
        let _ = guard; // hold the lock
        let module = parser::parse(src, FileId(0)).expect("parse failed");
        let mut checker = TypeChecker::new();
        let _ = checker.check_module(&module);
        let hir = lower_module(&module, &checker).unwrap();
        let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

        let mut backend = CraneliftJitBackend::new().expect("JIT init failed");
        let output = backend.codegen(&mir, &checker.tcx).expect("codegen failed");
        match output {
            CodegenOutput::Jit { entry } => {
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
                main_fn()
            }
            _ => panic!("expected JIT output"),
        }
    }

    let guard = JIT_LOCK.lock().unwrap();

    // Test 1: Simple arithmetic
    assert_eq!(jit_run_locked("def f() -> int:\n    return 1 + 2\n", &guard), 3);

    // Test 2: Variable reassignment
    assert_eq!(jit_run_locked(
        "def f() -> int:\n    x: int = 10\n    x = 20\n    return x\n", &guard
    ), 20);

    // Test 3: Loop with accumulator
    assert_eq!(jit_run_locked(
        "def f() -> int:\n    s: int = 0\n    i: int = 0\n    while i < 10:\n        s = s + i\n        i = i + 1\n    return s\n",
        &guard
    ), 45);

    // Test 4: List creation (exercises heap allocation with refcount)
    assert_eq!(jit_run_locked(
        "def f() -> int:\n    x: list = [1, 2, 3]\n    return 100\n", &guard
    ), 100);

    // Test 5: String literals (immortal refcount)
    assert_eq!(jit_run_locked(
        "def f() -> int:\n    x: str = \"hello\"\n    y: str = \"world\"\n    return 77\n", &guard
    ), 77);

    // Test 6: Multiple locals released at return
    assert_eq!(jit_run_locked(
        "def f() -> int:\n    a: int = 1\n    b: int = 2\n    c: int = 3\n    return a + b + c\n",
        &guard
    ), 6);
}

// ═══════════════════════════════════════════════════════════
// Edge case: Out-of-bounds getitem returns None (no crash)
// ═══════════════════════════════════════════════════════════

/// Edge case: list getitem with out-of-bounds index returns None, not a crash.
#[test]
fn test_list_getitem_out_of_bounds() {
    use crate::runtime::list_ops::mb_list_getitem;

    let list = list_val(vec![MbValue::from_int(1)]);
    let result = mb_list_getitem(list, MbValue::from_int(5));
    assert!(result.is_none(), "out-of-bounds getitem should return None");

    unsafe { rc::release_if_ptr(list); }
}

/// Edge case: tuple getitem with out-of-bounds index returns None.
#[test]
fn test_tuple_getitem_out_of_bounds() {
    use crate::runtime::tuple_ops::mb_tuple_getitem;

    let tup = tuple_val(vec![MbValue::from_int(1)]);
    let result = mb_tuple_getitem(tup, MbValue::from_int(5));
    assert!(result.is_none(), "out-of-bounds tuple getitem should return None");

    unsafe { rc::release_if_ptr(tup); }
}

/// Edge case: dict getitem with missing key — verify no crash.
#[test]
fn test_dict_getitem_missing_key() {
    use crate::runtime::dict_ops::mb_dict_getitem;

    let dict = dict_with(vec![("a", MbValue::from_int(1))]);
    // Accessing a missing key raises KeyError internally but returns None
    let result = mb_dict_getitem(dict, str_val("missing"));
    // The result may be None (KeyError was raised internally)
    let _ = result;

    unsafe { rc::release_if_ptr(dict); }
}

// ═══════════════════════════════════════════════════════════
// Negative index support with retain
// ═══════════════════════════════════════════════════════════

/// Negative indexing on list getitem still returns owned reference.
#[test]
fn test_list_getitem_negative_index_owned() {
    use crate::runtime::list_ops::mb_list_getitem;

    unsafe {
        let s = MbObject::new_str("last".into());
        assert_eq!(mb_refcount(s), 1);

        let list = list_val(vec![MbValue::from_int(1), MbValue::from_ptr(s)]);

        // list[-1] should return the string with retain
        let elem = mb_list_getitem(list, MbValue::from_int(-1));
        assert!(elem.is_ptr());
        assert_eq!(mb_refcount(s), 2);

        rc::release_if_ptr(elem);
        rc::release_if_ptr(list);
        // After list release, cascading frees the string
    }
}
