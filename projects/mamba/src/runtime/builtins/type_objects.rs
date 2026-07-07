use crate::runtime::rc::{self, MbObject, ObjData, ObjKind};
use crate::runtime::stdlib::uuid_mod;
use crate::runtime::value::MbValue;
use crate::runtime::{async_rt, class, dict_ops, exception, gc, iter, module};
use rustc_hash::FxHashMap;

/// type(value) — return a type object with __name__ attribute.
/// Returns an Instance object with class_name="type" and a __name__ field
/// so that `type(x).__name__` works like Python.
pub fn mb_type(val: MbValue) -> MbValue {
    let name = if let Some(iter_type) = iter::mb_iter_type_name(val) {
        iter_type
    } else if val.is_int() && async_rt::is_known_coroutine(val) {
        "coroutine"
    } else if val.is_int() {
        // uuid handles (NAMESPACE_*, uuid4(), ...) are int-tagged values; report
        // their real type so `type(uuid.NAMESPACE_DNS).__name__ == "UUID"`.
        let id = val.as_int().unwrap_or(0) as u64;
        if uuid_mod::is_uuid_handle(id) {
            "UUID"
        } else {
            "int"
        }
    } else if val.is_float() {
        "float"
    } else if val.is_bool() {
        "bool"
    } else if val.is_none() {
        "NoneType"
    } else if val.is_ellipsis() {
        "ellipsis"
    } else if val.is_not_implemented() {
        "NotImplementedType"
    } else if val.as_func().is_some() {
        // TAG_FUNC: JIT-compiled or extern function pointer.
        if let Some(addr) = val.as_func() {
            if module::is_native_func(addr as u64) {
                "builtin_function_or_method"
            } else {
                "function"
            }
        } else {
            "function"
        }
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Instance { class_name, .. } if class_name == "__instance_dict_proxy__" => {
                    return make_type_object("dict");
                }
                ObjData::Instance { class_name, fields } if class_name == "type" => {
                    if let Some(type_name) = fields.read().ok().and_then(|f| {
                        f.get("__name__").and_then(|v| {
                            v.as_ptr().and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                        })
                    }) {
                        if let Some(meta) = class::class_metaclass_name(&type_name) {
                            return make_type_object(&meta);
                        }
                    }
                    return make_type_object(class_name);
                }
                ObjData::Instance { class_name, .. } => {
                    return make_type_object(class_name);
                }
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::BigInt(_) => "int",
                ObjData::Complex(_, _) => "complex",
                ObjData::CodeObject { .. } => "code",
            }
        }
    } else {
        "unknown"
    };
    make_type_object(name)
}

pub fn mb_type_no_args() -> MbValue {
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "type() takes 1 or 3 arguments".to_string(),
        )),
    );
    MbValue::none()
}

pub fn mb_type2(_name: MbValue, _bases: MbValue) -> MbValue {
    mb_type_no_args()
}

pub(crate) fn reject_non_constructible_type_object(name: &str) -> Option<MbValue> {
    if !matches!(
        name,
        "list_iterator" | "list_reverseiterator" | "range_iterator"
    ) {
        return None;
    }
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "cannot create '{name}' instances"
        ))),
    );
    Some(MbValue::none())
}

// ── Type object singleton cache ────────────────────────────────────────────────
//
// Per-thread cache of `type(x)` results keyed by type-name string.
// `mb_type()` and `mb_builtin_type_obj()` share the same cache so that
// `type(True) is bool` holds: both sides resolve to the same heap pointer.
//
// GC note: the objects are never freed because they are GC-rooted on first
// creation and the cache keeps one permanent ref. Returned values are retained
// for the caller, so a JIT-side release cannot invalidate the cached singleton.
thread_local! {
    static TYPE_OBJ_CACHE: std::cell::RefCell<FxHashMap<String, MbValue>> =
        std::cell::RefCell::new(FxHashMap::default());
}

/// Create (or look up) a type object singleton for the given type name.
///
/// Returns a cached Instance with `class_name="type"` and `__name__=name`.
/// The first call allocates and GC-roots the object; subsequent calls return
/// the same heap pointer, making `type(x) is int` / `type(x) is bool` work.
pub(crate) fn make_type_object(name: &str) -> MbValue {
    TYPE_OBJ_CACHE.with(|cache| {
        // Fast path: already cached.
        if let Some(&val) = cache.borrow().get(name) {
            unsafe {
                rc::retain_if_ptr(val);
            }
            return val;
        }
        // Slow path: create the singleton.
        let mut fields = FxHashMap::default();
        fields.insert(
            "__name__".to_string(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
        );
        fields.insert(
            "__module__".to_string(),
            MbValue::from_ptr(MbObject::new_str("builtins".to_string())),
        );
        fields.insert(
            "__doc__".to_string(),
            MbValue::from_ptr(MbObject::new_str(format!("{name} type object."))),
        );
        let obj = Box::new(MbObject {
            header: rc::MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: "type".to_string(),
                fields: crate::runtime::rc::MbRwLock::new(fields),
            },
        });
        let val = MbValue::from_ptr(Box::into_raw(obj));
        // Root the object so the GC never frees it.
        gc::gc_add_root(val);
        cache.borrow_mut().insert(name.to_string(), val);
        unsafe {
            rc::retain_if_ptr(val);
        }
        val
    })
}

/// Return the singleton type object for a builtin type name.
///
/// Called from JIT code generated for builtin type names used in non-call
/// position (e.g. `bool`, `int`, `list` on the right-hand side of `is`).
/// Shares the same `TYPE_OBJ_CACHE` as `make_type_object` / `mb_type()`, so
/// `type(True) is bool` evaluates to `True`.
pub fn mb_builtin_type_obj(name: MbValue) -> MbValue {
    let name_str: String = if let Some(ptr) = name.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                s.clone()
            } else {
                String::new()
            }
        }
    } else {
        String::new()
    };
    make_type_object(&name_str)
}

fn value_is_abstractmethod_marker(val: MbValue) -> bool {
    let Some(ptr) = val.as_ptr() else {
        return false;
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            return lock
                .read()
                .unwrap()
                .get("__isabstractmethod__")
                .and_then(|v| v.as_bool())
                == Some(true);
        }
    }
    false
}

/// type(name, bases, dict) — 3-arg form: dynamically create a new class.
///
/// `name` is a string (the class name), `bases` is a tuple of base class names
/// (or type objects), `dict` is a dict of class attributes / methods.
/// Returns a type object (Instance with class_name="type" and __name__ field).
///
/// The new class is registered in the class registry so that isinstance/issubclass
/// and attribute lookup work correctly.
// @spec .aw/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md#R1
// @spec .aw/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md#R2
// @spec .aw/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md#R4
pub fn mb_type3(name: MbValue, bases: MbValue, dict: MbValue) -> MbValue {
    // 1. Extract name string
    let Some(class_name) = name.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    }) else {
        super::raise_type_error("type.__new__() argument 1 must be str, not object".to_string());
        return MbValue::none();
    };

    // 2. Extract bases tuple -> list of base class name strings
    let Some(base_items) = bases.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Tuple(items) = &(*ptr).data {
            Some(items.clone())
        } else {
            None
        }
    }) else {
        super::raise_type_error("type.__new__() argument 2 must be tuple, not list".to_string());
        return MbValue::none();
    };
    let mut base_names = Vec::new();
    for item in base_items {
        let Some(base_name) = class::resolve_class_name(item) else {
            super::raise_type_error(
                "metaclass conflict: the metaclass of a derived class must be a (non-strict) subclass of the metaclasses of all its bases"
                    .to_string(),
            );
            return MbValue::none();
        };
        if base_name == "bool" {
            super::raise_type_error("type 'bool' is not an acceptable base type".to_string());
            return MbValue::none();
        }
        base_names.push(base_name);
    }
    if base_names.is_empty() {
        base_names.push("object".to_string());
    }
    let layout_bases = base_names
        .iter()
        .filter(|name| {
            matches!(
                name.as_str(),
                "int"
                    | "str"
                    | "float"
                    | "complex"
                    | "list"
                    | "dict"
                    | "tuple"
                    | "set"
                    | "frozenset"
            )
        })
        .count();
    if layout_bases > 1 {
        super::raise_type_error("multiple bases have instance lay-out conflict".to_string());
        return MbValue::none();
    }

    // 3. Extract dict -> class attributes and methods
    // #974: Improved callable detection — TAG_FUNC, closure handles (TAG_INT),
    // and dunder-named entries are all classified as methods so that __init__,
    // __repr__, etc. passed through the dict are properly dispatched.
    let mut methods = std::collections::HashMap::new();
    let mut class_attrs: Vec<(String, MbValue)> = Vec::new();
    let mut abstract_names: Vec<String> = Vec::new();
    let namespace_dict = dict_ops::mb_dict_new();
    let dict_source = if dict
        .as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
    {
        dict
    } else if let Some(backing) = class::unwrap_dictlike_data(dict) {
        backing
    } else {
        super::raise_type_error("type.__new__() argument 3 must be dict".to_string());
        return MbValue::none();
    };
    if let Some(ptr) = dict_source.as_ptr() {
        unsafe {
            let ObjData::Dict(ref lock) = (*ptr).data else {
                super::raise_type_error("type.__new__() argument 3 must be dict".to_string());
                return MbValue::none();
            };
            let pairs = lock.read().unwrap();
            for (k, v) in pairs.iter() {
                let key = k.to_string();
                if key == "__classcell__" {
                    if !class::consume_classcell_marker_for_type_new(&class_name, *v) {
                        return MbValue::none();
                    }
                    continue;
                }
                dict_ops::mb_dict_setitem(
                    namespace_dict,
                    MbValue::from_ptr(MbObject::new_str(key.clone())),
                    *v,
                );
                if value_is_abstractmethod_marker(*v) {
                    abstract_names.push(key.clone());
                }
                let is_callable = super::resolve_callable(*v).is_some();
                let is_dunder = key.starts_with("__") && key.ends_with("__");
                let is_metadata_dunder =
                    matches!(key.as_str(), "__qualname__" | "__doc__" | "__module__");
                if is_callable || (is_dunder && !is_metadata_dunder && !v.is_none()) {
                    methods.insert(key, *v);
                } else {
                    class_attrs.push((key, *v));
                }
            }
        }
    }

    // 4. Register the class in the class registry
    class::mb_class_register(&class_name, base_names.clone(), methods);

    // 5. Set class attributes (non-method entries from dict)
    let cls_name_val = MbValue::from_ptr(MbObject::new_str(class_name.clone()));
    for (key, val) in &class_attrs {
        let attr_name_val = MbValue::from_ptr(MbObject::new_str(key.clone()));
        class::mb_class_set_class_attr(cls_name_val, attr_name_val, *val);
    }
    if !abstract_names.is_empty() {
        let names = MbValue::from_ptr(MbObject::new_list(
            abstract_names
                .into_iter()
                .map(|name| MbValue::from_ptr(MbObject::new_str(name)))
                .collect(),
        ));
        class::mb_class_set_abstractmethods(
            MbValue::from_ptr(MbObject::new_str(class_name.clone())),
            names,
        );
    }

    // 6. Return a type object with CPython-visible class metadata.
    let type_obj = make_type_object(&class_name);
    if let Some(ptr) = type_obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let qualname = dict_ops::mb_dict_get(
                    namespace_dict,
                    MbValue::from_ptr(MbObject::new_str("__qualname__".to_string())),
                    MbValue::none(),
                );
                let doc = dict_ops::mb_dict_get(
                    namespace_dict,
                    MbValue::from_ptr(MbObject::new_str("__doc__".to_string())),
                    MbValue::none(),
                );
                let mut guard = fields.write().unwrap();
                guard.insert(
                    "__qualname__".to_string(),
                    if qualname.is_none() {
                        MbValue::from_ptr(MbObject::new_str(class_name.clone()))
                    } else {
                        qualname
                    },
                );
                guard.insert("__doc__".to_string(), doc);
                guard.insert("__mamba_type_namespace__".to_string(), namespace_dict);
            }
        }
    }
    type_obj
}

pub fn mb_type3_kwargs(name: MbValue, bases: MbValue, dict: MbValue, kwargs: MbValue) -> MbValue {
    let type_obj = mb_type3(name, bases, dict);
    let Some(class_name) = class::resolve_class_name(type_obj) else {
        return type_obj;
    };
    let metaclass = kwargs.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().ok().and_then(|map| {
                map.get(&dict_ops::DictKey::Str("metaclass".to_string()))
                    .copied()
            })
        } else {
            None
        }
    });
    if let Some(meta) = metaclass {
        let Some(meta_name) = class::resolve_class_name(meta) else {
            exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str("metaclass must be a class".to_string())),
            );
            return MbValue::none();
        };
        class::mb_class_set_metaclass(
            MbValue::from_ptr(MbObject::new_str(class_name.clone())),
            MbValue::from_ptr(MbObject::new_str(meta_name)),
        );
        class::mb_class_finalize_definition_with_namespace(
            MbValue::from_ptr(MbObject::new_str(class_name)),
            dict,
        );
    }
    type_obj
}
