#![cfg(test)]

use crate::runtime::dict_ops::{
    mb_dict_clear, mb_dict_delitem, mb_dict_getitem, mb_dict_new, mb_dict_pop, mb_dict_pop_no_default,
    mb_dict_setitem,
};
use crate::runtime::gc;
use crate::runtime::list_ops::{mb_list_clear, mb_list_getitem, mb_list_new, mb_list_pop, mb_list_remove};
use crate::runtime::module::{
    mb_import, mb_module_getattr, mb_module_register, mb_module_setattr,
};
use crate::runtime::rc::{self, mb_refcount, MbObject, ObjData};
use crate::runtime::set_ops::{mb_set_add, mb_set_clear, mb_set_contains, mb_set_new, mb_set_pop, mb_set_remove};
use crate::runtime::value::MbValue;
use std::collections::HashMap;

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

fn str_val(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn list_val(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

// ═══════════════════════════════════════════════════════════
// Challenge 1: Dict delitem refcount balance
// ═══════════════════════════════════════════════════════════

#[test]
fn challenge_dict_delitem_value_refcount_leak() {
    let _gc = GcGuard::new();
    unsafe {
        let val_obj = MbObject::new_str("heap_value".to_string());
        assert_eq!(mb_refcount(val_obj), 1);

        let dict = mb_dict_new();
        let key = str_val("k");
        mb_dict_setitem(dict, key, MbValue::from_ptr(val_obj));
        // After setitem, dict holds 1 owned reference (rc 1 -> 2)
        assert_eq!(mb_refcount(val_obj), 2);

        // Call del dict["k"]
        mb_dict_delitem(dict, key);

        // EXPECTATION: Since "k" was removed from dict, dict no longer owns val_obj.
        // The refcount of val_obj should drop back to 1 (our local reference).
        let rc_after_del = mb_refcount(val_obj);
        println!("REFCOUNT AFTER DELITEM: {}", rc_after_del);
        
        // Clean up dict and key
        rc::release_if_ptr(dict);
        rc::release_if_ptr(key);

        // Assert expected behavior: rc_after_del should be 1. If it is 2, it leaked!
        assert_eq!(
            rc_after_del, 1,
            "BUG CONFIRMED: mb_dict_delitem leaked value refcount! Expected 1, got {rc_after_del}"
        );
        rc::mb_release(val_obj);
    }
}

// ═══════════════════════════════════════════════════════════
// Challenge 2: Dict clear refcount balance
// ═══════════════════════════════════════════════════════════

#[test]
fn challenge_dict_clear_values_refcount_leak() {
    let _gc = GcGuard::new();
    unsafe {
        let val_obj = MbObject::new_str("contained_val".to_string());
        assert_eq!(mb_refcount(val_obj), 1);

        let dict = mb_dict_new();
        let key = str_val("key1");
        mb_dict_setitem(dict, key, MbValue::from_ptr(val_obj));
        assert_eq!(mb_refcount(val_obj), 2);

        // Call dict.clear()
        mb_dict_clear(dict);

        // EXPECTATION: After clear(), dict is empty. Contained values should be released (rc 2 -> 1).
        let rc_after_clear = mb_refcount(val_obj);
        println!("REFCOUNT AFTER DICT CLEAR: {}", rc_after_clear);

        rc::release_if_ptr(dict);
        rc::release_if_ptr(key);

        assert_eq!(
            rc_after_clear, 1,
            "BUG CONFIRMED: mb_dict_clear leaked values refcount! Expected 1, got {rc_after_clear}"
        );
        rc::mb_release(val_obj);
    }
}

// ═══════════════════════════════════════════════════════════
// Challenge 3: Dict pop default argument refcount
// ═══════════════════════════════════════════════════════════

#[test]
fn challenge_dict_pop_default_refcount_behavior() {
    let _gc = GcGuard::new();
    unsafe {
        let dict = mb_dict_new();
        let key_in = str_val("present_key");
        let val_in = str_val("present_val");
        mb_dict_setitem(dict, key_in, val_in);

        let default_obj = MbObject::new_str("default_val".to_string());
        assert_eq!(mb_refcount(default_obj), 1);

        // 3a: Dict Hit scenario (key is present)
        let popped = mb_dict_pop(dict, key_in, MbValue::from_ptr(default_obj));
        assert_eq!(popped.to_bits(), val_in.to_bits());

        // What happened to default_obj on Hit?
        let rc_hit = mb_refcount(default_obj);
        println!("DEFAULT REFCOUNT ON HIT: {}", rc_hit);

        // 3b: Dict Miss scenario (key is missing)
        let default_obj_miss = MbObject::new_str("default_miss".to_string());
        let missing_key = str_val("missing_key");
        let returned_default = mb_dict_pop(dict, missing_key, MbValue::from_ptr(default_obj_miss));
        assert_eq!(returned_default.to_bits(), MbValue::from_ptr(default_obj_miss).to_bits());
        let rc_miss = mb_refcount(default_obj_miss);
        println!("DEFAULT REFCOUNT ON MISS: {}", rc_miss);

        // Clean up
        rc::release_if_ptr(dict);
        rc::release_if_ptr(key_in);
        rc::release_if_ptr(val_in);
        rc::release_if_ptr(popped);
        rc::release_if_ptr(missing_key);
        rc::release_if_ptr(returned_default);
        rc::mb_release(default_obj);
        rc::mb_release(default_obj_miss);
    }
}

// ═══════════════════════════════════════════════════════════
// Challenge 4: Module cached_value refcount & lifetime
// ═══════════════════════════════════════════════════════════

#[test]
fn challenge_module_cached_value_dangling_ptr() {
    let _gc = GcGuard::new();
    unsafe {
        let mut attrs = HashMap::new();
        attrs.insert("attr1".to_string(), MbValue::from_int(123));
        mb_module_register("test_mod_cached", attrs);

        let name = str_val("test_mod_cached");
        let mod_val1 = mb_import(name);
        assert!(mod_val1.is_ptr());
        let mod_ptr = mod_val1.as_ptr().unwrap();
        let rc1 = mb_refcount(mod_ptr);
        println!("MODULE DICT RC AFTER INITIAL IMPORT: {}", rc1);

        // Release the returned module value from import
        rc::release_if_ptr(mod_val1);

        // Now, import again or query module attribute
        let attr = str_val("attr1");
        let val_got = mb_module_getattr(name, attr);
        println!("GOT ATTR VALUE AFTER RELEASE: {:?}", val_got);

        rc::release_if_ptr(name);
        rc::release_if_ptr(attr);
    }
}

// ═══════════════════════════════════════════════════════════
// Challenge 5: Set high-frequency pop/remove & empty containers
// ═══════════════════════════════════════════════════════════

#[test]
fn challenge_set_pop_remove_empty() {
    let _gc = GcGuard::new();
    unsafe {
        let set = mb_set_new();
        // Pop on empty set raises KeyError
        let pop_empty = mb_set_pop(set);
        assert!(pop_empty.is_none());
        assert!(crate::runtime::exception::mb_has_exception().as_bool() == Some(true));
        crate::runtime::exception::mb_clear_exception();

        // High frequency add and pop
        for i in 0..1000 {
            let elem = str_val(&format!("item_{i}"));
            mb_set_add(set, elem);
            rc::release_if_ptr(elem);
        }

        // Pop all 1000 items
        for _ in 0..1000 {
            let item = mb_set_pop(set);
            assert!(item.is_ptr());
            rc::release_if_ptr(item);
        }

        // Pop on empty set again
        let pop_empty2 = mb_set_pop(set);
        assert!(pop_empty2.is_none());
        crate::runtime::exception::mb_clear_exception();

        rc::release_if_ptr(set);
    }
}

// ═══════════════════════════════════════════════════════════
// Challenge 6: List high-frequency pop/remove & empty containers
// ═══════════════════════════════════════════════════════════

#[test]
fn challenge_list_pop_remove_empty() {
    let _gc = GcGuard::new();
    unsafe {
        let list = mb_list_new();
        // Pop on empty list
        let popped = mb_list_pop(list);
        assert!(popped.is_none());
        assert!(crate::runtime::exception::mb_has_exception().as_bool() == Some(true));
        crate::runtime::exception::mb_clear_exception();

        // Push 1000 items
        for i in 0..1000 {
            let item = str_val(&format!("elem_{i}"));
            crate::runtime::list_ops::mb_list_append(list, item);
            rc::release_if_ptr(item);
        }

        // Pop all
        for _ in 0..1000 {
            let p = mb_list_pop(list);
            assert!(p.is_ptr());
            rc::release_if_ptr(p);
        }

        rc::release_if_ptr(list);
    }
}

// ═══════════════════════════════════════════════════════════
// Challenge 7: Module attribute updates under re-binding
// ═══════════════════════════════════════════════════════════

#[test]
fn challenge_module_attribute_rebinding_refcount() {
    let _gc = GcGuard::new();
    unsafe {
        let mut attrs = HashMap::new();
        attrs.insert("x".to_string(), MbValue::from_int(1));
        mb_module_register("mod_rebind", attrs);

        let mod_name = str_val("mod_rebind");
        let attr_name = str_val("x");

        let val1_obj = MbObject::new_str("val1".to_string());
        assert_eq!(mb_refcount(val1_obj), 1);

        // Bind x = val1
        mb_module_setattr(mod_name, attr_name, MbValue::from_ptr(val1_obj));
        // module.attrs holds a reference (+1 -> 2)
        assert_eq!(mb_refcount(val1_obj), 2);

        // Re-bind x = val2
        let val2_obj = MbObject::new_str("val2".to_string());
        mb_module_setattr(mod_name, attr_name, MbValue::from_ptr(val2_obj));

        // EXPECTATION: val1_obj's reference held by module should be released!
        let rc_val1_after = mb_refcount(val1_obj);
        let rc_val2_after = mb_refcount(val2_obj);

        println!("VAL1 RC AFTER REBIND: {}", rc_val1_after);
        println!("VAL2 RC AFTER REBIND: {}", rc_val2_after);

        assert_eq!(rc_val1_after, 1, "Re-binding module attribute should release previous value reference");
        assert_eq!(rc_val2_after, 2, "Re-binding module attribute should retain new value reference");

        rc::release_if_ptr(mod_name);
        rc::release_if_ptr(attr_name);
        rc::mb_release(val1_obj);
        rc::mb_release(val2_obj);
    }
}
