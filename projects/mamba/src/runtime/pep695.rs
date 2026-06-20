//! PEP 695 runtime objects (#233).
//!
//! Backs the `__mb_pep695_typevar__` / `__mb_pep695_type_alias__` intrinsics
//! injected by `lower::pep695`:
//!
//! * `mb_pep695_typevar` builds a TypeVar / TypeVarTuple / ParamSpec instance
//!   with `__name__`, variance flags, and *lazy* `__bound__` /
//!   `__constraints__` (zero-arg thunks evaluated on first attribute access,
//!   then cached — CPython's deferred-evaluation semantics).
//! * `mb_pep695_type_alias` builds a TypeAliasType instance whose `__value__`
//!   is likewise lazy, enabling recursive aliases (`type R = R | None`).
//!
//! This module also owns the generic function-attribute side registry
//! (FUNC_ATTRS) used for `f.__type_params__` (readable and writable) and any
//! other user-set attributes on function / closure values, which carry no
//! field storage of their own.

use super::rc::{MbObject, ObjData};
use super::value::MbValue;
use std::collections::HashMap;

/// Field names used to stash the lazy evaluation thunks on the instances.
const BOUND_THUNK: &str = "__mb_bound_thunk__";
const CONSTRAINTS_THUNK: &str = "__mb_constraints_thunk__";
const VALUE_THUNK: &str = "__mb_value_thunk__";

fn extract_str(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        match &(*p).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    })
}

/// Public field setter over an Instance MbValue (typing TypeVar kwargs).
pub fn instance_field_set_pub(inst: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        instance_field_set(ptr, name, value);
    }
}

fn instance_field_set(inst: *mut MbObject, name: &str, value: MbValue) {
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            super::rc::retain_if_ptr(value);
            let old = fields.write().unwrap().insert(name.to_string(), value);
            if let Some(prev) = old {
                super::rc::release_if_ptr(prev);
            }
        }
    }
}

/// `__mb_pep695_typevar__(name, kind, bound_thunk, constraints_thunk)`.
///
/// kind: 0 = TypeVar, 1 = TypeVarTuple, 2 = ParamSpec. The thunks are
/// zero-arg callables (or None) evaluated lazily on first `__bound__` /
/// `__constraints__` access.
pub fn mb_pep695_typevar(
    name: MbValue,
    kind: MbValue,
    bound_thunk: MbValue,
    constraints_thunk: MbValue,
) -> MbValue {
    let class_name = match kind.as_int().unwrap_or(0) {
        1 => "TypeVarTuple",
        2 => "ParamSpec",
        _ => "TypeVar",
    };
    let inst = MbObject::new_instance(class_name.to_string());
    let name_str = extract_str(name).unwrap_or_default();
    instance_field_set(
        inst,
        "__name__",
        MbValue::from_ptr(MbObject::new_str(name_str)),
    );
    // `[ ]`-syntax params always infer variance (PEP 695).
    instance_field_set(inst, "__infer_variance__", MbValue::from_bool(true));
    instance_field_set(inst, "__covariant__", MbValue::from_bool(false));
    instance_field_set(inst, "__contravariant__", MbValue::from_bool(false));
    if bound_thunk.is_none() {
        instance_field_set(inst, "__bound__", MbValue::none());
    } else {
        instance_field_set(inst, BOUND_THUNK, bound_thunk);
    }
    if constraints_thunk.is_none() {
        instance_field_set(
            inst,
            "__constraints__",
            MbValue::from_ptr(MbObject::new_tuple(vec![])),
        );
    } else {
        instance_field_set(inst, CONSTRAINTS_THUNK, constraints_thunk);
    }
    MbValue::from_ptr(inst)
}

/// Build a TypeVar-family instance directly (used by the `typing` module's
/// runtime constructors: `TypeVar('T', ...)`, `ParamSpec('P')`, ...).
///
/// kind: 0 = TypeVar, 1 = TypeVarTuple, 2 = ParamSpec. `constraints` holds
/// the eagerly evaluated constraint values (empty for none). Runtime-call
/// constructed vars never infer variance (unlike `[T]`-syntax params).
pub fn make_typevar_instance(name: &str, kind: i64, constraints: Vec<MbValue>) -> MbValue {
    let class_name = match kind {
        1 => "TypeVarTuple",
        2 => "ParamSpec",
        _ => "TypeVar",
    };
    let inst = MbObject::new_instance(class_name.to_string());
    instance_field_set(
        inst,
        "__name__",
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    instance_field_set(inst, "__infer_variance__", MbValue::from_bool(false));
    instance_field_set(inst, "__covariant__", MbValue::from_bool(false));
    instance_field_set(inst, "__contravariant__", MbValue::from_bool(false));
    instance_field_set(inst, "__bound__", MbValue::none());
    instance_field_set(
        inst,
        "__constraints__",
        MbValue::from_ptr(MbObject::new_tuple(constraints)),
    );
    MbValue::from_ptr(inst)
}

/// `__mb_pep695_type_alias__(name, value_thunk, params_tuple)`.
pub fn mb_pep695_type_alias(name: MbValue, value_thunk: MbValue, params: MbValue) -> MbValue {
    let inst = MbObject::new_instance("TypeAliasType".to_string());
    let name_str = extract_str(name).unwrap_or_default();
    instance_field_set(
        inst,
        "__name__",
        MbValue::from_ptr(MbObject::new_str(name_str)),
    );
    instance_field_set(inst, "__type_params__", params);
    instance_field_set(inst, VALUE_THUNK, value_thunk);
    MbValue::from_ptr(inst)
}

/// True for the instance classes built by this module.
pub fn is_pep695_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "TypeVar" | "TypeVarTuple" | "ParamSpec" | "TypeAliasType"
    )
}

/// Lazy attribute resolution for TypeVar / TypeAliasType instances: on first
/// access of `__bound__` / `__constraints__` / `__value__` the stored thunk
/// is called, the result cached in the field, and the thunk slot dropped.
/// Returns None when the attribute is not lazily managed here (so the normal
/// instance-field path proceeds).
pub fn instance_lazy_attr_hook(obj: MbValue, class_name: &str, attr_name: &str) -> Option<MbValue> {
    if !is_pep695_class(class_name) {
        return None;
    }
    let thunk_field = match attr_name {
        "__bound__" => BOUND_THUNK,
        "__constraints__" => CONSTRAINTS_THUNK,
        "__value__" => VALUE_THUNK,
        _ => return None,
    };
    let ptr = obj.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            // Cached already?
            if let Some(v) = fields.read().unwrap().get(attr_name).copied() {
                super::rc::retain_if_ptr(v);
                return Some(v);
            }
            let thunk = fields.read().unwrap().get(thunk_field).copied()?;
            // Call outside any held lock — the thunk runs arbitrary user code.
            let value = super::class::mb_call0(thunk);
            instance_field_set(ptr as *mut MbObject, attr_name, value);
            super::rc::retain_if_ptr(value);
            return Some(value);
        }
    }
    None
}

// ── Function attribute side registry ──

thread_local! {
    /// Attributes set on function / closure values (`f.attr = v`), keyed by
    /// the function value's bits. Functions carry no field storage, so PEP
    /// 695's writable `__type_params__` (and generic user attributes) live
    /// here.
    static FUNC_ATTRS: std::cell::RefCell<HashMap<u64, HashMap<String, MbValue>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// True when `v` is a known function or closure value. TAG_FUNC pointers are
/// unambiguous functions (class methods included, which skip the name
/// registry); int-tagged closure handles must be registered in the closure
/// module's name registry so plain ints never accrue attributes.
pub fn is_attrable_function(v: MbValue) -> bool {
    if let Some(addr) = v.as_func() {
        return addr > 4096;
    }
    if super::generator::is_known_generator(v) {
        return false;
    }
    !super::closure::mb_func_get_name(v).is_none() || super::closure::mb_func_is_registered(v)
}

/// Store `func.attr = value` in the side registry.
pub fn func_attrs_set(func: MbValue, attr: MbValue, value: MbValue) {
    let Some(attr_name) = extract_str(attr) else {
        return;
    };
    unsafe { super::rc::retain_if_ptr(value) };
    FUNC_ATTRS.with(|m| {
        let mut map = m.borrow_mut();
        let entry = map.entry(func.to_bits()).or_default();
        if let Some(prev) = entry.insert(attr_name, value) {
            unsafe { super::rc::release_if_ptr(prev) };
        }
    });
}

/// Read a previously stored function attribute. Returns None on miss.
pub fn func_attrs_get(func: MbValue, attr_name: &str) -> Option<MbValue> {
    FUNC_ATTRS
        .with(|m| {
            m.borrow()
                .get(&func.to_bits())
                .and_then(|attrs| attrs.get(attr_name))
                .copied()
        })
        .map(|v| {
            unsafe { super::rc::retain_if_ptr(v) };
            v
        })
}

/// Test-support / shutdown cleanup: drop all stored function attributes.
pub fn cleanup_func_attrs() {
    let _ = FUNC_ATTRS.with(|m| {
        m.try_borrow_mut().map(|mut map| {
            for (_, attrs) in map.drain() {
                for (_, v) in attrs {
                    unsafe { super::rc::release_if_ptr(v) };
                }
            }
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typevar_builds_named_instance_with_eager_defaults() {
        let name = MbValue::from_ptr(MbObject::new_str("T".to_string()));
        let tv = mb_pep695_typevar(name, MbValue::from_int(0), MbValue::none(), MbValue::none());
        let ptr = tv.as_ptr().expect("instance");
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                assert_eq!(class_name, "TypeVar");
                let f = fields.read().unwrap();
                assert!(f.get("__bound__").unwrap().is_none());
                assert_eq!(f.get("__infer_variance__").unwrap().as_bool(), Some(true));
            } else {
                panic!("expected Instance");
            }
        }
    }

    #[test]
    fn typevar_kind_selects_class_name() {
        let mk = |k: i64| {
            let name = MbValue::from_ptr(MbObject::new_str("P".to_string()));
            mb_pep695_typevar(name, MbValue::from_int(k), MbValue::none(), MbValue::none())
        };
        for (k, expected) in [(0, "TypeVar"), (1, "TypeVarTuple"), (2, "ParamSpec")] {
            let v = mk(k);
            let ptr = v.as_ptr().unwrap();
            unsafe {
                if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                    assert_eq!(class_name, expected);
                } else {
                    panic!("expected Instance");
                }
            }
        }
    }

    #[test]
    fn func_attrs_roundtrip() {
        let f = MbValue::from_int(123456789);
        let attr = MbValue::from_ptr(MbObject::new_str("__type_params__".to_string()));
        let val = MbValue::from_int(42);
        func_attrs_set(f, attr, val);
        assert_eq!(
            func_attrs_get(f, "__type_params__").unwrap().as_int(),
            Some(42)
        );
        assert!(func_attrs_get(f, "missing").is_none());
        cleanup_func_attrs();
        assert!(func_attrs_get(f, "__type_params__").is_none());
    }
}
