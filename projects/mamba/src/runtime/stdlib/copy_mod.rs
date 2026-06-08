use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// copy module for Mamba (#414).
///
/// Provides: copy.copy(obj) — shallow copy, copy.deepcopy(obj) — recursive deep copy.
/// Primitives (int, float, bool, None) are returned as-is since they are immutable.
/// Heap objects (str, list, dict, tuple, set) are cloned at the appropriate depth.
use std::collections::HashMap;

/// Helper: extract a string from an MbValue.
#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Dispatcher for copy.copy — unpacks args and calls mb_copy_copy.
unsafe extern "C" fn dispatch_copy(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let arg = unsafe { *args_ptr };
    mb_copy_copy(arg)
}

/// Dispatcher for copy.deepcopy — unpacks args and calls mb_copy_deepcopy.
/// The second argument (memo dict) is accepted but not yet honored.
unsafe extern "C" fn dispatch_deepcopy(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let arg = unsafe { *args_ptr };
    mb_copy_deepcopy(arg)
}

/// Register the copy module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("copy", dispatch_copy as *const () as usize),
        ("deepcopy", dispatch_deepcopy as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // copy.Error class shell (Instance with class_name). CPython exposes
    // this as a subclass of Exception raised when an unpicklable callable
    // is encountered during deepcopy; mamba returns a class-like Instance
    // so callers can test `isinstance` / catch via class_name.
    let error_shell = {
        use super::super::rc::{MbObject, MbObjectHeader, ObjKind};
        let obj = Box::new(MbObject {
            header: MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: "copy.Error".to_string(),
                fields: crate::runtime::rc::MbRwLock::new(FxHashMap::default()),
            },
        });
        MbValue::from_ptr(Box::into_raw(obj))
    };
    attrs.insert("Error".to_string(), error_shell);

    super::register_module("copy", attrs);
}

/// copy.copy(obj) -> shallow copy.
///
/// - Primitives (int, float, bool, None): returned as-is (immutable).
/// - Str: clone the string into a new object.
/// - List: clone the vec of MbValues (inner refs are shared).
/// - Dict: clone the map (inner values are shared).
/// - Tuple: clone the vec.
/// - Set: clone the vec.
/// - Other heap objects: returned as-is (no copy semantics defined).
pub fn mb_copy_copy(obj: MbValue) -> MbValue {
    if obj.is_none()
        || obj.as_int().is_some()
        || obj.as_float().is_some()
        || obj.as_bool().is_some()
    {
        return obj;
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
                ObjData::List(lock) => {
                    let items = lock.read().unwrap();
                    MbValue::from_ptr(MbObject::new_list_borrowed(items.to_vec()))
                }
                ObjData::Dict(lock) => {
                    let map = lock.read().unwrap();
                    let d = MbObject::new_dict();
                    if let ObjData::Dict(ref new_lock) = (*d).data {
                        let mut new_map = new_lock.write().unwrap();
                        *new_map = map.clone();
                        // Retain all values borrowed from the source dict.
                        for val in new_map.values() {
                            super::super::rc::retain_if_ptr(*val);
                        }
                    }
                    MbValue::from_ptr(d)
                }
                ObjData::Tuple(items) => {
                    MbValue::from_ptr(MbObject::new_tuple_borrowed(items.clone()))
                }
                ObjData::Set(lock) => {
                    let items = lock.read().unwrap();
                    MbValue::from_ptr(MbObject::new_set_borrowed(items.to_vec()))
                }
                _ => {
                    // Return input — retain so JIT can release both arg and result VRegs.
                    super::super::rc::retain_if_ptr(obj);
                    obj
                }
            }
        }
    } else {
        // Inline value — no retain needed.
        obj
    }
}

/// copy.deepcopy(obj) -> recursive deep copy.
///
/// - Primitives and Str: same as shallow copy.
/// - List: new list with deepcopy of each element.
/// - Dict: new dict with deepcopy of each value.
/// - Tuple: new tuple with deepcopy of each element.
/// - Set: new set with deepcopy of each element.
/// - Other: same as shallow copy.
pub fn mb_copy_deepcopy(obj: MbValue) -> MbValue {
    if obj.is_none()
        || obj.as_int().is_some()
        || obj.as_float().is_some()
        || obj.as_bool().is_some()
    {
        return obj;
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
                ObjData::List(lock) => {
                    let items = lock.read().unwrap();
                    let deep_items: Vec<MbValue> =
                        items.iter().map(|v| mb_copy_deepcopy(*v)).collect();
                    MbValue::from_ptr(MbObject::new_list(deep_items))
                }
                ObjData::Dict(lock) => {
                    let map = lock.read().unwrap();
                    let d = MbObject::new_dict();
                    if let ObjData::Dict(ref new_lock) = (*d).data {
                        let mut new_map = new_lock.write().unwrap();
                        for (k, v) in map.iter() {
                            new_map.insert(k.clone(), mb_copy_deepcopy(*v));
                        }
                    }
                    MbValue::from_ptr(d)
                }
                ObjData::Tuple(items) => {
                    let deep_items: Vec<MbValue> =
                        items.iter().map(|v| mb_copy_deepcopy(*v)).collect();
                    MbValue::from_ptr(MbObject::new_tuple(deep_items))
                }
                ObjData::Set(lock) => {
                    let items = lock.read().unwrap();
                    let deep_items: Vec<MbValue> =
                        items.iter().map(|v| mb_copy_deepcopy(*v)).collect();
                    MbValue::from_ptr(MbObject::new_set(deep_items))
                }
                _ => obj,
            }
        }
    } else {
        obj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_primitives() {
        let int_val = MbValue::from_int(42);
        assert_eq!(mb_copy_copy(int_val).as_int(), Some(42));

        let float_val = MbValue::from_float(3.14);
        assert_eq!(mb_copy_copy(float_val).as_float(), Some(3.14));

        let bool_val = MbValue::from_bool(true);
        assert_eq!(mb_copy_copy(bool_val).as_bool(), Some(true));

        let none_val = MbValue::none();
        assert!(mb_copy_copy(none_val).is_none());
    }

    #[test]
    fn test_shallow_copy_list() {
        let inner = MbValue::from_int(10);
        let list = MbValue::from_ptr(MbObject::new_list(vec![inner]));
        let copied = mb_copy_copy(list);

        // Copied list should be a different object
        assert_ne!(list.as_ptr(), copied.as_ptr());

        // But inner values are shared (same bit pattern for int)
        unsafe {
            if let ObjData::List(ref lock) = (*copied.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].as_int(), Some(10));
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_deepcopy_nested_list() {
        let inner_list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let outer = MbValue::from_ptr(MbObject::new_list(vec![inner_list]));
        let deep = mb_copy_deepcopy(outer);

        // Outer list is a different object
        assert_ne!(outer.as_ptr(), deep.as_ptr());

        // Inner list should also be a different object
        unsafe {
            let outer_items = if let ObjData::List(ref lock) = (*outer.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                items[0].as_ptr().unwrap()
            } else {
                panic!("expected list");
            };
            let deep_items = if let ObjData::List(ref lock) = (*deep.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                items[0].as_ptr().unwrap()
            } else {
                panic!("expected list");
            };
            assert_ne!(outer_items, deep_items);
        }
    }

    #[test]
    fn test_copy_str() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let copied = mb_copy_copy(s);
        // Different pointer
        assert_ne!(s.as_ptr(), copied.as_ptr());
        // Same content
        unsafe {
            if let ObjData::Str(ref cs) = (*copied.as_ptr().unwrap()).data {
                assert_eq!(cs, "hello");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_copy_dict() {
        let d = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*d).data {
                let mut map = lock.write().unwrap();
                map.insert("key".into(), MbValue::from_int(99));
            }
        }
        let original = MbValue::from_ptr(d);
        let copied = mb_copy_copy(original);

        assert_ne!(original.as_ptr(), copied.as_ptr());
        unsafe {
            if let ObjData::Dict(ref lock) = (*copied.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("key").and_then(|v| v.as_int()), Some(99));
            } else {
                panic!("expected Dict");
            }
        }
    }

    #[test]
    fn test_copy_tuple() {
        let t = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let copied = mb_copy_copy(t);
        assert_ne!(t.as_ptr(), copied.as_ptr());
        unsafe {
            if let ObjData::Tuple(ref items) = (*copied.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_int(), Some(1));
                assert_eq!(items[1].as_int(), Some(2));
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_deepcopy_primitives() {
        assert_eq!(mb_copy_deepcopy(MbValue::from_int(7)).as_int(), Some(7));
        assert_eq!(
            mb_copy_deepcopy(MbValue::from_float(2.5)).as_float(),
            Some(2.5)
        );
        assert!(mb_copy_deepcopy(MbValue::none()).is_none());
        assert_eq!(
            mb_copy_deepcopy(MbValue::from_bool(false)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_deepcopy_dict() {
        let inner_list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(10)]));
        let d = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*d).data {
                let mut map = lock.write().unwrap();
                map.insert("lst".into(), inner_list);
            }
        }
        let original = MbValue::from_ptr(d);
        let deep = mb_copy_deepcopy(original);

        // Dict pointer is different
        assert_ne!(original.as_ptr(), deep.as_ptr());
        // Inner list pointer is also different
        unsafe {
            let orig_list = if let ObjData::Dict(ref lock) = (*original.as_ptr().unwrap()).data {
                lock.read().unwrap().get("lst").unwrap().as_ptr().unwrap()
            } else {
                panic!("expected Dict")
            };
            let deep_list = if let ObjData::Dict(ref lock) = (*deep.as_ptr().unwrap()).data {
                lock.read().unwrap().get("lst").unwrap().as_ptr().unwrap()
            } else {
                panic!("expected Dict")
            };
            assert_ne!(orig_list, deep_list);
        }
    }

    #[test]
    fn test_copy_set() {
        let s = MbValue::from_ptr(MbObject::new_set(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let copied = mb_copy_copy(s);
        assert_ne!(s.as_ptr(), copied.as_ptr());
        unsafe {
            if let ObjData::Set(ref lock) = (*copied.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 2);
            } else {
                panic!("expected Set");
            }
        }
    }
}
