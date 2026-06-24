use super::super::rc::MbObject;
use super::super::value::MbValue;
/// importlib module for Mamba (#655).
///
/// Exposes Mamba's module import machinery as a Python-compatible API.
/// Wraps the runtime/module.rs functions: import_module, reload, find_spec.
use std::collections::HashMap;

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_import_module, mb_importlib_import_module);
dispatch_unary!(dispatch_reload, mb_importlib_reload);
dispatch_unary!(dispatch_find_spec, mb_importlib_find_spec);
dispatch_unary!(dispatch_find_loader, mb_importlib_find_loader);
dispatch_nullary!(dispatch_invalidate_caches, mb_importlib_invalidate_caches);

fn register_func(name: &str, addr: usize) -> (String, MbValue) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    (name.to_string(), MbValue::from_func(addr))
}

pub fn register() {
    let mut attrs = HashMap::new();

    // Callable functions — register as dispatched function values.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("import_module", dispatch_import_module as usize),
        ("reload", dispatch_reload as usize),
        ("find_spec", dispatch_find_spec as usize),
        ("find_loader", dispatch_find_loader as usize),
        ("invalidate_caches", dispatch_invalidate_caches as usize),
    ];
    for (name, addr) in dispatchers {
        let (k, v) = register_func(name, addr);
        attrs.insert(k, v);
    }

    // Submodule attributes — eagerly evaluate at register-time as dict
    // values (CPython exposes these as module objects with
    // `callable(importlib.util) == False`; building them eagerly matches
    // that parity bit).
    attrs.insert("util".into(), mb_importlib_util());
    attrs.insert("abc".into(), mb_importlib_abc());
    attrs.insert("machinery".into(), mb_importlib_machinery());
    attrs.insert("resources".into(), mb_importlib_resources());

    super::register_module("importlib", attrs);
}

/// importlib.import_module(name, package=None) -> module
/// Imports the named module, returning its namespace dict.
pub fn mb_importlib_import_module(name: MbValue) -> MbValue {
    // An empty module name is a ValueError (CPython: "Empty module name").
    if let Some(ptr) = name.as_ptr() {
        if let super::super::rc::ObjData::Str(ref s) = unsafe { &(*ptr).data } {
            if s.is_empty() {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("Empty module name".to_string())),
                );
                return MbValue::none();
            }
            // A relative name ('.mod') needs the `package` argument, which this
            // 1-arg form doesn't supply (CPython raises TypeError).
            if s.starts_with('.') {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "the 'package' argument is required to perform a relative import for '{s}'"
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    // Delegate to the runtime's mb_import
    super::super::module::mb_import(name)
}

/// importlib.reload(module) -> module
/// Reloads the module from disk. For now, returns the existing module.
pub fn mb_importlib_reload(module: MbValue) -> MbValue {
    // reload() requires a module object (mamba models modules as dicts —
    // ptr-backed). A primitive like reload(42) is a TypeError.
    if module.as_ptr().is_none() {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "reload() argument must be a module".to_string(),
            )),
        );
        return MbValue::none();
    }
    module // Stub: return the module unchanged
}

/// importlib.find_spec(name, package=None) -> ModuleSpec | None
/// Returns a ModuleSpec if the module can be found, None otherwise.
pub fn mb_importlib_find_spec(name: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let module_name = name
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Check if module is registered
    let found = super::super::module::MODULES.with(|mods| mods.borrow().contains_key(&module_name));

    if found {
        // Return a ModuleSpec-like dict
        let spec = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*spec).data {
                let mut map = lock.write().unwrap();
                map.insert("name".into(), name);
                map.insert("origin".into(), MbValue::none());
                map.insert("submodule_search_locations".into(), MbValue::none());
                map.insert("loader".into(), MbValue::none());
            }
        }
        MbValue::from_ptr(spec)
    } else {
        MbValue::none()
    }
}

/// importlib.find_loader(name, path=None) -> loader | None (deprecated in 3.12)
pub fn mb_importlib_find_loader(_name: MbValue) -> MbValue {
    MbValue::none()
}

/// importlib.invalidate_caches()
/// Clears all finder caches. No-op for Mamba's simple module system.
pub fn mb_importlib_invalidate_caches() -> MbValue {
    MbValue::none()
}

/// importlib.util submodule stub
pub fn mb_importlib_util() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            let (k, v) = register_func("find_spec", dispatch_find_spec as usize);
            map.insert(k.into(), v);
            map.insert("module_from_spec".into(), MbValue::none());
            map.insert("spec_from_file_location".into(), MbValue::none());
        }
    }
    MbValue::from_ptr(dict)
}

/// importlib.abc submodule stub
pub fn mb_importlib_abc() -> MbValue {
    let dict = MbObject::new_dict();
    MbValue::from_ptr(dict)
}

/// importlib.machinery submodule stub
pub fn mb_importlib_machinery() -> MbValue {
    let dict = MbObject::new_dict();
    MbValue::from_ptr(dict)
}

/// importlib.resources submodule stub
pub fn mb_importlib_resources() -> MbValue {
    let dict = MbObject::new_dict();
    MbValue::from_ptr(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_module_sys() {
        let name = MbValue::from_ptr(MbObject::new_str("sys".to_string()));
        // sys is registered by the stdlib, so this should return a module dict
        // (Module may not be registered in unit test context, so just check no panic)
        let _result = mb_importlib_import_module(name);
    }

    #[test]
    fn test_find_spec_unknown() {
        let name = MbValue::from_ptr(MbObject::new_str("nonexistent_module_xyz".to_string()));
        let spec = mb_importlib_find_spec(name);
        assert!(spec.is_none());
    }

    #[test]
    fn test_invalidate_caches() {
        let result = mb_importlib_invalidate_caches();
        assert!(result.is_none());
    }

    #[test]
    fn test_remaining_stubs_return_expected_shapes() {
        let m = MbValue::from_ptr(MbObject::new_str("anything".to_string()));
        assert!(mb_importlib_reload(m).is_ptr());
        assert!(mb_importlib_find_loader(m).is_none());
        for submod in [
            mb_importlib_util(),
            mb_importlib_abc(),
            mb_importlib_machinery(),
            mb_importlib_resources(),
        ] {
            assert!(submod.is_ptr(), "submodule stub should return a Dict ptr");
        }
    }
}
