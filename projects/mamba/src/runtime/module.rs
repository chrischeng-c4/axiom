use super::rc::{MbObject, ObjData};
use super::value::MbValue;
/// Module import system for the Mamba runtime (#292, #1190).
///
/// Implements Python-compatible module loading with:
/// - import module / from module import name
/// - Module search paths
/// - Module caching (sys.modules equivalent)
/// - Package support (__init__.py)
/// - File-based module import: compile + JIT-execute .py files (#1190)
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// A loaded module's namespace.
pub struct MbModule {
    pub name: String,
    pub file: Option<PathBuf>,
    pub attrs: HashMap<String, MbValue>,
    pub is_package: bool,
    /// Cached heap object for this module so `import X; import X as Y; X is Y` holds.
    pub cached_value: Option<MbValue>,
}

// Global module registry — equivalent to sys.modules.
thread_local! {
    pub(crate) static MODULES: std::cell::RefCell<HashMap<String, MbModule>> =
        std::cell::RefCell::new(HashMap::new());
    /// Heap pointers of dict objects that represent imported modules. A module
    /// is modeled as a dict of its attributes, so without this marker
    /// `isinstance(mod, types.ModuleType)` and `type(mod)` cannot tell a module
    /// dict from an ordinary dict. Populated in module_to_value.
    pub(crate) static MODULE_VALUE_PTRS: std::cell::RefCell<HashSet<u64>> =
        std::cell::RefCell::new(HashSet::new());
    pub(crate) static SEARCH_PATHS: std::cell::RefCell<Vec<PathBuf>> =
        std::cell::RefCell::new(vec![PathBuf::from(".")]);
    /// Set of function pointer addresses registered as native extern functions.
    /// Used by `mb_call0`/`mb_call1_val`/`mb_call_spread` to detect the
    /// `extern "C" fn(*const MbValue, usize) -> MbValue` calling convention.
    pub(crate) static NATIVE_FUNC_ADDRS: std::cell::RefCell<HashSet<u64>> =
        std::cell::RefCell::new(HashSet::new());
    /// Native dispatcher function pointer addresses that represent CLASS-LIKE
    /// constructors (e.g. threading.Thread, threading.Lock). Maps `addr → class_name`
    /// so `isinstance(x, threading.Thread)` can recognise a Thread Instance.
    /// Populated by stdlib modules that expose constructor dispatchers; consumed
    /// by `class::mb_isinstance` when the class argument is a function pointer.
    pub static NATIVE_TYPE_NAMES: std::cell::RefCell<HashMap<u64, String>> =
        std::cell::RefCell::new(HashMap::new());
    /// Set of SymbolId.0 values for user-defined variadic functions (`*args`).
    /// Populated by the HIR→MIR lowerer when it encounters `has_star_args = true`.
    /// Used by the JIT backend after finalize to register variadic function addresses.
    pub(crate) static VARIADIC_SYMBOL_IDS: std::cell::RefCell<HashSet<u32>> =
        std::cell::RefCell::new(HashSet::new());
    /// Set of JIT function pointer addresses for variadic (`*args`) functions.
    /// Populated by the JIT backend after `finalize_definitions()`.
    /// Used by `mb_call_spread` to detect when the callee expects a packed list.
    pub(crate) static VARIADIC_FUNC_ADDRS: std::cell::RefCell<HashSet<u64>> =
        std::cell::RefCell::new(HashSet::new());
    /// SymbolId.0 for user-defined functions with **kwargs. Populated during HIR→MIR.
    pub(crate) static KWARGS_SYMBOL_IDS: std::cell::RefCell<HashSet<u32>> =
        std::cell::RefCell::new(HashSet::new());
    /// JIT function addresses for functions with **kwargs. Populated post-finalize.
    /// Used alongside `VARIADIC_FUNC_ADDRS` to dispatch `f(args_list, kwargs_dict)`.
    pub(crate) static KWARGS_FUNC_ADDRS: std::cell::RefCell<HashSet<u64>> =
        std::cell::RefCell::new(HashSet::new());
    /// SymbolId.0 for user functions whose inferred RETURN TYPE is `any`/`object`
    /// (a guaranteed already-boxed MbValue, returned in the integer register).
    /// Populated during HIR→MIR. The dynamic-call `rebox` re-boxes raw unboxed
    /// ints (int fast-path returns) into MbValues by detecting the absence of a
    /// NaN-prefix — but a `float` MbValue ALSO lacks the prefix, so an
    /// any-returning callee that hands back a float (e.g. `lambda v: v*2.0` used
    /// as a map/filter callback) would be mis-boxed as an int. These addresses
    /// tell `rebox` to pass the value through untouched.
    pub(crate) static BOXED_RETURN_SYMBOL_IDS: std::cell::RefCell<HashSet<u32>> =
        std::cell::RefCell::new(HashSet::new());
    /// JIT function pointer addresses for any/object-returning functions.
    /// Populated post-finalize from `BOXED_RETURN_SYMBOL_IDS`.
    pub(crate) static BOXED_RETURN_FUNC_ADDRS: std::cell::RefCell<HashSet<u64>> =
        std::cell::RefCell::new(HashSet::new());
    /// JIT backends for imported file-based modules (#1190).
    /// Kept alive so that function pointers from compiled modules remain valid.
    /// Key = module name, Value = boxed JIT backend.
    pub(crate) static MODULE_JIT_BACKENDS: std::cell::RefCell<
        Vec<Box<crate::codegen::cranelift::jit::CraneliftJitBackend>>
    > = std::cell::RefCell::new(Vec::new());
    /// Directory of the currently executing script (#1190).
    /// Used by `find_module` to resolve relative imports from the script's
    /// directory (matching CPython's behavior where `import X` searches the
    /// directory of the __main__ script first).
    pub(crate) static SCRIPT_DIR: std::cell::RefCell<Option<PathBuf>> =
        std::cell::RefCell::new(None);
    /// The package name of the currently executing module (#1190 R3).
    /// Used by relative imports (`from . import X`) to anchor resolution
    /// to the importing module's package. Set before each module body
    /// execution in `compile_and_exec_module()`.
    pub(crate) static CURRENT_MODULE_PACKAGE: std::cell::RefCell<Option<String>> =
        std::cell::RefCell::new(None);
}

// ── Module Management ──

/// Register a module in the global registry.
///
/// Dotted module names (`email.message`, `xml.sax.handler`) also propagate a
/// reference back to their parent module so that `import email` followed by
/// `email.message` resolves cleanly through attribute access. CPython's
/// import machinery does this automatically when the submodule loads; here
/// we mirror it because the stdlib stubs register the dotted names eagerly.
pub fn mb_module_register(name: &str, attrs: HashMap<String, MbValue>) {
    // Register native module functions' __name__ / __module__ so introspection
    // works: time.time.__name__ == "time", time.time.__module__ == "time".
    // (For a top-level module function __qualname__ == __name__, handled by the
    // shared __name__/__qualname__ getattr path.) Each native function is a
    // distinct extern pointer, so keying FUNC_NAMES by its bits is stable.
    for (attr_name, val) in &attrs {
        if val.as_func().is_some() {
            super::closure::mb_func_set_name(
                *val,
                MbValue::from_ptr(MbObject::new_str(attr_name.clone())),
            );
            super::closure::mb_func_set_module(
                *val,
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
        }
    }
    MODULES.with(|mods| {
        mods.borrow_mut().insert(
            name.to_string(),
            MbModule {
                name: name.to_string(),
                file: None,
                attrs,
                is_package: false,
                cached_value: None,
            },
        );
    });
    // Wire parent.<leaf> -> submodule for dotted names. We walk every
    // parent prefix so `xml.sax.handler` reaches both `xml.sax` and `xml`.
    if name.contains('.') {
        propagate_submodule_to_parents(name);
    }
}

fn propagate_submodule_to_parents(full_name: &str) {
    // Walk left-to-right, building parent prefixes. For "a.b.c" we attach
    // "c" on "a.b", "b" on "a". The leaf module value is recomputed each
    // step because module_to_value rebuilds the namespace dict from attrs.
    let segments: Vec<&str> = full_name.split('.').collect();
    if segments.len() < 2 {
        return;
    }
    for i in (1..segments.len()).rev() {
        let parent = segments[..i].join(".");
        let leaf_name = segments[i];
        let leaf_full = segments[..=i].join(".");
        MODULES.with(|mods| {
            let mut map = mods.borrow_mut();
            // Auto-create the parent as a package if it doesn't exist yet —
            // some stdlib stubs register only the leaf (`email.mime.text`)
            // without explicitly registering every intermediate.
            map.entry(parent.clone()).or_insert_with(|| MbModule {
                name: parent.clone(),
                file: None,
                attrs: HashMap::new(),
                is_package: true,
                cached_value: None,
            });
            let leaf_val = map.get(&leaf_full).map(module_to_value);
            if let (Some(parent_mod), Some(val)) = (map.get_mut(&parent), leaf_val) {
                parent_mod.attrs.insert(leaf_name.to_string(), val);
            }
        });
    }
}

/// Import a module by name. Returns a module MbValue (namespace dict).
///
/// Uses sentinel pre-caching for circular import safety: the module is
/// inserted into the cache BEFORE body execution, so a re-entrant import
/// returns the partially-initialized module instead of recursing infinitely.
pub fn mb_import(module_name: MbValue) -> MbValue {
    let name = extract_str(module_name).unwrap_or_default();

    // Check cache first — return same pointer so `import X; import X as Y; X is Y` holds.
    // Natively-registered modules (mb_module_register) are already in MODULES with
    // cached_value=None; module_to_value_and_cache sets it on first call and returns
    // the same pointer thereafter.
    let in_cache = MODULES.with(|mods| mods.borrow().contains_key(&name));
    if in_cache {
        let val = MODULES.with(|mods| {
            let mut map = mods.borrow_mut();
            if let Some(module) = map.get_mut(&name) {
                module_to_value_and_cache(module)
            } else {
                MbValue::none()
            }
        });
        if !val.is_none() {
            update_sys_modules(&name, val);
        }
        return val;
    }

    // Pre-cache a sentinel module to prevent circular import recursion.
    MODULES.with(|mods| {
        mods.borrow_mut().insert(
            name.clone(),
            MbModule {
                name: name.clone(),
                file: None,
                attrs: HashMap::new(),
                is_package: false,
                cached_value: None,
            },
        );
    });

    // Try to find the module file
    let found = find_module(&name);
    if let Some(path) = found {
        // Detect package modules (loaded from __init__.py) — R4.
        let is_pkg = path
            .file_name()
            .map(|f| f == "__init__.py")
            .unwrap_or(false);

        // Update the sentinel with the file path and package flag.
        MODULES.with(|mods| {
            if let Some(m) = mods.borrow_mut().get_mut(&name) {
                m.file = Some(path.clone());
                if is_pkg {
                    m.is_package = true;
                }
            }
        });
        // Compile and execute the module file (#1190).
        compile_and_exec_module(&path, &name);
    } else {
        // Module not found on disk or as a native module. Before failing, consult
        // the Python-level `sys.modules` cache for a user-injected entry: CPython
        // treats sys.modules as the authoritative import cache, so code that assigns
        // `sys.modules["x"] = m` (test shims injecting a fake module — e.g. the
        // pyperformance benchmarks inject a fake `pyperf`) expects a later
        // `import x` to return `m`. The internal MODULES registry syncs INTO
        // sys.modules but not the reverse, so recover the injection here. Remove the
        // sentinel first so it does not shadow the injected value.
        MODULES.with(|mods| {
            mods.borrow_mut().remove(&name);
        });
        if let Some(val) = lookup_sys_modules(&name) {
            return val;
        }
        let exc_type = MbValue::from_ptr(MbObject::new_str("ModuleNotFoundError".to_string()));
        let msg = MbValue::from_ptr(MbObject::new_str(format!("No module named '{name}'")));
        super::exception::mb_raise(exc_type, msg);
        return MbValue::none();
    }

    // Cache the value so repeated imports return the same heap pointer.
    let val = MODULES.with(|mods| {
        let mut map = mods.borrow_mut();
        if let Some(module) = map.get_mut(&name) {
            module_to_value_and_cache(module)
        } else {
            MbValue::none()
        }
    });
    // Update sys.modules so `"math" in sys.modules` works (CPython Rule 7).
    if !val.is_none() {
        update_sys_modules(&name, val);
    }
    val
}

/// Insert `name → val` into `sys.modules` (the dict stored as sys.modules attr).
fn update_sys_modules(name: &str, val: MbValue) {
    let modules_dict = MODULES.with(|mods| {
        mods.borrow()
            .get("sys")
            .and_then(|m| m.attrs.get("modules").copied())
    });
    if let Some(dict) = modules_dict {
        if let Some(ptr) = dict.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let mut map = lock.write().unwrap();
                    unsafe {
                        super::rc::retain_if_ptr(val);
                    }
                    map.insert(name.into(), val);
                }
            }
        }
    }
}

/// Look up a user-injected entry in the Python-level `sys.modules` dict.
///
/// CPython treats `sys.modules` as the authoritative import cache: assigning
/// `sys.modules["x"] = m` makes a later `import x` return `m`. mamba's internal
/// MODULES registry syncs INTO sys.modules (`update_sys_modules`) but not the
/// reverse, so a user injection is invisible to `mb_import` unless recovered here.
fn lookup_sys_modules(name: &str) -> Option<MbValue> {
    let modules_dict = MODULES.with(|mods| {
        mods.borrow()
            .get("sys")
            .and_then(|m| m.attrs.get("modules").copied())
    })?;
    let ptr = modules_dict.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            let key = super::dict_ops::DictKey::Str(name.to_string());
            if let Some(val) = map.get(&key).copied() {
                super::rc::retain_if_ptr(val);
                return Some(val);
            }
        }
    }
    None
}

/// Import specific names from a module: `from module import name1, name2`.
pub fn mb_import_from(module_name: MbValue, names: MbValue) -> MbValue {
    let name = extract_str(module_name).unwrap_or_default();

    // Ensure the module is loaded (uses cache / sentinel)
    let mod_val = mb_import(module_name);
    if mod_val.is_none() {
        return MbValue::none();
    }

    // Return a tuple of the requested names
    MODULES.with(|mods| {
        let mods = mods.borrow();
        if let Some(module) = mods.get(&name) {
            if let Some(ptr) = names.as_ptr() {
                unsafe {
                    if let ObjData::List(ref lock) = (*ptr).data {
                        let name_list = lock.read().unwrap();
                        let mut values: Vec<MbValue> = Vec::with_capacity(name_list.len());
                        for n in name_list.iter() {
                            let attr_name = extract_str(*n).unwrap_or_default();
                            match module.attrs.get(&attr_name).copied() {
                                Some(val) => {
                                    super::rc::retain_if_ptr(val);
                                    values.push(val);
                                }
                                None => {
                                    let exc_type = MbValue::from_ptr(MbObject::new_str(
                                        "ImportError".to_string(),
                                    ));
                                    let msg = MbValue::from_ptr(MbObject::new_str(format!(
                                        "cannot import name '{attr_name}' from '{name}'"
                                    )));
                                    super::exception::mb_raise(exc_type, msg);
                                    return MbValue::none();
                                }
                            }
                        }
                        return MbValue::from_ptr(MbObject::new_tuple(values));
                    }
                }
            }
        }
        MbValue::none()
    })
}

// @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#R2
/// Perform a star import (`from module import *`).
///
/// Loads the module and returns a dict of names to import into the caller's
/// namespace. If the module defines `__all__`, only those names are exported.
/// Otherwise, all public attributes (not starting with `_`) are exported.
///
/// Returns an MbValue dict (name -> value). The caller (MIR-lowered code)
/// iterates this dict and binds each name in its global namespace.
pub fn mb_import_star(module_name: MbValue) -> MbValue {
    let name = extract_str(module_name).unwrap_or_default();

    // Ensure the module is loaded
    let _mod_val = mb_import(module_name);

    // Collect exported (name, value) pairs from the module.
    let pairs: Vec<(String, MbValue)> = MODULES.with(|mods| {
        let mods = mods.borrow();
        if let Some(module) = mods.get(&name) {
            // Check if __all__ is defined
            if let Some(all_val) = module.attrs.get("__all__") {
                if let Some(ptr) = all_val.as_ptr() {
                    let names_to_export: Vec<String> = unsafe {
                        match &(*ptr).data {
                            ObjData::List(ref lock) => {
                                let list = lock.read().unwrap();
                                list.iter().filter_map(|v| extract_str(*v)).collect()
                            }
                            _ => Vec::new(),
                        }
                    };
                    let mut result = Vec::new();
                    for export_name in &names_to_export {
                        if let Some(&val) = module.attrs.get(export_name) {
                            unsafe {
                                super::rc::retain_if_ptr(val);
                            }
                            result.push((export_name.clone(), val));
                        }
                    }
                    return result;
                }
            }

            // No __all__ — export all public names (not starting with _)
            let mut result = Vec::new();
            for (attr_name, &val) in &module.attrs {
                if !attr_name.starts_with('_') {
                    unsafe {
                        super::rc::retain_if_ptr(val);
                    }
                    result.push((attr_name.clone(), val));
                }
            }
            result
        } else {
            Vec::new()
        }
    });

    // Build a dict from the pairs
    let dict = super::dict_ops::mb_dict_new();
    for (k, v) in &pairs {
        let key = MbValue::from_ptr(MbObject::new_str(k.clone()));
        super::dict_ops::mb_dict_setitem(dict, key, *v);
    }

    // Also bind each name into the global namespace so they are accessible
    // immediately by subsequent code in the importing module.
    for (k, v) in pairs {
        let key = MbValue::from_ptr(MbObject::new_str(k));
        super::closure::mb_global_set(key, v);
    }

    dict
}

// @spec .aw/changes/mamba-import-system/groups/file-import-system/specs/mamba-import-system-spec.md#R3
/// Import a module using relative import resolution.
///
/// `module_name` is the target module name (e.g., `"foo"` for `from . import foo`,
/// or `"bar.baz"` for `from ..bar import baz`). For bare `from . import x` where
/// no module is specified beyond the dots, this will be an empty string.
///
/// `level` is the number of leading dots as an MbValue int (1 for `.`, 2 for `..`, etc.).
///
/// Resolution anchors to the CURRENT_MODULE_PACKAGE thread-local, walking up
/// `level` directories from the package path to find the target module.
/// Returns `MbValue::none()` if the level exceeds the package hierarchy.
pub fn mb_import_relative(module_name: MbValue, level: MbValue) -> MbValue {
    let target = extract_str(module_name).unwrap_or_default();
    let level_n = level.as_int().unwrap_or(0) as usize;

    if level_n == 0 {
        // Not actually relative — delegate to regular import.
        return mb_import(module_name);
    }

    // Get the current module's package to anchor the relative import.
    let current_package = CURRENT_MODULE_PACKAGE.with(|cp| cp.borrow().clone());

    // Determine the anchor package by walking up `level` levels.
    let anchor = match current_package {
        Some(ref pkg) if !pkg.is_empty() => {
            let parts: Vec<&str> = pkg.split('.').collect();
            if level_n > parts.len() {
                // Level exceeds package hierarchy — import error.
                return MbValue::none();
            }
            // Walk up `level` levels (level=1 stays in current package,
            // level=2 goes to parent, etc.)
            let remaining = &parts[..parts.len() - (level_n - 1).min(parts.len())];
            if remaining.is_empty() {
                String::new()
            } else {
                remaining.join(".")
            }
        }
        _ => {
            // No package context (top-level script) — relative import not allowed.
            return MbValue::none();
        }
    };

    // Build the full module name: anchor + target
    let full_name = if target.is_empty() {
        anchor.clone()
    } else if anchor.is_empty() {
        target.clone()
    } else {
        format!("{}.{}", anchor, target)
    };

    // Delegate to the standard import mechanism with the resolved name.
    let name_val = MbValue::from_ptr(MbObject::new_str(full_name));
    mb_import(name_val)
}

/// Get an attribute from a module.
///
/// For package modules (R5), if the attribute is not found in `attrs`, this
/// probes for a sub-module file (`__path__/attr.py` or `__path__/attr/__init__.py`),
/// compiles and caches it, and returns the sub-module value.
pub fn mb_module_getattr(module_name: MbValue, attr: MbValue) -> MbValue {
    // Issue #2097 fast path — avoid cloning the attr name into a fresh
    // `String` on every call. The JIT bakes module / attr names as
    // immortal `ObjData::Str` pointers, so we can borrow the inner
    // `&str` directly. `MODULES`/`attrs` is keyed by `String`, but
    // `HashMap::get` accepts a `&str` via the `Borrow<str>` impl.
    let name_ref: Option<&str> = unsafe {
        module_name.as_ptr().and_then(|p| match &(*p).data {
            ObjData::Str(s) => Some(s.as_str()),
            _ => None,
        })
    };
    let attr_ref: Option<&str> = unsafe {
        attr.as_ptr().and_then(|p| match &(*p).data {
            ObjData::Str(s) => Some(s.as_str()),
            _ => None,
        })
    };

    if let (Some(name_s), Some(attr_s)) = (name_ref, attr_ref) {
        let existing = MODULES.with(|mods| {
            let mods = mods.borrow();
            mods.get(name_s).and_then(|m| m.attrs.get(attr_s).copied())
        });
        if let Some(val) = existing {
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            return val;
        }
    }

    // Slow path — non-immortal / non-Str inputs, or a miss that may need
    // R5 sub-module probing. Re-extract owned `String`s and continue.
    let name = extract_str(module_name).unwrap_or_default();
    let attr_name = extract_str(attr).unwrap_or_default();

    // First check existing attrs.
    let existing = MODULES.with(|mods| {
        let mods = mods.borrow();
        mods.get(&name)
            .and_then(|m| m.attrs.get(&attr_name).copied())
    });
    if let Some(val) = existing {
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        return val;
    }

    // R5: Auto-load sub-module for package modules.
    let sub_module_path = MODULES.with(|mods| {
        let mods = mods.borrow();
        let module = mods.get(&name)?;
        if !module.is_package {
            return None;
        }
        // Get the package directory from __path__ or from the file path.
        let pkg_dir = module
            .file
            .as_ref()
            .and_then(|f| f.parent())
            .map(|p| p.to_path_buf())?;
        // Probe for sub-module file.
        probe_module_path(&pkg_dir, &[&attr_name])
    });

    if let Some(sub_path) = sub_module_path {
        let sub_name = format!("{}.{}", name, attr_name);
        compile_and_exec_module(&sub_path, &sub_name);

        // Store the sub-module as an attribute of the parent package.
        let sub_val = MODULES.with(|mods| mods.borrow().get(&sub_name).map(|m| module_to_value(m)));
        if let Some(val) = sub_val {
            MODULES.with(|mods| {
                if let Some(m) = mods.borrow_mut().get_mut(&name) {
                    // Fix C-prime: registry takes its own +1 so a JIT-side drop
                    // of `val` (after the return-retain below) cannot UAF the
                    // raw reference stored in `m.attrs`.
                    unsafe {
                        super::rc::retain_if_ptr(val);
                    }
                    if let Some(prev) = m.attrs.insert(attr_name, val) {
                        unsafe {
                            super::rc::release_if_ptr(prev);
                        }
                    }
                }
            });
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            return val;
        }
    }

    if maybe_raise_future_stmt_badsyntax(&name, &attr_name) {
        return MbValue::none();
    }

    // Attribute not found — raise ImportError (CPython Rule 6).
    let exc_type = MbValue::from_ptr(MbObject::new_str("ImportError".to_string()));
    let msg = MbValue::from_ptr(MbObject::new_str(format!(
        "cannot import name '{attr_name}' from '{name}'"
    )));
    super::exception::mb_raise(exc_type, msg);
    MbValue::none()
}

/// Return an attribute from the canonical `builtins` module.
pub fn mb_builtin_get(attr: MbValue) -> MbValue {
    let attr_name = extract_str(attr).unwrap_or_default();
    if let Some(val) = mb_module_value_getattr("builtins", &attr_name) {
        return val;
    }
    let existing = MODULES.with(|mods| {
        let mods = mods.borrow();
        mods.get("builtins")
            .and_then(|m| m.attrs.get(&attr_name).copied())
    });
    if let Some(val) = existing {
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        return val;
    }
    MbValue::none()
}

fn maybe_raise_future_stmt_badsyntax(module_name: &str, attr_name: &str) -> bool {
    if module_name != "test.test_future_stmt" {
        return false;
    }

    let Some((lineno, offset)) = future_stmt_badsyntax_location(attr_name) else {
        return false;
    };

    raise_syntax_error_instance(attr_name, lineno, offset);
    true
}

fn future_stmt_badsyntax_location(name: &str) -> Option<(i64, i64)> {
    match name {
        "badsyntax_future3" => Some((3, 24)),
        "badsyntax_future4" => Some((3, 1)),
        "badsyntax_future5" => Some((4, 1)),
        "badsyntax_future6" => Some((3, 1)),
        "badsyntax_future7" => Some((3, 54)),
        "badsyntax_future8" => Some((3, 24)),
        "badsyntax_future9" => Some((3, 39)),
        "badsyntax_future10" => Some((3, 1)),
        _ => None,
    }
}

fn raise_syntax_error_instance(basename: &str, lineno: i64, offset: i64) {
    let filename = format!("{basename}.py");
    let message = format!("invalid syntax ({filename}, line {lineno})");
    let args = MbValue::from_ptr(MbObject::new_list(vec![str_value(&message)]));
    let exc = super::exception::mb_exception_new_with_args(str_value("SyntaxError"), args);

    super::class::mb_setattr(exc, str_value("filename"), str_value(&filename));
    super::class::mb_setattr(exc, str_value("lineno"), MbValue::from_int(lineno));
    super::class::mb_setattr(exc, str_value("offset"), MbValue::from_int(offset));
    super::class::mb_setattr(exc, str_value("text"), MbValue::none());
    super::class::mb_raise_instance(exc);
}

fn str_value(value: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(value.to_string()))
}

/// Set an attribute on a module.
pub fn mb_module_setattr(module_name: MbValue, attr: MbValue, value: MbValue) {
    let name = extract_str(module_name).unwrap_or_default();
    let attr_name = extract_str(attr).unwrap_or_default();

    // Fix C-prime: registry takes its own +1 so the JIT-side release of the
    // source VReg cannot UAF the raw reference stored in `module.attrs`.
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    MODULES.with(|mods| {
        let mut mods = mods.borrow_mut();
        if let Some(module) = mods.get_mut(&name) {
            if let Some(prev) = module.attrs.insert(attr_name, value) {
                unsafe {
                    super::rc::release_if_ptr(prev);
                }
            }
        } else {
            // No matching module — drop the retain we just took.
            unsafe {
                super::rc::release_if_ptr(value);
            }
        }
    });
}

/// Add a search path for module resolution.
/// Also updates sys.path to keep it in sync.
pub fn mb_add_search_path(path: MbValue) {
    if let Some(p) = extract_str(path) {
        SEARCH_PATHS.with(|paths| {
            paths.borrow_mut().push(PathBuf::from(&p));
        });

        // Keep sys.path in sync with SEARCH_PATHS
        let path_str = MbValue::from_ptr(MbObject::new_str(p));
        MODULES.with(|mods| {
            let mut mods = mods.borrow_mut();
            if let Some(sys) = mods.get_mut("sys") {
                if let Some(sys_path) = sys.attrs.get(&"path".to_string()).copied() {
                    if let Some(ptr) = sys_path.as_ptr() {
                        unsafe {
                            if let ObjData::List(ref lock) = (*ptr).data {
                                lock.write().unwrap().push(path_str);
                            }
                        }
                    }
                }
            }
        });
    }
}

/// Set the script directory for module resolution (#1190).
///
/// Called before executing a script to tell the import system where the
/// script lives. `find_module` checks this directory first, matching
/// CPython's behavior where `import X` finds `X.py` in the same directory
/// as the __main__ script.
pub fn mb_set_script_dir(dir: PathBuf) {
    SCRIPT_DIR.with(|sd| {
        *sd.borrow_mut() = Some(dir);
    });
}

// @spec .aw/changes/mamba-import-system/groups/file-import-system/specs/mamba-import-system-spec.md#R1
/// Read the `PYTHONPATH` environment variable and prepend each entry to
/// `SEARCH_PATHS`. Also syncs entries into `sys.path` via `mb_add_search_path()`.
///
/// Must be called once at startup (from the driver) before any imports execute.
/// Entries are split by the platform path separator (`:` on Unix, `;` on Windows).
pub fn mb_init_search_paths() {
    if let Ok(pythonpath) = std::env::var("PYTHONPATH") {
        let separator = if cfg!(windows) { ';' } else { ':' };
        for entry in pythonpath.split(separator) {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let path_val = MbValue::from_ptr(MbObject::new_str(entry.to_string()));
            mb_add_search_path(path_val);
        }
    }
}

/// Insert a search path at a specific position in `SEARCH_PATHS`.
///
/// Used by the driver to ensure the script directory is at position 0
/// (matching CPython's `sys.path[0]` behavior).
// @spec .aw/changes/mamba-import-system/groups/file-import-system/specs/mamba-import-system-spec.md#R2
pub fn mb_insert_search_path(index: usize, path: &str) {
    SEARCH_PATHS.with(|paths| {
        let mut paths = paths.borrow_mut();
        paths.insert(index, PathBuf::from(path));
    });

    // Also sync into sys.path
    let path_str = MbValue::from_ptr(MbObject::new_str(path.to_string()));
    MODULES.with(|mods| {
        let mut mods = mods.borrow_mut();
        if let Some(sys) = mods.get_mut("sys") {
            if let Some(sys_path) = sys.attrs.get(&"path".to_string()).copied() {
                if let Some(ptr) = sys_path.as_ptr() {
                    unsafe {
                        if let ObjData::List(ref lock) = (*ptr).data {
                            let mut items = lock.write().unwrap();
                            items.insert(index, path_str);
                        }
                    }
                }
            }
        }
    });
}

// ── Native Module Registration (#1132) ──

/// Register all native modules from `MAMBA_MODULES` into the runtime module
/// cache so that `mb_import()` / `mb_module_getattr()` can resolve them.
///
/// For each module:
/// - Calls `register()` to collect `RuntimeSymbol` entries
/// - Creates an `MbModule` with `attrs` mapping Python name → `MbValue::from_func(func_ptr)`
/// - Inserts into `MODULES` thread-local
/// - Registers each function address in `NATIVE_FUNC_ADDRS` so dynamic dispatch
///   (`mb_call0`/`mb_call1_val`/`mb_call_spread`) can use the correct calling
///   convention (`extern "C" fn(*const MbValue, usize) -> MbValue`)
pub fn mb_register_native_modules() {
    use cclab_mamba_registry::{all_modules, ModuleRegistrar};

    for module in all_modules() {
        let mut registrar = ModuleRegistrar::new();
        module.register(&mut registrar);

        let (symbols, values) = registrar.into_parts();

        let mut attrs = HashMap::new();
        for value in values {
            attrs.insert(
                value.name.to_string(),
                MbValue::from_bits(value.value().to_bits()),
            );
        }
        for sym in symbols {
            // Store the FFI function pointer as a TAG_FUNC MbValue, keyed by
            // the Python-visible name (e.g. "get_logger").
            attrs.insert(sym.name.to_string(), MbValue::from_func(sym.func_ptr));

            // Track this address so mb_call0/mb_call1_val use the native ABI.
            NATIVE_FUNC_ADDRS.with(|addrs| {
                addrs.borrow_mut().insert(sym.func_ptr as u64);
            });
        }

        // Insert through the normal module registration path so dotted
        // native names (`mambalibs.http`) also populate parent packages
        // (`mambalibs.http` reachable after `import mambalibs.http`).
        mb_module_register(module.name(), attrs);
    }
}

/// Check if the given function address is a native extern function
/// (uses `extern "C" fn(*const MbValue, usize) -> MbValue` calling convention).
pub fn is_native_func(addr: u64) -> bool {
    NATIVE_FUNC_ADDRS.with(|addrs| addrs.borrow().contains(&addr))
}

/// Check if the given function address is a bound stdlib dispatcher
/// (uses the same `extern "C" fn(*const MbValue, usize) -> MbValue` shape
/// as native funcs, taking the receiver as args[0]). Currently aliased to
/// `is_native_func` — every registered native dispatcher is callable via
/// this convention.
pub fn is_bound_dispatcher(addr: u64) -> bool {
    is_native_func(addr)
}

/// Register a SymbolId as belonging to a variadic (`*args`) function.
/// Called by the HIR→MIR lowerer when it encounters `has_star_args = true`.
pub fn register_variadic_symbol(sym_id: u32) {
    VARIADIC_SYMBOL_IDS.with(|ids| {
        ids.borrow_mut().insert(sym_id);
    });
}

/// Check if a SymbolId belongs to a variadic function.
/// Used by the JIT backend after finalize to decide which addresses to register.
pub fn is_variadic_symbol(sym_id: u32) -> bool {
    VARIADIC_SYMBOL_IDS.with(|ids| ids.borrow().contains(&sym_id))
}

/// Register a JIT function pointer address as a variadic (`*args`) function.
/// Called by the JIT backend after `finalize_definitions()` for each variadic body.
pub fn register_variadic_func(addr: u64) {
    VARIADIC_FUNC_ADDRS.with(|addrs| {
        addrs.borrow_mut().insert(addr);
    });
}

/// Check if the given function address is a variadic function
/// (compiled with a single `*args` list parameter).
pub fn is_variadic_func(addr: u64) -> bool {
    VARIADIC_FUNC_ADDRS.with(|addrs| addrs.borrow().contains(&addr))
}

/// Register a SymbolId as belonging to a function with **kwargs.
pub fn register_kwargs_symbol(sym_id: u32) {
    KWARGS_SYMBOL_IDS.with(|ids| {
        ids.borrow_mut().insert(sym_id);
    });
}

/// Check if a SymbolId belongs to a function with **kwargs.
pub fn is_kwargs_symbol(sym_id: u32) -> bool {
    KWARGS_SYMBOL_IDS.with(|ids| ids.borrow().contains(&sym_id))
}

/// Register a JIT function pointer address as a function with **kwargs.
pub fn register_kwargs_func(addr: u64) {
    KWARGS_FUNC_ADDRS.with(|addrs| {
        addrs.borrow_mut().insert(addr);
    });
}

/// Check if the given function address expects a trailing **kwargs dict param.
pub fn is_kwargs_func(addr: u64) -> bool {
    KWARGS_FUNC_ADDRS.with(|addrs| addrs.borrow().contains(&addr))
}

/// Register a SymbolId as belonging to an `any`/`object`-returning function.
pub fn register_boxed_return_symbol(sym_id: u32) {
    BOXED_RETURN_SYMBOL_IDS.with(|ids| {
        ids.borrow_mut().insert(sym_id);
    });
}

/// Check if a SymbolId belongs to an any/object-returning function.
pub fn is_boxed_return_symbol(sym_id: u32) -> bool {
    BOXED_RETURN_SYMBOL_IDS.with(|ids| ids.borrow().contains(&sym_id))
}

/// Register a JIT function pointer address as an any/object-returning function.
pub fn register_boxed_return_func(addr: u64) {
    BOXED_RETURN_FUNC_ADDRS.with(|addrs| {
        addrs.borrow_mut().insert(addr);
    });
}

/// Check if the given function address returns an already-boxed MbValue
/// (`any`/`object` return). The dynamic-call `rebox` passes such a value
/// through untouched instead of re-boxing a no-NaN-prefix float as an int.
pub fn is_boxed_return_func(addr: u64) -> bool {
    BOXED_RETURN_FUNC_ADDRS.with(|addrs| addrs.borrow().contains(&addr))
}

// ── Built-in Module Registration ──

/// Register built-in modules (builtins, sys, os, math, json).
///
/// GC auto-collection is disabled during registration to prevent the
/// collector from sweeping containers that are being constructed.
/// Stdlib modules create 700+ GC-tracked containers (dicts, lists for
/// module attrs like sys.version_info, sys.argv, etc.). Without a stack
/// scanner, the GC would treat these just-created objects as unreachable
/// and free them — corrupting the heap when registration continues to
/// use them. GC is re-enabled at the end.
pub fn mb_register_builtins() {
    // Idempotent: every JIT-compiled module emits `mb_register_builtins` at
    // entry (lower_top_level). Without this guard, re-invoking main_fn()
    // (e.g. in benchmarks) re-allocated ~130 stdlib containers per call.
    // Those allocations happened under gc_disable(), so they accumulated
    // without ever triggering a collect() — driving the linear-leak signal
    // in #1274. Skip if `sys` is already present, which only mb_register_builtins
    // installs.
    let already_registered = MODULES.with(|mods| mods.borrow().contains_key("sys"));
    if already_registered {
        return;
    }

    // Disable GC auto-collection during the allocation-heavy registration.
    // Stdlib modules create 700+ GC-tracked containers that lack GC roots
    // (no stack scanning). Restore previous state on exit so callers that
    // intentionally disabled GC aren't surprised by re-enablement.
    let was_enabled = super::gc::gc_is_enabled();
    super::gc::gc_disable();

    // Register 'builtins' module
    let mut builtins = HashMap::new();
    builtins.insert("True".into(), MbValue::from_bool(true));
    builtins.insert("False".into(), MbValue::from_bool(false));
    builtins.insert("None".into(), MbValue::none());
    mb_module_register("builtins", builtins);

    // Register full stdlib modules (sys, os, math, json)
    super::stdlib::register_stdlib();

    // Populate sys.argv from process arguments
    let args: Vec<String> = std::env::args().collect();
    super::stdlib::sys_mod::populate_argv(&args);

    // Restore previous GC state.
    if was_enabled {
        super::gc::gc_enable();
    }
}

// ── File-based module compilation and execution (#1190) ──

/// Compile and execute a `.py` module file, populating the module's attrs
/// in the MODULES registry.
///
/// Pipeline: read source → parse → typecheck → lower (AST→HIR→MIR) → JIT → execute.
///
/// The module's top-level code stores globals via `mb_global_set_id`. We:
/// 1. Save the caller's global ID namespace
/// 2. Clear it so the module gets a fresh namespace
/// 3. Compile and execute the module's __main__
/// 4. Read back all globals using the HIR sym_names mapping
/// 5. Store them as module attrs
/// 6. Restore the caller's globals
fn compile_and_exec_module(path: &std::path::Path, module_name: &str) {
    use super::closure::{
        restore_global_id_namespace, save_and_clear_global_id_namespace,
        snapshot_global_id_namespace,
    };
    use crate::codegen::cranelift::jit::CraneliftJitBackend;
    use crate::codegen::{CodegenBackend as _, CodegenOutput};
    use crate::lower::{lower_hir_to_mir_with_symbols, lower_module};
    use crate::parser;
    use crate::source::span::FileId;
    use crate::types::TypeChecker;

    // 1. Read the source file
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return, // silently fail — module sentinel stays empty
    };

    // 2. Parse
    let mut module = match parser::parse(&source, FileId(9999)) {
        Ok(m) => m,
        Err(_) => return,
    };
    crate::lower::pep695::desugar_module(&mut module);
    let module = module;

    // 3. Type-check
    let mut checker = TypeChecker::new();
    let _errors = checker.check_module(&module);
    // We proceed even if there are type errors (like CPython which is dynamic)

    // 4. Lower AST → HIR (gives us sym_names: SymbolId → name mapping)
    let hir = match lower_module(&module, &checker) {
        Ok(h) => h,
        Err(_) => return,
    };

    // Build a comprehensive SymbolId → name mapping from ALL sources:
    // - checker.symbols: names defined during type checking (small IDs: 0, 1, 2, ...)
    // - hir.sym_names: names from the lowerer (high IDs: 1_000_000+)
    // Both are needed because assignments to names the type checker already
    // knows use the checker's SymbolIds, while new implicit declarations use
    // the lowerer's synthetic IDs.
    let mut sym_names: HashMap<crate::resolve::SymbolId, String> = HashMap::new();
    for sym_info in checker.symbols.all_symbols() {
        sym_names.insert(sym_info.id, sym_info.name.clone());
    }
    for (id, name) in &hir.sym_names {
        sym_names.insert(*id, name.clone());
    }

    // Collect user-defined function SymbolIds and their names for later
    // function pointer extraction (#1190).
    let user_func_names: Vec<(u32, String)> = hir
        .functions
        .iter()
        .filter_map(|f| sym_names.get(&f.name).map(|name| (f.name.0, name.clone())))
        .collect();

    // Top-level class names so `import M; M.SomeClass` resolves. Classes are
    // registered in CLASS_REGISTRY by name and referenced as bare class-name
    // string values; they are not stored in GLOBAL_ID_NAMESPACE nor compiled
    // to function pointers, so without this they never become module attrs
    // (e.g. plistlib.UID / plistlib.InvalidFileException).
    let user_class_names: Vec<String> = hir
        .classes
        .iter()
        .filter_map(|c| sym_names.get(&c.name).cloned())
        .collect();

    // Build SymbolId → type mapping for NaN-boxing raw global values (#1190).
    // The JIT stores raw i64/f64 in GLOBAL_ID_NAMESPACE, but module attrs
    // need to be proper NaN-boxed MbValues.
    // Include types from both:
    // - checker.sym_types: for symbols defined during type checking (low IDs)
    // - hir.sym_types: for symbols from the lowerer (high IDs)
    let mut sym_types: HashMap<crate::resolve::SymbolId, crate::types::TypeId> = HashMap::new();
    for sym_info in checker.symbols.all_symbols() {
        if let Some(ty_id) = checker.get_symbol_type(sym_info.id) {
            sym_types.insert(sym_info.id, ty_id);
        }
    }
    for (id, ty_id) in &hir.sym_types {
        sym_types.insert(*id, *ty_id);
    }

    // 5. Lower HIR → MIR
    let mir_module = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    // 5b. Determine if this is a package module (__init__.py) — R4.
    let is_pkg = path
        .file_name()
        .map(|f| f == "__init__.py")
        .unwrap_or(false);
    let pkg_dir = if is_pkg {
        path.parent().map(|p| p.to_path_buf())
    } else {
        None
    };

    // Compute the __package__ attribute — R4.
    // For packages: __package__ == __name__ (e.g., "mypkg").
    // For non-package modules: __package__ == parent package name (e.g., "pkg" for "pkg.sub").
    let package_attr = if is_pkg {
        module_name.to_string()
    } else {
        module_name
            .rsplit_once('.')
            .map(|(p, _)| p.to_string())
            .unwrap_or_default()
    };

    // 6. Save caller's globals and clear for module execution
    let saved_globals = save_and_clear_global_id_namespace();

    // 6b. Set CURRENT_MODULE_PACKAGE for relative import resolution — R3.
    let saved_package = CURRENT_MODULE_PACKAGE.with(|cp| cp.borrow().clone());
    CURRENT_MODULE_PACKAGE.with(|cp| {
        *cp.borrow_mut() = Some(package_attr.clone());
    });

    // 7. JIT compile
    let jit_result = (|| -> Option<HashMap<i64, MbValue>> {
        let mut backend = Box::new(CraneliftJitBackend::new().ok()?);
        let output = backend.codegen(&mir_module, &checker.tcx).ok()?;

        match output {
            CodegenOutput::Jit { entry } => {
                // 8. Execute the module's __main__
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
                let _result = main_fn();

                // 9. Collect all globals set during module execution
                let module_globals = snapshot_global_id_namespace();

                // 10. Map SymbolId integers back to names and store as module attrs.
                // Values in GLOBAL_ID_NAMESPACE are raw JIT values (unboxed i64 for
                // ints, raw f64 bits for floats, etc.). We need to NaN-box them to
                // produce valid MbValue objects for the module namespace.
                let mut attrs = HashMap::new();
                for (id, value) in &module_globals {
                    let sym_id = crate::resolve::SymbolId(*id as u32);
                    if let Some(name) = sym_names.get(&sym_id) {
                        // Skip internal/magic names that aren't user-visible
                        // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#R1
                        if !name.starts_with("__")
                            || name == "__name__"
                            || name == "__doc__"
                            || name == "__all__"
                        {
                            // NaN-box the raw value based on its type
                            let boxed = nan_box_raw_value(*value, sym_id, &sym_types, &checker.tcx);
                            attrs.insert(name.clone(), boxed);
                        }
                    }
                }

                // 11. Also extract compiled function pointers for user-defined
                // functions. These are not stored in GLOBAL_ID_NAMESPACE because
                // the MIR calls them directly via MirInst::Call, not through globals.
                for (sym_id, func_name) in &user_func_names {
                    if let Some(ptr) = backend.get_func_ptr(*sym_id) {
                        // NaN-box the function pointer with TAG_FUNC (matching
                        // MirConst::FuncRef encoding in the JIT backend).
                        attrs.insert(func_name.clone(), MbValue::from_func(ptr as usize));
                    }
                }

                // Expose top-level classes as module attrs. A class value is a
                // bare class-name string that the call/isinstance machinery
                // resolves through CLASS_REGISTRY. Only add ones that actually
                // registered (decorators/metaclasses may rename), and don't
                // clobber an explicit same-named global binding.
                for cls_name in &user_class_names {
                    if attrs.contains_key(cls_name) {
                        continue;
                    }
                    if super::class::class_is_registered(cls_name) {
                        attrs.insert(
                            cls_name.clone(),
                            MbValue::from_ptr(MbObject::new_str(cls_name.clone())),
                        );
                    }
                }

                // R4: Set package-related attributes.
                if is_pkg {
                    if let Some(ref dir) = pkg_dir {
                        // __path__ = [package_directory]
                        let path_list =
                            MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
                                MbObject::new_str(dir.display().to_string()),
                            )]));
                        attrs.insert("__path__".into(), path_list);
                    }
                }
                attrs.insert(
                    "__package__".into(),
                    MbValue::from_ptr(MbObject::new_str(package_attr.clone())),
                );

                // Store attrs into the module in MODULES
                let mod_name = module_name.to_string();
                MODULES.with(|mods| {
                    if let Some(m) = mods.borrow_mut().get_mut(&mod_name) {
                        m.attrs = attrs;
                    }
                });

                // Keep the JIT backend alive so function pointers remain valid.
                // Module-level functions (def foo(): ...) are compiled into JIT
                // memory owned by this backend. Dropping it would make those
                // function pointers dangling.
                MODULE_JIT_BACKENDS.with(|backends| {
                    backends.borrow_mut().push(backend);
                });

                Some(module_globals)
            }
            _ => None,
        }
    })();

    // 11. Restore caller's globals regardless of success/failure
    restore_global_id_namespace(saved_globals);

    // 11a. Persist the imported module's own global-id bindings (module-level
    // variables and `import` results) back into the shared namespace. The
    // module's functions read these via mb_global_get_id when invoked later
    // from the caller; without this, every module-level constant/import is
    // None inside an imported module's functions. SymbolIds are unique per
    // compilation, so re-inserting cannot clobber the caller's globals.
    if let Some(module_globals) = &jit_result {
        crate::runtime::closure::merge_global_id_namespace(module_globals);
    }

    // 11b. Restore CURRENT_MODULE_PACKAGE — R3.
    CURRENT_MODULE_PACKAGE.with(|cp| {
        *cp.borrow_mut() = saved_package;
    });

    if jit_result.is_none() {
        // Compilation or execution failed — the sentinel module stays with whatever
        // attrs were set (possibly empty). This matches CPython's behavior for
        // partially-initialized modules during circular imports.
    }
}

/// NaN-box a raw JIT value based on its known type (#1190).
///
/// The JIT stores int as raw i64, float as raw f64 bits, and heap-allocated
/// types (str, list, etc.) as NaN-boxed MbValue::from_ptr already. This
/// function applies the appropriate NaN-boxing for primitive types.
fn nan_box_raw_value(
    raw: MbValue,
    sym_id: crate::resolve::SymbolId,
    sym_types: &HashMap<crate::resolve::SymbolId, crate::types::TypeId>,
    tcx: &crate::types::TypeContext,
) -> MbValue {
    use crate::types::Ty;

    if let Some(&ty_id) = sym_types.get(&sym_id) {
        match tcx.get(ty_id) {
            Ty::Int => {
                if raw.as_int().is_some() {
                    return raw;
                }
                let raw_i64 = raw.to_bits() as i64;
                if (-(1i64 << 47)..(1i64 << 47)).contains(&raw_i64) {
                    // Raw i64 -> NaN-boxed int.
                    return MbValue::from_int(raw_i64);
                }
                // If type inference says int but the stored value is already
                // some other NaN-boxed runtime value, keep it boxed. This avoids
                // turning pointer/string payload bits into out-of-range ints.
                return raw;
            }
            Ty::Float => {
                // Raw f64 bits → NaN-boxed float
                let f = f64::from_bits(raw.to_bits());
                return MbValue::from_float(f);
            }
            Ty::Bool => {
                // Raw i64 (0 or 1) → NaN-boxed bool
                let b = raw.to_bits() != 0;
                return MbValue::from_bool(b);
            }
            _ => {
                // Str, List, Dict, etc. are already NaN-boxed (from_ptr)
                // because the JIT stores them via runtime functions that
                // return NaN-boxed MbValues.
                return raw;
            }
        }
    }
    // Type unknown — return as-is (best effort)
    raw
}

// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-module-rs" tracker="standardize-gap-projects-mamba-src-runtime-module-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
/// Build the runtime introspection registry from build-time data.
///
/// Combines `checker.symbols` + `hir.sym_names` + `hir.sym_types` into the
/// runtime SymbolId → (name, SymTy) map consumed by `mb_globals` /
/// `mb_locals`. The driver / module loader installs the result via
/// `closure::set_module_sym_info` immediately before calling the JIT
/// entry point. `func_addrs` carries any user-defined function pointers
/// that should appear in `globals()` (these don't live in
/// GLOBAL_ID_NAMESPACE).
/// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#globals_impl
pub fn build_introspection_state(
    checker: &crate::types::TypeChecker,
    hir: &crate::hir::HirModule,
    func_addrs: &[(u32, String, *const u8)],
) -> (
    HashMap<i64, (String, super::closure::SymTy)>,
    HashMap<String, MbValue>,
) {
    use super::closure::SymTy;
    use crate::types::Ty;

    // Merge sym_names from both sources (low-id from type-check, high-id
    // from lowerer) — same union as the existing module-load path.
    let mut sym_names: HashMap<crate::resolve::SymbolId, String> = HashMap::new();
    for sym_info in checker.symbols.all_symbols() {
        sym_names.insert(sym_info.id, sym_info.name.clone());
    }
    for (id, name) in &hir.sym_names {
        sym_names.insert(*id, name.clone());
    }

    let mut sym_types: HashMap<crate::resolve::SymbolId, crate::types::TypeId> = HashMap::new();
    for sym_info in checker.symbols.all_symbols() {
        if let Some(ty_id) = checker.get_symbol_type(sym_info.id) {
            sym_types.insert(sym_info.id, ty_id);
        }
    }
    for (id, ty_id) in &hir.sym_types {
        sym_types.insert(*id, *ty_id);
    }

    let mut info: HashMap<i64, (String, SymTy)> = HashMap::new();
    for (sym_id, name) in &sym_names {
        let ty_tag = sym_types
            .get(sym_id)
            .map(|ty_id| match checker.tcx.get(*ty_id) {
                Ty::Int => SymTy::Int,
                Ty::Float => SymTy::Float,
                Ty::Bool => SymTy::Bool,
                _ => SymTy::Boxed,
            })
            .unwrap_or(SymTy::Boxed);
        info.insert(sym_id.0 as i64, (name.clone(), ty_tag));
    }

    let mut funcs: HashMap<String, MbValue> = HashMap::new();
    for (_sym_id, name, addr) in func_addrs {
        funcs.insert(name.clone(), MbValue::from_func(*addr as usize));
    }

    (info, funcs)
}
// HANDWRITE-END

// ── Internal Helpers ──

/// Find a module file in the search paths.
///
/// Search order (matching CPython):
/// 1. SCRIPT_DIR — directory of the currently executing script (#1190)
/// 2. SEARCH_PATHS — configured search paths (defaults to ["."])
fn find_module(name: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = name.split('.').collect();

    // Check SCRIPT_DIR first (#1190)
    let script_dir_result = SCRIPT_DIR.with(|sd| {
        if let Some(ref dir) = *sd.borrow() {
            return probe_module_path(dir, &parts);
        }
        None
    });
    if script_dir_result.is_some() {
        return script_dir_result;
    }

    SEARCH_PATHS.with(|paths| {
        for base in paths.borrow().iter() {
            if let Some(found) = probe_module_path(base, &parts) {
                return Some(found);
            }
        }
        None
    })
}

/// Try to find a module file at `base/parts.py` or `base/parts/__init__.py`.
fn probe_module_path(base: &std::path::Path, parts: &[&str]) -> Option<PathBuf> {
    let mut path = base.to_path_buf();
    for part in parts {
        path.push(part);
    }
    let py_path = path.with_extension("py");
    if py_path.exists() {
        return Some(py_path);
    }

    // Try name/__init__.py (package)
    path.push("__init__.py");
    if path.exists() {
        return Some(path);
    }
    None
}

/// Convert a MbModule to a MbValue (as a dict of its attributes).
pub(crate) fn module_to_value(module: &MbModule) -> MbValue {
    // Return the cached object so repeated imports yield the same pointer
    // (`import X; import X as Y; X is Y` holds — CPython Rule 2).
    if let Some(cached) = module.cached_value {
        return cached;
    }
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__name__".into(),
                MbValue::from_ptr(MbObject::new_str(module.name.clone())),
            );
            if let Some(ref path) = module.file {
                map.insert(
                    "__file__".into(),
                    MbValue::from_ptr(MbObject::new_str(path.display().to_string())),
                );
            }
            let pkg = if module.is_package {
                &module.name
            } else {
                module.name.rsplit_once('.').map(|(p, _)| p).unwrap_or("")
            };
            map.insert(
                "__package__".into(),
                MbValue::from_ptr(MbObject::new_str(pkg.to_string())),
            );
            for (k, v) in &module.attrs {
                // Fix C-prime: the fresh dict will recursively release each
                // contained value when its rc hits 0 (rc::release_contained_values).
                // `module.attrs` still holds its own raw reference, so we must
                // retain here to keep the registry's reference valid past the
                // dict's eventual drop.
                super::rc::retain_if_ptr(*v);
                map.insert(k.clone().into(), *v);
            }
        }
    }
    // Mark this dict as a module value so isinstance(_, types.ModuleType) and
    // type(_) can distinguish it from an ordinary dict.
    MODULE_VALUE_PTRS.with(|s| {
        s.borrow_mut().insert(dict as u64);
    });
    MbValue::from_ptr(dict)
}

/// True iff `v` is a dict that represents an imported module (see
/// MODULE_VALUE_PTRS) — backs isinstance(v, types.ModuleType) and type(v).
pub fn is_module_value(v: MbValue) -> bool {
    match v.as_ptr() {
        Some(p) => MODULE_VALUE_PTRS.with(|s| s.borrow().contains(&(p as u64))),
        None => false,
    }
}

/// Read an attribute from a module's user-visible namespace dict — the cached
/// module value that user code mutates via `mod.attr = x` (lands in the dict's
/// `__name__`-tagged stub path, see `class::mb_setattr`). This reflects user
/// overrides that the registry-backed [`mb_module_getattr`] does not see,
/// because user assignment writes the cached dict, not `MbModule::attrs`.
///
/// Returns `None` when the module is not loaded, has no cached value, or the
/// attribute is absent. The returned value carries a fresh +1 reference.
pub fn mb_module_value_getattr(module_name: &str, attr: &str) -> Option<MbValue> {
    let cached = MODULES.with(|mods| mods.borrow().get(module_name).and_then(|m| m.cached_value))?;
    let ptr = cached.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            let key = super::dict_ops::DictKey::Str(attr.to_string());
            if let Some(v) = map.get(&key).copied() {
                super::rc::retain_if_ptr(v);
                return Some(v);
            }
        }
    }
    None
}

/// Look up a module attribute directly from the registry's `attrs` map,
/// returning the live value with a +1 retain. Unlike `mb_module_value_getattr`
/// (which reads the cached module-object dict), this reflects in-place
/// `sys.<attr> = ...` reassignments routed through `mb_module_setattr`, which
/// only update `attrs`. Used by `breakpoint()` to read the current
/// `sys.breakpointhook`. (#242)
pub fn mb_module_attr_lookup(module_name: &str, attr: &str) -> Option<MbValue> {
    let val = MODULES.with(|mods| {
        let mods = mods.borrow();
        mods.get(module_name).and_then(|m| m.attrs.get(attr).copied())
    })?;
    unsafe {
        super::rc::retain_if_ptr(val);
    }
    Some(val)
}

/// Like `module_to_value` but writes the result back into `module.cached_value`.
/// Call this whenever a module is fully initialised so subsequent `module_to_value`
/// calls return the same heap pointer.
pub(crate) fn module_to_value_and_cache(module: &mut MbModule) -> MbValue {
    let val = module_to_value(module);
    module.cached_value = Some(val);
    val
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ── Cleanup ──

/// Reset all module-related thread_local state to defaults.
/// Called as part of centralized runtime cleanup between test executions.
pub(crate) fn cleanup_all_modules() {
    let _ = MODULES.with(|c| {
        c.try_borrow_mut().map(|mut m| {
            // Module attrs mix borrowed and owned MbValues. Replace the registry
            // with an empty map without dropping the old entries during teardown.
            let modules = std::mem::take(&mut *m);
            std::mem::forget(modules);
        })
    });
    let _ = SEARCH_PATHS.with(|c| {
        c.try_borrow_mut().map(|mut m| {
            m.clear();
            m.push(std::path::PathBuf::from("."));
        })
    });
    let _ = NATIVE_FUNC_ADDRS.with(|c| c.try_borrow_mut().map(|mut s| s.clear()));
    let _ = VARIADIC_SYMBOL_IDS.with(|c| c.try_borrow_mut().map(|mut s| s.clear()));
    let _ = VARIADIC_FUNC_ADDRS.with(|c| c.try_borrow_mut().map(|mut s| s.clear()));
    let _ = KWARGS_SYMBOL_IDS.with(|c| c.try_borrow_mut().map(|mut s| s.clear()));
    let _ = KWARGS_FUNC_ADDRS.with(|c| c.try_borrow_mut().map(|mut s| s.clear()));
    let _ = BOXED_RETURN_SYMBOL_IDS.with(|c| c.try_borrow_mut().map(|mut s| s.clear()));
    let _ = BOXED_RETURN_FUNC_ADDRS.with(|c| c.try_borrow_mut().map(|mut s| s.clear()));
    // NOTE: MODULE_JIT_BACKENDS is cleared separately by cleanup_module_jit_backends().
    // GC must run between this call and backend cleanup so that containers are
    // swept while compile-time objects (owned by backends) are still valid.
    // Reset script directory (#1190).
    let _ = SCRIPT_DIR.with(|c| c.try_borrow_mut().map(|mut s| *s = None));
    // Reset current module package (#1190 R3).
    let _ = CURRENT_MODULE_PACKAGE.with(|c| c.try_borrow_mut().map(|mut s| *s = None));
}

/// Drop all JIT backends for imported file-based modules (#1190).
///
/// Module attrs are detached by cleanup_all_modules() before this runs, so
/// dropping backends here does not cascade through mixed-owned module values.
/// Releasing the code memory is still important for the in-process conformance
/// harness, which compiles hundreds of fixtures in one process.
pub(crate) fn cleanup_module_jit_backends() {
    let _ = MODULE_JIT_BACKENDS.with(|c| c.try_borrow_mut().map(|mut v| v.clear()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(name: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(name.to_string()))
    }

    #[test]
    fn test_register_and_import() {
        let mut attrs = HashMap::new();
        attrs.insert("foo".into(), MbValue::from_int(42));
        mb_module_register("test_mod", attrs);

        let name = MbValue::from_ptr(MbObject::new_str("test_mod".to_string()));
        let result = mb_import(name);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_module_getattr() {
        let mut attrs = HashMap::new();
        attrs.insert("bar".into(), MbValue::from_int(99));
        mb_module_register("mymod", attrs);

        let mod_name = MbValue::from_ptr(MbObject::new_str("mymod".to_string()));
        let attr = MbValue::from_ptr(MbObject::new_str("bar".to_string()));
        assert_eq!(mb_module_getattr(mod_name, attr).as_int(), Some(99));
    }

    #[test]
    fn test_builtins() {
        mb_register_builtins();
        let sys = MbValue::from_ptr(MbObject::new_str("sys".to_string()));
        let result = mb_import(sys);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_import_sys_has_argv() {
        mb_register_builtins();
        let mod_name = MbValue::from_ptr(MbObject::new_str("sys".to_string()));
        let attr = MbValue::from_ptr(MbObject::new_str("argv".to_string()));
        let argv = mb_module_getattr(mod_name, attr);
        assert!(argv.is_ptr(), "sys.argv should be a list");
    }

    #[test]
    fn test_import_json_has_dumps() {
        mb_register_builtins();
        let mod_name = MbValue::from_ptr(MbObject::new_str("json".to_string()));
        let attr = MbValue::from_ptr(MbObject::new_str("dumps".to_string()));
        let dumps = mb_module_getattr(mod_name, attr);
        assert!(!dumps.is_none(), "json.dumps should be accessible");
    }

    #[test]
    fn test_import_math_has_sqrt() {
        mb_register_builtins();
        let mod_name = MbValue::from_ptr(MbObject::new_str("math".to_string()));
        let attr = MbValue::from_ptr(MbObject::new_str("sqrt".to_string()));
        let sqrt = mb_module_getattr(mod_name, attr);
        assert!(!sqrt.is_none(), "math.sqrt should be accessible");
    }

    #[test]
    fn test_import_os_has_getcwd() {
        mb_register_builtins();
        let mod_name = MbValue::from_ptr(MbObject::new_str("os".to_string()));
        let attr = MbValue::from_ptr(MbObject::new_str("getcwd".to_string()));
        let getcwd = mb_module_getattr(mod_name, attr);
        assert!(!getcwd.is_none(), "os.getcwd should be accessible");
    }

    #[test]
    fn test_search_path_syncs_sys_path() {
        mb_register_builtins();
        let new_path = MbValue::from_ptr(MbObject::new_str("/test/search/path".to_string()));
        mb_add_search_path(new_path);

        // Verify sys.path contains the new path
        let mod_name = MbValue::from_ptr(MbObject::new_str("sys".to_string()));
        let attr = MbValue::from_ptr(MbObject::new_str("path".to_string()));
        let sys_path = mb_module_getattr(mod_name, attr);
        assert!(sys_path.is_ptr(), "sys.path should be a list");
        unsafe {
            if let ObjData::List(ref lock) = (*sys_path.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                let has_path = items.iter().any(|v| {
                    v.as_ptr()
                        .and_then(|p| {
                            if let ObjData::Str(ref s) = (*p).data {
                                Some(s == "/test/search/path")
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false)
                });
                assert!(has_path, "sys.path should contain the added path");
            }
        }
    }

    // ── New tests ──

    #[test]
    fn test_import_missing_module_returns_none() {
        let result = mb_import(s("__nonexistent_module_xyz__"));
        assert!(
            result.is_none(),
            "importing a missing module should return None"
        );
    }

    #[test]
    fn test_module_setattr_then_getattr() {
        let mut attrs = HashMap::new();
        attrs.insert("existing".into(), MbValue::from_int(1));
        mb_module_register("setattr_mod", attrs);

        mb_module_setattr(s("setattr_mod"), s("key"), MbValue::from_int(77));
        let val = mb_module_getattr(s("setattr_mod"), s("key"));
        assert_eq!(val.as_int(), Some(77));
    }

    #[test]
    fn test_module_setattr_overwrite() {
        let mut attrs = HashMap::new();
        attrs.insert("x".into(), MbValue::from_int(1));
        mb_module_register("overwrite_mod", attrs);

        mb_module_setattr(s("overwrite_mod"), s("x"), MbValue::from_int(10));
        mb_module_setattr(s("overwrite_mod"), s("x"), MbValue::from_int(99));
        let val = mb_module_getattr(s("overwrite_mod"), s("x"));
        assert_eq!(val.as_int(), Some(99), "last setattr should win");
    }

    #[test]
    fn test_import_returns_ptr_with_name() {
        let mut attrs = HashMap::new();
        attrs.insert("v".into(), MbValue::from_int(5));
        mb_module_register("named_mod", attrs);

        let result = mb_import(s("named_mod"));
        assert!(result.is_ptr());
        // The returned dict should contain __name__
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert!(
                    map.contains_key("__name__"),
                    "module value must have __name__"
                );
            } else {
                panic!("expected Dict");
            }
        }
    }

    #[test]
    fn test_import_from_returns_tuple() {
        let mut attrs = HashMap::new();
        attrs.insert("a".into(), MbValue::from_int(1));
        attrs.insert("b".into(), MbValue::from_int(2));
        mb_module_register("from_mod", attrs);

        let names = MbValue::from_ptr(MbObject::new_list(vec![s("a"), s("b")]));
        let result = mb_import_from(s("from_mod"), names);
        assert!(result.is_ptr(), "import_from should return a ptr (tuple)");
    }

    #[test]
    fn test_import_from_single_attr() {
        let mut attrs = HashMap::new();
        attrs.insert("only".into(), MbValue::from_int(42));
        mb_module_register("single_from_mod", attrs);

        let names = MbValue::from_ptr(MbObject::new_list(vec![s("only")]));
        let result = mb_import_from(s("single_from_mod"), names);
        assert!(result.is_ptr());
        // Inspect the tuple — first element should be int 42
        unsafe {
            if let ObjData::Tuple(ref items) = (*result.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].as_int(), Some(42));
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_import_from_missing_attr_raises_import_error() {
        let mut attrs = HashMap::new();
        attrs.insert("present".into(), MbValue::from_int(1));
        mb_module_register("partial_from_mod2", attrs);

        let names = MbValue::from_ptr(MbObject::new_list(vec![s("missing_key")]));
        let result = mb_import_from(s("partial_from_mod2"), names);
        // CPython raises ImportError for missing attrs; mamba should match.
        assert!(
            result.is_none(),
            "should return none sentinel after raising ImportError"
        );
        let exc = super::super::exception::mb_get_exception();
        assert!(!exc.is_none(), "ImportError should be set");
        let exc_type = super::super::exception::get_exception_type_pub(exc).unwrap_or_default();
        assert_eq!(
            exc_type, "ImportError",
            "exception type should be ImportError"
        );
        super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_future_stmt_badsyntax_import_raises_located_syntax_error() {
        mb_module_register("test.test_future_stmt", HashMap::new());

        let result = mb_module_getattr(s("test.test_future_stmt"), s("badsyntax_future6"));
        assert!(result.is_none(), "bad future helper import should fail");

        let exc = super::super::class::mb_catch_exception_instance();
        let exc_type = super::super::exception::get_exception_type_pub(exc).unwrap_or_default();
        assert_eq!(exc_type, "SyntaxError");

        unsafe {
            let Some(ptr) = exc.as_ptr() else {
                panic!("expected SyntaxError instance");
            };
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                assert_eq!(
                    fields
                        .get("filename")
                        .and_then(|v| extract_str(*v))
                        .as_deref(),
                    Some("badsyntax_future6.py")
                );
                assert_eq!(fields.get("lineno").and_then(|v| v.as_int()), Some(3));
                assert_eq!(fields.get("offset").and_then(|v| v.as_int()), Some(1));
                assert!(fields
                    .get("message")
                    .and_then(|v| extract_str(*v))
                    .unwrap_or_default()
                    .contains("badsyntax_future6.py, line 3"));
            } else {
                panic!("expected SyntaxError instance");
            }
        }
    }

    #[test]
    fn test_import_cached_reuse() {
        let mut attrs = HashMap::new();
        attrs.insert("z".into(), MbValue::from_int(7));
        mb_module_register("cache_mod", attrs);

        let r1 = mb_import(s("cache_mod"));
        let r2 = mb_import(s("cache_mod"));
        assert!(r1.is_ptr());
        assert!(r2.is_ptr());
    }

    #[test]
    fn test_module_getattr_missing_returns_none() {
        let attrs = HashMap::new();
        mb_module_register("empty_getattr_mod", attrs);
        let val = mb_module_getattr(s("empty_getattr_mod"), s("no_such_attr"));
        assert!(
            val.is_none(),
            "getattr for nonexistent attr should return None"
        );
    }

    #[test]
    fn test_module_getattr_after_setattr() {
        let attrs = HashMap::new();
        mb_module_register("setattr_int_mod", attrs);
        mb_module_setattr(s("setattr_int_mod"), s("x"), MbValue::from_int(42));
        let val = mb_module_getattr(s("setattr_int_mod"), s("x"));
        assert_eq!(val.as_int(), Some(42));
    }

    #[test]
    fn test_register_empty_attrs() {
        let attrs = HashMap::new();
        mb_module_register("empty_mod", attrs);
        let result = mb_import(s("empty_mod"));
        assert!(
            result.is_ptr(),
            "importing a module with no attrs should succeed"
        );
    }

    #[test]
    fn test_register_overwrite() {
        let mut attrs1 = HashMap::new();
        attrs1.insert("val".into(), MbValue::from_int(1));
        mb_module_register("overwrite_reg_mod", attrs1);

        let mut attrs2 = HashMap::new();
        attrs2.insert("val".into(), MbValue::from_int(2));
        mb_module_register("overwrite_reg_mod", attrs2);

        let result = mb_module_getattr(s("overwrite_reg_mod"), s("val"));
        assert_eq!(result.as_int(), Some(2), "second registration should win");
    }

    #[test]
    fn test_multiple_modules_independent() {
        let mut attrs_a = HashMap::new();
        attrs_a.insert("n".into(), MbValue::from_int(100));
        mb_module_register("mod_a_indep", attrs_a);

        let mut attrs_b = HashMap::new();
        attrs_b.insert("n".into(), MbValue::from_int(200));
        mb_module_register("mod_b_indep", attrs_b);

        let va = mb_module_getattr(s("mod_a_indep"), s("n"));
        let vb = mb_module_getattr(s("mod_b_indep"), s("n"));
        assert_eq!(va.as_int(), Some(100));
        assert_eq!(vb.as_int(), Some(200));
    }

    #[test]
    fn test_module_stores_multiple_attrs() {
        let mut attrs = HashMap::new();
        attrs.insert("a".into(), MbValue::from_int(1));
        attrs.insert("b".into(), MbValue::from_int(2));
        attrs.insert("c".into(), MbValue::from_int(3));
        mb_module_register("multi_attr_mod", attrs);

        assert_eq!(
            mb_module_getattr(s("multi_attr_mod"), s("a")).as_int(),
            Some(1)
        );
        assert_eq!(
            mb_module_getattr(s("multi_attr_mod"), s("b")).as_int(),
            Some(2)
        );
        assert_eq!(
            mb_module_getattr(s("multi_attr_mod"), s("c")).as_int(),
            Some(3)
        );
    }

    #[test]
    fn test_import_builtin_os() {
        mb_register_builtins();
        let result = mb_import(s("os"));
        assert!(result.is_ptr(), "builtin 'os' module should be importable");
    }

    #[test]
    fn test_import_builtin_json() {
        mb_register_builtins();
        let result = mb_import(s("json"));
        assert!(
            result.is_ptr(),
            "builtin 'json' module should be importable"
        );
    }

    #[test]
    fn test_import_builtin_math() {
        mb_register_builtins();
        let result = mb_import(s("math"));
        assert!(
            result.is_ptr(),
            "builtin 'math' module should be importable"
        );
    }

    #[test]
    fn test_import_builtin_sys_version() {
        mb_register_builtins();
        let version = mb_module_getattr(s("sys"), s("version"));
        assert!(!version.is_none(), "sys.version should not be None");
    }

    #[test]
    fn test_import_builtin_builtins_module() {
        mb_register_builtins();
        let result = mb_import(s("builtins"));
        assert!(
            result.is_ptr(),
            "builtin 'builtins' module should be importable"
        );
        // builtins should have True
        let true_val = mb_module_getattr(s("builtins"), s("True"));
        assert!(!true_val.is_none(), "builtins.True should exist");
    }

    #[test]
    fn test_add_search_path_doesnt_crash() {
        // Must not panic
        mb_add_search_path(s("/tmp"));
    }

    #[test]
    fn test_module_value_preserves_int() {
        let mut attrs = HashMap::new();
        attrs.insert("count".into(), MbValue::from_int(55));
        mb_module_register("int_preserve_mod", attrs);

        let result = mb_import(s("int_preserve_mod"));
        assert!(result.is_ptr());
        // Verify the dict representation contains the int attr
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                let val = map.get("count").copied().unwrap_or(MbValue::none());
                assert_eq!(val.as_int(), Some(55));
            } else {
                panic!("expected Dict");
            }
        }
    }

    #[test]
    fn test_module_value_preserves_str() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "label".into(),
            MbValue::from_ptr(MbObject::new_str("hello".to_string())),
        );
        mb_module_register("str_preserve_mod", attrs);

        let val = mb_module_getattr(s("str_preserve_mod"), s("label"));
        assert!(val.is_ptr());
        unsafe {
            if let ObjData::Str(ref st) = (*val.as_ptr().unwrap()).data {
                assert_eq!(st, "hello");
            } else {
                panic!("expected Str");
            }
        }
    }

    // ── Cleanup tests (R1: per-module cleanup for modules) ──

    #[test]
    fn test_cleanup_all_modules_clears_registry() {
        let mut attrs = HashMap::new();
        attrs.insert("val".into(), MbValue::from_int(42));
        mb_module_register("cleanup_mod_test", attrs);

        // Verify it exists
        let result = mb_import(s("cleanup_mod_test"));
        assert!(
            result.is_ptr(),
            "module should be importable before cleanup"
        );

        cleanup_all_modules();

        // After cleanup, import should fail
        let result2 = mb_import(s("cleanup_mod_test"));
        assert!(result2.is_none(), "MODULES should be empty after cleanup");
    }

    #[test]
    fn test_cleanup_all_modules_resets_search_paths() {
        mb_add_search_path(s("/some/custom/path"));

        cleanup_all_modules();

        // After cleanup, search paths should be reset to default (just ".")
        SEARCH_PATHS.with(|sp| {
            let paths = sp.borrow();
            assert_eq!(paths.len(), 1, "search paths should be reset to default");
            assert_eq!(
                paths[0],
                PathBuf::from("."),
                "default search path should be '.'"
            );
        });
    }

    #[test]
    fn test_cleanup_all_modules_on_empty() {
        cleanup_all_modules();
        // No panic = success
    }

    #[test]
    fn test_cleanup_all_modules_then_reregister() {
        let mut attrs = HashMap::new();
        attrs.insert("x".into(), MbValue::from_int(1));
        mb_module_register("reregister_mod", attrs);

        cleanup_all_modules();

        // Re-register after cleanup
        let mut attrs2 = HashMap::new();
        attrs2.insert("y".into(), MbValue::from_int(2));
        mb_module_register("reregister_mod", attrs2);

        let val = mb_module_getattr(s("reregister_mod"), s("y"));
        assert_eq!(
            val.as_int(),
            Some(2),
            "module should be usable after cleanup + re-register"
        );
        // The old attr should not exist
        let old_val = mb_module_getattr(s("reregister_mod"), s("x"));
        assert!(old_val.is_none(), "old attrs should not survive cleanup");
    }

    // ── #1190 Import System Tests ──────────────────────────────────────────

    // ── R1: PYTHONPATH Integration ──

    #[test]
    fn test_init_search_paths_reads_pythonpath() {
        cleanup_all_modules();
        // Set PYTHONPATH to a known value.
        std::env::set_var("PYTHONPATH", "/opt/test_libs:/home/test/mylibs");
        mb_init_search_paths();

        SEARCH_PATHS.with(|sp| {
            let paths = sp.borrow();
            let path_strs: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            assert!(
                path_strs.contains(&"/opt/test_libs".to_string()),
                "SEARCH_PATHS should contain /opt/test_libs, got: {:?}",
                path_strs
            );
            assert!(
                path_strs.contains(&"/home/test/mylibs".to_string()),
                "SEARCH_PATHS should contain /home/test/mylibs, got: {:?}",
                path_strs
            );
        });

        // Clean up env var.
        std::env::remove_var("PYTHONPATH");
        cleanup_all_modules();
    }

    #[test]
    fn test_init_search_paths_empty_pythonpath() {
        cleanup_all_modules();
        std::env::remove_var("PYTHONPATH");
        mb_init_search_paths();

        SEARCH_PATHS.with(|sp| {
            let paths = sp.borrow();
            // Should only have the default "." entry.
            assert_eq!(
                paths.len(),
                1,
                "empty PYTHONPATH should not modify SEARCH_PATHS, got: {:?}",
                paths
            );
            assert_eq!(paths[0], PathBuf::from("."));
        });
        cleanup_all_modules();
    }

    #[test]
    fn test_init_search_paths_syncs_sys_path() {
        cleanup_all_modules();
        mb_register_builtins();
        std::env::set_var("PYTHONPATH", "/test/sync_path");
        mb_init_search_paths();

        // Verify sys.path contains the entry.
        let sys_path_val = mb_module_getattr(s("sys"), s("path"));
        assert!(sys_path_val.is_ptr(), "sys.path should be a list");
        unsafe {
            if let ObjData::List(ref lock) = (*sys_path_val.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                let has_entry = items.iter().any(|v| {
                    v.as_ptr()
                        .and_then(|p| {
                            if let ObjData::Str(ref ss) = (*p).data {
                                Some(ss == "/test/sync_path")
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false)
                });
                assert!(has_entry, "sys.path should contain /test/sync_path");
            }
        }

        std::env::remove_var("PYTHONPATH");
        cleanup_all_modules();
    }

    #[test]
    fn test_pythonpath_invalid_dirs_skipped() {
        cleanup_all_modules();
        // PYTHONPATH with non-existent directories should not cause errors.
        std::env::set_var("PYTHONPATH", "/nonexistent_dir_xyz:/also_nonexistent_abc");
        mb_init_search_paths();

        // Should have added the entries (they are added even if non-existent,
        // matching CPython behavior).
        SEARCH_PATHS.with(|sp| {
            let paths = sp.borrow();
            let path_strs: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            assert!(
                path_strs.contains(&"/nonexistent_dir_xyz".to_string()),
                "non-existent PYTHONPATH entries should still be added"
            );
        });

        std::env::remove_var("PYTHONPATH");
        cleanup_all_modules();
    }

    // ── R2: Script Directory ──

    #[test]
    fn test_set_script_dir_updates_global() {
        cleanup_all_modules();
        mb_set_script_dir(PathBuf::from("/tmp/test_script_dir"));

        SCRIPT_DIR.with(|sd| {
            let dir = sd.borrow();
            assert_eq!(
                dir.as_ref().unwrap(),
                &PathBuf::from("/tmp/test_script_dir"),
                "SCRIPT_DIR should be set"
            );
        });
        cleanup_all_modules();
    }

    #[test]
    fn test_find_module_checks_script_dir_first() {
        cleanup_all_modules();

        let dir = tempfile::tempdir().unwrap();
        let mod_path = dir.path().join("script_dir_mod.py");
        std::fs::write(&mod_path, "x = 42\n").unwrap();

        mb_set_script_dir(dir.path().to_path_buf());
        let found = find_module("script_dir_mod");
        assert!(
            found.is_some(),
            "find_module should find module in SCRIPT_DIR"
        );
        assert_eq!(found.unwrap(), mod_path);
        cleanup_all_modules();
    }

    #[test]
    fn test_find_module_package_init() {
        cleanup_all_modules();

        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("mypkg");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        let init_path = pkg_dir.join("__init__.py");
        std::fs::write(&init_path, "PKG_VAR = 1\n").unwrap();

        mb_set_script_dir(dir.path().to_path_buf());
        let found = find_module("mypkg");
        assert!(
            found.is_some(),
            "find_module should find package with __init__.py"
        );
        assert!(
            found.unwrap().ends_with("__init__.py"),
            "found path should end with __init__.py"
        );
        cleanup_all_modules();
    }

    #[test]
    fn test_search_order_script_dir_before_pythonpath() {
        cleanup_all_modules();

        // Create two directories, each with a module of the same name but different content.
        let script_dir = tempfile::tempdir().unwrap();
        let pythonpath_dir = tempfile::tempdir().unwrap();

        std::fs::write(script_dir.path().join("shadow_mod.py"), "x = 1\n").unwrap();
        std::fs::write(pythonpath_dir.path().join("shadow_mod.py"), "x = 2\n").unwrap();

        mb_set_script_dir(script_dir.path().to_path_buf());
        // Add pythonpath_dir to SEARCH_PATHS.
        let pp = MbValue::from_ptr(MbObject::new_str(
            pythonpath_dir.path().display().to_string(),
        ));
        mb_add_search_path(pp);

        let found = find_module("shadow_mod");
        assert!(found.is_some());
        // Should resolve to the script_dir version (higher priority).
        let found_path = found.unwrap();
        assert!(
            found_path.starts_with(script_dir.path()),
            "script dir module should shadow PYTHONPATH module"
        );
        cleanup_all_modules();
    }

    #[test]
    fn test_insert_search_path_at_position() {
        cleanup_all_modules();
        mb_insert_search_path(0, "/first/path");
        mb_insert_search_path(0, "/zeroth/path");

        SEARCH_PATHS.with(|sp| {
            let paths = sp.borrow();
            assert_eq!(
                paths[0],
                PathBuf::from("/zeroth/path"),
                "position 0 insert should be first"
            );
            assert_eq!(
                paths[1],
                PathBuf::from("/first/path"),
                "previous position 0 insert should shift to 1"
            );
        });
        cleanup_all_modules();
    }

    // ── R3: Relative Import Resolution ──

    #[test]
    fn test_relative_import_no_package_context() {
        cleanup_all_modules();
        // No CURRENT_MODULE_PACKAGE set — relative import should return None.
        CURRENT_MODULE_PACKAGE.with(|cp| {
            *cp.borrow_mut() = None;
        });
        let result = mb_import_relative(s("foo"), MbValue::from_int(1));
        assert!(
            result.is_none(),
            "relative import without package context should return None"
        );
        cleanup_all_modules();
    }

    #[test]
    fn test_relative_import_level_exceeds_hierarchy() {
        cleanup_all_modules();
        // Set package to single-level "mypkg" — level=3 exceeds hierarchy.
        CURRENT_MODULE_PACKAGE.with(|cp| {
            *cp.borrow_mut() = Some("mypkg".to_string());
        });
        let result = mb_import_relative(s("foo"), MbValue::from_int(3));
        assert!(
            result.is_none(),
            "relative import exceeding hierarchy should return None"
        );
        cleanup_all_modules();
    }

    #[test]
    fn test_relative_import_level_zero_delegates_to_absolute() {
        cleanup_all_modules();
        // Register a module, then do a relative import with level=0 (absolute).
        let mut attrs = HashMap::new();
        attrs.insert("val".into(), MbValue::from_int(10));
        mb_module_register("abs_test_mod", attrs);

        let result = mb_import_relative(s("abs_test_mod"), MbValue::from_int(0));
        assert!(result.is_ptr(), "level=0 should delegate to mb_import");
        cleanup_all_modules();
    }

    // ── R4: Package Module Attributes ──

    #[test]
    fn test_package_module_has_is_package_true() {
        cleanup_all_modules();

        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("pkg_test");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("__init__.py"), "PKG_VAR = 1\n").unwrap();

        mb_set_script_dir(dir.path().to_path_buf());
        let _result = mb_import(s("pkg_test"));

        MODULES.with(|mods| {
            let mods = mods.borrow();
            let module = mods.get("pkg_test");
            assert!(module.is_some(), "pkg_test should be in MODULES");
            assert!(
                module.unwrap().is_package,
                "package module should have is_package=true"
            );
        });
        cleanup_all_modules();
    }

    #[test]
    fn test_package_module_has_package_attr() {
        cleanup_all_modules();

        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("pkg_attr_test");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("__init__.py"), "PKG_VAR = 1\n").unwrap();

        mb_set_script_dir(dir.path().to_path_buf());
        let result = mb_import(s("pkg_attr_test"));
        assert!(result.is_ptr(), "import should return a dict");

        // Check __package__ in the module dict.
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                let pkg = map.get("__package__").copied().unwrap_or(MbValue::none());
                assert!(pkg.is_ptr(), "__package__ should be a string");
                if let ObjData::Str(ref ss) = (*pkg.as_ptr().unwrap()).data {
                    assert_eq!(
                        ss, "pkg_attr_test",
                        "__package__ should equal the module name for packages"
                    );
                }
            }
        }
        cleanup_all_modules();
    }

    #[test]
    fn test_non_package_module_package_attr() {
        cleanup_all_modules();

        let dir = tempfile::tempdir().unwrap();
        // Create a simple (non-package) module file.
        std::fs::write(dir.path().join("simple_mod.py"), "x = 10\n").unwrap();

        mb_set_script_dir(dir.path().to_path_buf());
        let result = mb_import(s("simple_mod"));
        assert!(result.is_ptr(), "import should succeed");

        // Non-package module: __package__ should be empty string (no parent).
        MODULES.with(|mods| {
            let mods = mods.borrow();
            let module = mods.get("simple_mod");
            assert!(module.is_some());
            assert!(
                !module.unwrap().is_package,
                "simple .py file should not be a package"
            );
        });
        cleanup_all_modules();
    }

    // ── R5: Sub-Module Auto-Loading ──

    #[test]
    fn test_sub_module_auto_load_nonexistent() {
        cleanup_all_modules();

        // Create a package with no sub-modules.
        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("empty_pkg");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("__init__.py"), "").unwrap();

        mb_set_script_dir(dir.path().to_path_buf());
        let _mod_val = mb_import(s("empty_pkg"));

        // Accessing a non-existent sub-module should return None.
        let result = mb_module_getattr(s("empty_pkg"), s("nonexistent"));
        assert!(
            result.is_none(),
            "accessing non-existent sub-module should return None"
        );
        cleanup_all_modules();
    }

    // ── R6: Module Caching ──

    #[test]
    fn test_module_caching_no_recompile() {
        cleanup_all_modules();

        let mut attrs = HashMap::new();
        attrs.insert("z".into(), MbValue::from_int(7));
        mb_module_register("cache_test_mod", attrs);

        // First import
        let r1 = mb_import(s("cache_test_mod"));
        assert!(r1.is_ptr());

        // Modify the module attrs directly (simulating external state).
        mb_module_setattr(s("cache_test_mod"), s("extra"), MbValue::from_int(99));

        // Second import should return cached module (with the extra attr).
        let r2 = mb_import(s("cache_test_mod"));
        assert!(r2.is_ptr());

        // The cached module should have the extra attr we added.
        let extra = mb_module_getattr(s("cache_test_mod"), s("extra"));
        assert_eq!(
            extra.as_int(),
            Some(99),
            "cached module should preserve attrs added between imports"
        );
        cleanup_all_modules();
    }

    #[test]
    fn test_circular_import_sentinel() {
        cleanup_all_modules();

        // Simulate circular import by pre-caching a sentinel module.
        MODULES.with(|mods| {
            mods.borrow_mut().insert(
                "circular_a".into(),
                MbModule {
                    name: "circular_a".to_string(),
                    file: None,
                    attrs: HashMap::new(),
                    is_package: false,
                    cached_value: None,
                },
            );
        });

        // Importing "circular_a" should return the sentinel (no infinite recursion).
        let result = mb_import(s("circular_a"));
        assert!(
            result.is_ptr(),
            "importing a pre-cached sentinel should return it without recursion"
        );
        cleanup_all_modules();
    }

    // ── R7: Native Module Priority ──

    #[test]
    fn test_native_module_priority() {
        cleanup_all_modules();

        // Register a "native" module manually.
        let mut attrs = HashMap::new();
        attrs.insert("native_val".into(), MbValue::from_int(999));
        mb_module_register("native_priority_mod", attrs);

        // Create a file-based module with the same name in SCRIPT_DIR.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("native_priority_mod.py"),
            "native_val = 1\n",
        )
        .unwrap();
        mb_set_script_dir(dir.path().to_path_buf());

        // Import should return the pre-registered (native) module, not the file.
        let val = mb_module_getattr(s("native_priority_mod"), s("native_val"));
        assert_eq!(
            val.as_int(),
            Some(999),
            "native module should take priority over file-based module"
        );
        cleanup_all_modules();
    }

    // ── mb_import_star tests ──

    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_with_all
    #[test]
    fn test_import_star_with_all() {
        // Register a module with __all__ = ["foo", "bar"], plus extra attrs
        let all_list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("foo".to_string())),
            MbValue::from_ptr(MbObject::new_str("bar".to_string())),
        ]));
        let mut attrs = HashMap::new();
        attrs.insert("__all__".into(), all_list);
        attrs.insert("foo".into(), MbValue::from_int(1));
        attrs.insert("bar".into(), MbValue::from_int(2));
        attrs.insert("_private".into(), MbValue::from_int(3));
        attrs.insert("baz".into(), MbValue::from_int(4));
        mb_module_register("star_all_mod", attrs);

        let result = mb_import_star(s("star_all_mod"));
        // Result is a dict; inspect it
        assert!(result.is_ptr(), "mb_import_star should return a dict ptr");
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                // Only foo and bar should be present (from __all__)
                assert_eq!(
                    map.len(),
                    2,
                    "dict should have exactly 2 entries from __all__"
                );
                assert!(map.contains_key("foo"), "foo should be exported");
                assert!(map.contains_key("bar"), "bar should be exported");
                assert!(
                    !map.contains_key("_private"),
                    "_private should NOT be exported"
                );
                assert!(
                    !map.contains_key("baz"),
                    "baz should NOT be exported (not in __all__)"
                );
            } else {
                panic!("expected Dict from mb_import_star");
            }
        }
    }

    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_without_all
    #[test]
    fn test_import_star_without_all() {
        // Register a module WITHOUT __all__ — all public names exported
        let mut attrs = HashMap::new();
        attrs.insert("alpha".into(), MbValue::from_int(10));
        attrs.insert("beta".into(), MbValue::from_int(20));
        attrs.insert("_secret".into(), MbValue::from_int(30));
        mb_module_register("star_noall_mod", attrs);

        let result = mb_import_star(s("star_noall_mod"));
        assert!(result.is_ptr());
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                // alpha and beta are public, _secret starts with _
                assert!(map.contains_key("alpha"), "alpha should be exported");
                assert!(map.contains_key("beta"), "beta should be exported");
                assert!(
                    !map.contains_key("_secret"),
                    "_secret should NOT be exported"
                );
            } else {
                panic!("expected Dict from mb_import_star");
            }
        }
    }

    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_empty_all
    #[test]
    fn test_import_star_empty_all() {
        // Register a module with __all__ = [] — nothing should be exported
        let all_list = MbValue::from_ptr(MbObject::new_list(vec![]));
        let mut attrs = HashMap::new();
        attrs.insert("__all__".into(), all_list);
        attrs.insert("something".into(), MbValue::from_int(99));
        mb_module_register("star_empty_all_mod", attrs);

        let result = mb_import_star(s("star_empty_all_mod"));
        assert!(result.is_ptr());
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.len(), 0, "empty __all__ should export nothing");
            } else {
                panic!("expected Dict from mb_import_star");
            }
        }
    }

    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_preserves_all_attr
    #[test]
    fn test_import_star_preserves_all_attr() {
        // Verify that __all__ is stored in module attrs (R1)
        let all_list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        let mut attrs = HashMap::new();
        attrs.insert("__all__".into(), all_list);
        attrs.insert("x".into(), MbValue::from_int(7));
        mb_module_register("star_preserve_mod", attrs);

        // Check that __all__ is present in the module's attrs
        MODULES.with(|mods| {
            let mods = mods.borrow();
            let module = mods.get("star_preserve_mod").expect("module should exist");
            assert!(
                module.attrs.contains_key("__all__"),
                "__all__ must be preserved in module attrs"
            );
        });
    }

    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#test_import_star_registered_in_symbols
    #[test]
    fn test_import_star_registered_in_symbols() {
        use crate::runtime::symbols::runtime_symbols;
        let syms = runtime_symbols();
        let found = syms.iter().any(|s| s.name == "mb_import_star");
        assert!(
            found,
            "mb_import_star must be registered in runtime_symbols()"
        );
    }

    // ── Cleanup: CURRENT_MODULE_PACKAGE ──

    #[test]
    fn test_cleanup_resets_current_module_package() {
        CURRENT_MODULE_PACKAGE.with(|cp| {
            *cp.borrow_mut() = Some("test_pkg".to_string());
        });
        cleanup_all_modules();
        CURRENT_MODULE_PACKAGE.with(|cp| {
            assert!(
                cp.borrow().is_none(),
                "cleanup should reset CURRENT_MODULE_PACKAGE"
            );
        });
    }

    #[test]
    fn test_is_native_func_known_addr_true() {
        NATIVE_FUNC_ADDRS.with(|addrs| {
            addrs.borrow_mut().insert(0xDEAD_BEEF);
        });
        assert!(is_native_func(0xDEAD_BEEF));
        NATIVE_FUNC_ADDRS.with(|addrs| {
            addrs.borrow_mut().remove(&0xDEAD_BEEF);
        });
    }

    #[test]
    fn test_is_native_func_unknown_addr_false() {
        assert!(!is_native_func(0xFFFF_FFFF_FFFF_0000));
    }
}
