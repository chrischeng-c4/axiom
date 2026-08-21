use super::super::rc::MbObject;
use super::super::value::MbValue;
/// copyreg module for mamba (#955).
///
/// CPython's `Lib/copyreg.py` provides functions used by pickle / copy to
/// register constructors and reduce functions, plus the `_reconstructor` /
/// `__newobj__` / `__newobj_ex__` shims that protocol 0/1/2/4 pickles call
/// back into to rebuild an instance. Previously every symbol here (besides
/// `pickle`, which fed `copy_mod.rs`'s `reduce_func_for_class`) was wired to
/// a no-op that just returned `None` — so e.g. `copyreg._reconstructor(C,
/// object, None)` silently returned `None` instead of a real `C` instance.
///
/// This implements the real `Lib/copyreg.py` semantics:
///   * `_reconstructor(cls, base, state)` = `base.__new__(cls)` (or
///     `base.__new__(cls, state)` + conditional `base.__init__(obj, state)`
///     when `base is not object`).
///   * `__newobj__(cls, *args)` / `__newobj_ex__(cls, args, kwargs)` =
///     `cls.__new__(cls, *args[, **kwargs])`.
///   * `pickle`/`constructor` validate callability like CPython.
///   * `add_extension` / `remove_extension` / `clear_extension_cache` /
///     `_extension_registry` / `_inverted_registry` / `_extension_cache` /
///     `dispatch_table` are real, mutable registries (real dict objects, not
///     no-op callables) with CPython's ValueError contracts.
///
/// `_slotnames` stays a best-effort no-op (deep `__slots__`/MRO
/// introspection, not needed for the reconstruction contract above).
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static REDUCE_REGISTRY: RefCell<HashMap<String, MbValue>> =
        RefCell::new(HashMap::new());
    static DISPATCH_TABLE: RefCell<Option<MbValue>> = RefCell::new(None);
    static EXTENSION_REGISTRY: RefCell<Option<MbValue>> = RefCell::new(None);
    static INVERTED_REGISTRY: RefCell<Option<MbValue>> = RefCell::new(None);
    static EXTENSION_CACHE: RefCell<Option<MbValue>> = RefCell::new(None);
}

fn dispatch_table() -> MbValue {
    DISPATCH_TABLE.with(|c| {
        c.borrow()
            .as_ref()
            .copied()
            .expect("copyreg.dispatch_table not registered")
    })
}

fn extension_registry() -> MbValue {
    EXTENSION_REGISTRY.with(|c| {
        c.borrow()
            .as_ref()
            .copied()
            .expect("copyreg._extension_registry not registered")
    })
}

fn inverted_registry() -> MbValue {
    INVERTED_REGISTRY.with(|c| {
        c.borrow()
            .as_ref()
            .copied()
            .expect("copyreg._inverted_registry not registered")
    })
}

fn extension_cache() -> MbValue {
    EXTENSION_CACHE.with(|c| {
        c.borrow()
            .as_ref()
            .copied()
            .expect("copyreg._extension_cache not registered")
    })
}

fn pending_exception() -> bool {
    super::super::exception::mb_has_exception().as_bool() == Some(true)
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let super::super::rc::ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Best-effort short description of a value for ValueError text (message
/// fidelity isn't contract-checked by any fixture; kept simple on purpose).
fn describe(v: MbValue) -> String {
    if let Some(s) = extract_str(v) {
        format!("'{s}'")
    } else if let Some(i) = v.as_int() {
        i.to_string()
    } else {
        "<object>".to_string()
    }
}

/// Describe a `(module, name)` key tuple as `(mod, name)` for error text.
fn describe_key(v: MbValue) -> String {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let super::super::rc::ObjData::Tuple(ref items) = (*ptr).data {
                if items.len() == 2 {
                    return format!("({}, {})", describe(items[0]), describe(items[1]));
                }
            }
        }
    }
    "<key>".to_string()
}

fn raise_type_error(msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_value_error(msg: String) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
}

fn new_tuple(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(items))
}

unsafe extern "C" fn d_noop(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// `pickle(ob_type, pickle_function, constructor_ob=None)`.
unsafe extern "C" fn d_pickle(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 2 {
        raise_type_error("pickle() missing required argument: 'pickle_function'");
        return MbValue::none();
    }
    let ob_type = unsafe { *args_ptr };
    let pickle_function = unsafe { *args_ptr.add(1) };
    if super::super::builtins::mb_callable(pickle_function).as_bool() != Some(true) {
        raise_type_error("reduction functions must be callable");
        return MbValue::none();
    }
    super::super::dict_ops::mb_dict_setitem(dispatch_table(), ob_type, pickle_function);
    if let Some(class_name) = super::super::class::resolve_class_name(ob_type) {
        unsafe {
            super::super::rc::retain_if_ptr(pickle_function);
        }
        REDUCE_REGISTRY.with(|registry| {
            let mut registry = registry.borrow_mut();
            if let Some(prev) = registry.insert(class_name, pickle_function) {
                unsafe {
                    super::super::rc::release_if_ptr(prev);
                }
            }
        });
    }
    if nargs >= 3 {
        let constructor_ob = unsafe { *args_ptr.add(2) };
        if !constructor_ob.is_none()
            && super::super::builtins::mb_callable(constructor_ob).as_bool() != Some(true)
        {
            raise_type_error("constructors must be callable");
        }
    }
    MbValue::none()
}

pub(crate) fn reduce_func_for_class(class_name: &str) -> Option<MbValue> {
    REDUCE_REGISTRY.with(|registry| registry.borrow().get(class_name).copied())
}

/// `constructor(object)`.
unsafe extern "C" fn d_constructor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 1 {
        raise_type_error("constructor() missing required argument: 'object'");
        return MbValue::none();
    }
    let obj = unsafe { *args_ptr };
    if super::super::builtins::mb_callable(obj).as_bool() != Some(true) {
        raise_type_error("constructors must be callable");
    }
    MbValue::none()
}

/// `_reconstructor(cls, base, state)`.
unsafe extern "C" fn d_reconstructor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 3 {
        raise_type_error("_reconstructor() takes exactly 3 arguments");
        return MbValue::none();
    }
    let cls = unsafe { *args_ptr };
    let base = unsafe { *args_ptr.add(1) };
    let state = unsafe { *args_ptr.add(2) };
    let base_name = super::super::class::resolve_class_name(base);
    let is_object_base = base_name.as_deref() == Some("object");
    let obj = if is_object_base {
        super::super::class::object_new_unbound(&[cls])
    } else {
        super::super::class::class_new_unbound(&[cls, state])
    };
    if pending_exception() {
        return MbValue::none();
    }
    if !is_object_base {
        if let Some(ref name) = base_name {
            let init_method = super::super::class::lookup_method(name, "__init__");
            if !init_method.is_none() {
                let args_list = MbValue::from_ptr(MbObject::new_list(vec![obj, state]));
                super::super::builtins::mb_call_spread(init_method, args_list);
                if pending_exception() {
                    return MbValue::none();
                }
            }
        }
    }
    obj
}

/// `__newobj__(cls, *args)` = `cls.__new__(cls, *args)`.
unsafe extern "C" fn d_newobj(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 1 {
        raise_type_error("__newobj__() missing required argument: 'cls'");
        return MbValue::none();
    }
    let items: Vec<MbValue> = (0..nargs).map(|i| unsafe { *args_ptr.add(i) }).collect();
    super::super::class::class_new_unbound(&items)
}

/// `__newobj_ex__(cls, args, kwargs)` = `cls.__new__(cls, *args, **kwargs)`.
unsafe extern "C" fn d_newobj_ex(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 3 {
        raise_type_error("__newobj_ex__() takes exactly 3 arguments");
        return MbValue::none();
    }
    let cls = unsafe { *args_ptr };
    let args_tuple = unsafe { *args_ptr.add(1) };
    let kwargs_dict = unsafe { *args_ptr.add(2) };
    let mut items = vec![cls];
    items.extend(super::super::builtins::extract_items(args_tuple));
    super::super::class::class_new_unbound_kwargs(&items, kwargs_dict)
}

/// `add_extension(module, name, code)`.
unsafe extern "C" fn d_add_extension(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 3 {
        raise_type_error("add_extension() takes exactly 3 arguments");
        return MbValue::none();
    }
    let module = unsafe { *args_ptr };
    let name = unsafe { *args_ptr.add(1) };
    let code_val = unsafe { *args_ptr.add(2) };
    let Some(code) = code_val.as_int() else {
        raise_type_error("an integer is required");
        return MbValue::none();
    };
    if !(1..=0x7fff_ffff).contains(&code) {
        raise_value_error("code out of range".to_string());
        return MbValue::none();
    }
    let key = new_tuple(vec![module, name]);
    let ext_registry = extension_registry();
    let inv_registry = inverted_registry();
    let code_mb = MbValue::from_int(code);
    let existing_code = super::super::dict_ops::mb_dict_get(ext_registry, key, MbValue::none());
    let existing_key = super::super::dict_ops::mb_dict_get(inv_registry, code_mb, MbValue::none());
    let redundant = existing_code.as_int() == Some(code)
        && !existing_key.is_none()
        && super::super::builtins::mb_eq(existing_key, key).as_bool() == Some(true);
    if redundant {
        return MbValue::none();
    }
    if !existing_code.is_none() {
        raise_value_error(format!(
            "key {} is already registered with code {}",
            describe_key(key),
            existing_code.as_int().unwrap_or(0)
        ));
        return MbValue::none();
    }
    if !existing_key.is_none() {
        raise_value_error(format!(
            "code {code} is already in use for key {}",
            describe_key(existing_key)
        ));
        return MbValue::none();
    }
    super::super::dict_ops::mb_dict_setitem(ext_registry, key, code_mb);
    super::super::dict_ops::mb_dict_setitem(inv_registry, code_mb, key);
    MbValue::none()
}

/// `remove_extension(module, name, code)`.
unsafe extern "C" fn d_remove_extension(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 3 {
        raise_type_error("remove_extension() takes exactly 3 arguments");
        return MbValue::none();
    }
    let module = unsafe { *args_ptr };
    let name = unsafe { *args_ptr.add(1) };
    let code_val = unsafe { *args_ptr.add(2) };
    let Some(code) = code_val.as_int() else {
        raise_type_error("an integer is required");
        return MbValue::none();
    };
    let key = new_tuple(vec![module, name]);
    let ext_registry = extension_registry();
    let inv_registry = inverted_registry();
    let code_mb = MbValue::from_int(code);
    let existing_code = super::super::dict_ops::mb_dict_get(ext_registry, key, MbValue::none());
    let existing_key = super::super::dict_ops::mb_dict_get(inv_registry, code_mb, MbValue::none());
    let matches = existing_code.as_int() == Some(code)
        && !existing_key.is_none()
        && super::super::builtins::mb_eq(existing_key, key).as_bool() == Some(true);
    if !matches {
        raise_value_error(format!(
            "key {} is not registered with code {code}",
            describe_key(key)
        ));
        return MbValue::none();
    }
    super::super::dict_ops::mb_dict_delitem(ext_registry, key);
    super::super::dict_ops::mb_dict_delitem(inv_registry, code_mb);
    let cache = extension_cache();
    if super::super::dict_ops::mb_dict_contains(cache, code_mb).as_bool() == Some(true) {
        super::super::dict_ops::mb_dict_delitem(cache, code_mb);
    }
    MbValue::none()
}

/// `clear_extension_cache()`.
unsafe extern "C" fn d_clear_extension_cache(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    super::super::dict_ops::mb_dict_clear(extension_cache());
    MbValue::none()
}

pub fn register() {
    let mut attrs = HashMap::new();

    let pickle = d_pickle as usize;
    let constructor = d_constructor as usize;
    let reconstructor = d_reconstructor as usize;
    let newobj = d_newobj as usize;
    let newobj_ex = d_newobj_ex as usize;
    let add_ext = d_add_extension as usize;
    let remove_ext = d_remove_extension as usize;
    let clear_cache = d_clear_extension_cache as usize;
    let noop = d_noop as usize;

    for addr in [
        pickle,
        constructor,
        reconstructor,
        newobj,
        newobj_ex,
        add_ext,
        remove_ext,
        clear_cache,
        noop,
    ] {
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    attrs.insert("pickle".to_string(), MbValue::from_func(pickle));
    attrs.insert("constructor".to_string(), MbValue::from_func(constructor));
    attrs.insert(
        "_reconstructor".to_string(),
        MbValue::from_func(reconstructor),
    );
    attrs.insert("__newobj__".to_string(), MbValue::from_func(newobj));
    attrs.insert("__newobj_ex__".to_string(), MbValue::from_func(newobj_ex));
    attrs.insert("add_extension".to_string(), MbValue::from_func(add_ext));
    attrs.insert(
        "remove_extension".to_string(),
        MbValue::from_func(remove_ext),
    );
    attrs.insert(
        "clear_extension_cache".to_string(),
        MbValue::from_func(clear_cache),
    );
    // Deep __slots__/MRO introspection; not needed for the reconstruction
    // contract this issue targets — kept as a best-effort no-op.
    attrs.insert("_slotnames".to_string(), MbValue::from_func(noop));

    // Real, mutable dict objects (previously mis-bound to no-op callables,
    // which made e.g. `type(copyreg.dispatch_table) is dict` false).
    let dispatch_table_dict = MbValue::from_ptr(MbObject::new_dict());
    unsafe {
        super::super::rc::retain_if_ptr(dispatch_table_dict);
    }
    DISPATCH_TABLE.with(|c| *c.borrow_mut() = Some(dispatch_table_dict));
    attrs.insert("dispatch_table".to_string(), dispatch_table_dict);

    let extension_registry_dict = MbValue::from_ptr(MbObject::new_dict());
    unsafe {
        super::super::rc::retain_if_ptr(extension_registry_dict);
    }
    EXTENSION_REGISTRY.with(|c| *c.borrow_mut() = Some(extension_registry_dict));
    attrs.insert("_extension_registry".to_string(), extension_registry_dict);

    let inverted_registry_dict = MbValue::from_ptr(MbObject::new_dict());
    unsafe {
        super::super::rc::retain_if_ptr(inverted_registry_dict);
    }
    INVERTED_REGISTRY.with(|c| *c.borrow_mut() = Some(inverted_registry_dict));
    attrs.insert("_inverted_registry".to_string(), inverted_registry_dict);

    let extension_cache_dict = MbValue::from_ptr(MbObject::new_dict());
    unsafe {
        super::super::rc::retain_if_ptr(extension_cache_dict);
    }
    EXTENSION_CACHE.with(|c| *c.borrow_mut() = Some(extension_cache_dict));
    attrs.insert("_extension_cache".to_string(), extension_cache_dict);

    super::register_module("copyreg", attrs);
}
