use super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::value::MbValue;
use rustc_hash::FxHashMap;
/// Closure and nested function support for the Mamba runtime (#289).
///
/// Closures capture variables from their enclosing scope. This module
/// provides the runtime infrastructure for:
/// - Creating closure objects with captured environments
/// - Accessing captured variables from within closures
/// - Decorator application (wrapping functions)
use std::collections::HashMap;

/// A closure object — a function paired with its captured environment.
pub struct MbClosure {
    /// Name of the function
    pub name: String,
    /// Captured variables: name → MbValue
    pub captures: Vec<MbValue>,
    /// The function pointer (compiled code entry point).
    /// In practice, this is a MbValue pointing to a Function object.
    pub func: MbValue,
    /// Default argument values, evaluated at lambda/function creation time.
    /// Python semantics: defaults fill in missing positional args from the right.
    /// For `lambda x, y=10, z=20`: defaults = [10, 20] (for y and z).
    /// When `mb_call0` receives a closure with `defaults.len() == arity`, it
    /// calls the underlying function with the defaults as all arguments.
    pub defaults: Vec<MbValue>,
    /// Total parameter count. Set when defaults are partial so the call
    /// dispatch (`mb_call1_val` etc.) can decide how many defaults to consume
    /// to fill missing trailing params. 0 means "unset / not relevant".
    pub arity: usize,
}

// Thread-local closure storage — Vec-indexed by closure ID for O(1) lookup (#1199).
// ID N maps to index N-1 (IDs start at 1 for compatibility with MbValue::from_int).
thread_local! {
    static CLOSURES: std::cell::RefCell<Vec<Option<MbClosure>>> =
        std::cell::RefCell::new(Vec::new());
}

// ── Closure Creation ──

/// Create a new closure capturing the given variables.
pub fn mb_closure_new(name: MbValue, func: MbValue, captures: MbValue) -> MbValue {
    let closure_name = extract_str(name).unwrap_or_else(|| "<closure>".to_string());
    let captured_vars = extract_list(captures);

    let closure = MbClosure {
        name: closure_name,
        captures: captured_vars,
        func,
        defaults: Vec::new(),
        arity: 0,
    };
    CLOSURES.with(|closures| {
        let mut vec = closures.borrow_mut();
        let id = (vec.len() + 1) as u64; // IDs start at 1
        vec.push(Some(closure));
        MbValue::from_int(id as i64)
    })
}

/// Set default argument values on an existing closure.
/// Called at lambda creation time after `mb_closure_new` to freeze default-arg
/// expressions into the closure. Takes a list MbValue whose elements are the
/// evaluated default values, in parameter order (defaults fill trailing params).
pub fn mb_closure_set_defaults(closure_handle: MbValue, defaults_list: MbValue) {
    if let Some(id) = closure_handle.as_int() {
        let vals = extract_list(defaults_list);
        CLOSURES.with(|closures| {
            let mut vec = closures.borrow_mut();
            let idx = (id as u64).wrapping_sub(1) as usize;
            if let Some(Some(c)) = vec.get_mut(idx) {
                // REQ: R1 — release prior defaults before overwriting
                for old_val in c.defaults.drain(..) {
                    unsafe {
                        super::rc::release_if_ptr(old_val);
                    }
                }
                c.defaults = vals;
            }
        });
    }
}

/// Set the total parameter count on a closure. Codegen emits this whenever
/// a function/lambda has at least one default value, so the dispatcher can
/// fill missing trailing args from `defaults`.
pub fn mb_closure_set_arity(closure_handle: MbValue, arity: MbValue) {
    if let (Some(id), Some(n)) = (closure_handle.as_int(), arity.as_int()) {
        CLOSURES.with(|closures| {
            let mut vec = closures.borrow_mut();
            let idx = (id as u64).wrapping_sub(1) as usize;
            if let Some(Some(c)) = vec.get_mut(idx) {
                c.arity = n.max(0) as usize;
            }
        });
    }
}

/// Get the recorded arity for a closure (0 = unset).
pub fn closure_arity(closure_handle: MbValue) -> usize {
    if let Some(id) = closure_handle.as_int() {
        CLOSURES.with(|closures| {
            let vec = closures.borrow();
            let idx = (id as u64).wrapping_sub(1) as usize;
            vec.get(idx)
                .and_then(|slot| slot.as_ref())
                .map(|c| c.arity)
                .unwrap_or(0)
        })
    } else {
        0
    }
}

/// Get a clone of the default argument values for a closure. Returns an
/// empty Vec if the closure has no defaults or the handle is invalid.
pub fn closure_defaults(closure_handle: MbValue) -> Vec<MbValue> {
    if let Some(id) = closure_handle.as_int() {
        CLOSURES.with(|closures| {
            let vec = closures.borrow();
            let idx = (id as u64).wrapping_sub(1) as usize;
            vec.get(idx)
                .and_then(|slot| slot.as_ref())
                .map(|c| c.defaults.clone())
                .unwrap_or_default()
        })
    } else {
        Vec::new()
    }
}

/// Get a captured variable by index.
pub fn mb_closure_get_capture(closure_handle: MbValue, index: MbValue) -> MbValue {
    if let (Some(id), Some(idx)) = (closure_handle.as_int(), index.as_int()) {
        CLOSURES.with(|closures| {
            let vec = closures.borrow();
            let slot_idx = (id as u64).wrapping_sub(1) as usize;
            let val = vec
                .get(slot_idx)
                .and_then(|slot| slot.as_ref())
                .and_then(|c| c.captures.get(idx as usize).copied())
                .unwrap_or(MbValue::none());
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            val
        })
    } else {
        MbValue::none()
    }
}

/// Set a captured variable by index (for mutable closures).
pub fn mb_closure_set_capture(closure_handle: MbValue, index: MbValue, value: MbValue) {
    if let (Some(id), Some(idx)) = (closure_handle.as_int(), index.as_int()) {
        CLOSURES.with(|closures| {
            let mut vec = closures.borrow_mut();
            let slot_idx = (id as u64).wrapping_sub(1) as usize;
            if let Some(Some(c)) = vec.get_mut(slot_idx) {
                let idx = idx as usize;
                if idx >= c.captures.len() {
                    c.captures.resize(idx + 1, MbValue::none());
                }
                // REQ: R1 — release prior capture value before overwriting
                unsafe {
                    super::rc::release_if_ptr(c.captures[idx]);
                }
                c.captures[idx] = value;
            }
        });
    }
}

/// Get the underlying function of a closure.
pub fn mb_closure_get_func(closure_handle: MbValue) -> MbValue {
    if let Some(id) = closure_handle.as_int() {
        CLOSURES.with(|closures| {
            let vec = closures.borrow();
            let idx = (id as u64).wrapping_sub(1) as usize;
            let val = vec
                .get(idx)
                .and_then(|slot| slot.as_ref())
                .map(|c| c.func)
                .unwrap_or(MbValue::none());
            // REQ: R2 — retain returned func, symmetric with mb_closure_get_capture
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            val
        })
    } else {
        MbValue::none()
    }
}

/// Release a closure's resources, cascading rc releases on captures and
/// defaults so heap values referenced only via the closure get freed.
pub fn mb_closure_release(closure_handle: MbValue) {
    if let Some(id) = closure_handle.as_int() {
        let dead = CLOSURES.with(|closures| {
            let mut vec = closures.borrow_mut();
            let idx = (id as u64).wrapping_sub(1) as usize;
            if idx < vec.len() {
                vec[idx].take()
            } else {
                None
            }
        });
        if let Some(closure) = dead {
            for cap in closure.captures {
                unsafe {
                    super::rc::release_if_ptr(cap);
                }
            }
            for def in closure.defaults {
                unsafe {
                    super::rc::release_if_ptr(def);
                }
            }
        }
    }
}

// ── Decorator Support (#294) ──

/// Apply a decorator to a function: decorator(func) → wrapped_func.
/// This is a generic dispatch — the decorator is called as a function.
pub fn mb_apply_decorator(decorator: MbValue, func: MbValue) -> MbValue {
    // In the compiled code, a decorator is just a function call:
    // @decorator
    // def foo(): ...
    //
    // becomes: foo = decorator(foo)
    //
    // The runtime just needs to support calling the decorator.
    // The actual call is handled by the compiled code's Call instruction.
    // This function is a marker for the lowering pass to emit the right code.

    // For runtime decorators that are already callable:
    // Return a placeholder indicating "call decorator with func"
    // The actual calling happens in compiled code

    // Simple built-in decorator support:
    // @staticmethod, @classmethod, @property are handled here
    if let Some(ptr) = decorator.as_ptr() {
        unsafe {
            if let ObjData::Str(ref name) = (*ptr).data {
                match name.as_str() {
                    "staticmethod" => {
                        // Mark function as static (no self parameter manipulation)
                        return func;
                    }
                    "classmethod" => {
                        // Mark function as classmethod
                        // In practice, wrap it to pass cls as first arg
                        return func;
                    }
                    "property" => {
                        // Create a property descriptor
                        return mb_property_new(func, MbValue::none(), MbValue::none());
                    }
                    _ => {}
                }
            }
        }
    }

    // Default: return func (decorator is applied in compiled code)
    func
}

/// Apply a decorator stack: @dec1 @dec2 @dec3 def foo → dec1(dec2(dec3(foo)))
pub fn mb_apply_decorators(func: MbValue, decorators: MbValue) -> MbValue {
    if let Some(ptr) = decorators.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let decs = lock.read().unwrap();
                let mut result = func;
                // Apply in reverse order (innermost first)
                for dec in decs.iter().rev() {
                    result = mb_apply_decorator(*dec, result);
                }
                return result;
            }
        }
    }
    func
}

// ── Property Descriptor ──

/// Create a property descriptor.
pub fn mb_property_new(fget: MbValue, fset: MbValue, fdel: MbValue) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("fget".to_string(), fget);
    fields.insert("fset".to_string(), fset);
    fields.insert("fdel".to_string(), fdel);
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "property".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ── Function name registry (for __name__ attribute) ──

thread_local! {
    static FUNC_NAMES: std::cell::RefCell<HashMap<u64, String>> =
        std::cell::RefCell::new(HashMap::new());
    static FUNC_DOCS: std::cell::RefCell<HashMap<u64, String>> =
        std::cell::RefCell::new(HashMap::new());
    static FUNC_MODULES: std::cell::RefCell<HashMap<u64, String>> =
        std::cell::RefCell::new(HashMap::new());
    // Code-object introspection metadata (CORE #3): positional argument count
    // (`co_argcount`) and local/parameter variable names (`co_varnames`) for
    // each user-defined function, keyed by the function value's bits. Populated
    // at module init alongside FUNC_NAMES so `f.__code__.co_argcount` and
    // `f.__code__.co_varnames` return the real compiled signature metadata.
    static FUNC_ARGCOUNTS: std::cell::RefCell<HashMap<u64, i64>> =
        std::cell::RefCell::new(HashMap::new());
    static FUNC_VARNAMES: std::cell::RefCell<HashMap<u64, Vec<String>>> =
        std::cell::RefCell::new(HashMap::new());
    // Declared-signature metadata for `inspect.signature` (FUNC_PARAMS) and
    // the textual return annotation (FUNC_RET_ANNOS). Populated at module
    // init via mb_func_set_params / mb_func_set_retanno emitted by lowering.
    static FUNC_PARAMS: std::cell::RefCell<HashMap<u64, Vec<MbParamInfo>>> =
        std::cell::RefCell::new(HashMap::new());
    static FUNC_RET_ANNOS: std::cell::RefCell<HashMap<u64, String>> =
        std::cell::RefCell::new(HashMap::new());
    // Source location metadata: first line number of the `def`/`lambda` and
    // the source filename. Primed at module init via mb_func_set_srcinfo so
    // `f.__code__.co_firstlineno` / `.co_filename` report real locations.
    static FUNC_LINES: std::cell::RefCell<HashMap<u64, i64>> =
        std::cell::RefCell::new(HashMap::new());
    static FUNC_FILES: std::cell::RefCell<HashMap<u64, String>> =
        std::cell::RefCell::new(HashMap::new());
}

/// One declared parameter as recorded for introspection.
#[derive(Clone)]
pub struct MbParamInfo {
    pub name: String,
    /// CPython `inspect.Parameter` kind ordinal: 0 POSITIONAL_ONLY,
    /// 1 POSITIONAL_OR_KEYWORD, 2 VAR_POSITIONAL, 3 KEYWORD_ONLY,
    /// 4 VAR_KEYWORD.
    pub kind: u8,
    pub has_default: bool,
    /// Default value (None-MbValue when has_default is false or the literal
    /// was not representable at lowering time).
    pub default: MbValue,
    /// Textual annotation (`"int"`), None when un-annotated.
    pub annotation: Option<String>,
}

/// Register a function's declared parameters. `params` is a list of
/// (name, kind, has_default, default, annotation) tuples — see the
/// lower_top_level priming loop in hir_to_mir.rs.
pub fn mb_func_set_params(func: MbValue, params: MbValue) {
    let mut infos: Vec<MbParamInfo> = Vec::new();
    if let Some(ptr) = params.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                for item in lock.read().unwrap().iter() {
                    let Some(tp) = item.as_ptr() else { continue };
                    let ObjData::Tuple(ref elems) = (*tp).data else {
                        continue;
                    };
                    if elems.len() < 5 {
                        continue;
                    }
                    let name = extract_str(elems[0]).unwrap_or_default();
                    let kind = elems[1].as_int().unwrap_or(1).clamp(0, 4) as u8;
                    let has_default = elems[2].as_int().unwrap_or(0) != 0;
                    let default = elems[3];
                    super::rc::retain_if_ptr(default);
                    let annotation = extract_str(elems[4]);
                    infos.push(MbParamInfo {
                        name,
                        kind,
                        has_default,
                        default,
                        annotation,
                    });
                }
            }
        }
    }
    let key = func.to_bits();
    FUNC_PARAMS.with(|m| {
        if let Some(prev) = m.borrow_mut().insert(key, infos) {
            for p in prev {
                unsafe {
                    super::rc::release_if_ptr(p.default);
                }
            }
        }
    });
}

/// Register a function's textual return annotation.
pub fn mb_func_set_retanno(func: MbValue, anno: MbValue) {
    if let Some(s) = extract_str(anno) {
        let key = func.to_bits();
        FUNC_RET_ANNOS.with(|m| m.borrow_mut().insert(key, s));
    }
}

/// Declared parameters for a registered function, or None when unknown.
pub fn func_params(func: MbValue) -> Option<Vec<MbParamInfo>> {
    let key = func.to_bits();
    FUNC_PARAMS.with(|m| m.borrow().get(&key).cloned())
}

/// Textual return annotation for a registered function, or None.
pub fn func_ret_anno(func: MbValue) -> Option<String> {
    let key = func.to_bits();
    FUNC_RET_ANNOS.with(|m| m.borrow().get(&key).cloned())
}

/// Build a function's `__annotations__` dict from its registered parameter and
/// return annotations (PEP 3107 / 526). Values are the textual annotations,
/// matching mamba's module- and class-level `__annotations__`. Returns
/// None-MbValue when the function is unregistered, an (possibly empty) dict
/// otherwise — CPython exposes `__annotations__` on every function.
pub fn mb_func_get_annotations(func: MbValue) -> MbValue {
    let key = func.to_bits();
    let known = FUNC_PARAMS.with(|m| m.borrow().contains_key(&key))
        || FUNC_RET_ANNOS.with(|m| m.borrow().contains_key(&key));
    if !known {
        return MbValue::none();
    }
    let dict = MbValue::from_ptr(MbObject::new_dict());
    if let Some(params) = func_params(func) {
        for p in params {
            if let Some(anno) = p.annotation {
                let k = MbValue::from_ptr(MbObject::new_str(p.name.clone()));
                let v = MbValue::from_ptr(MbObject::new_str(anno));
                super::dict_ops::mb_dict_setitem(dict, k, v);
            }
        }
    }
    if let Some(ret) = func_ret_anno(func) {
        let k = MbValue::from_ptr(MbObject::new_str("return".to_string()));
        let v = MbValue::from_ptr(MbObject::new_str(ret));
        super::dict_ops::mb_dict_setitem(dict, k, v);
    }
    dict
}

/// Register a function's name (called at definition time so `f.__name__` works).
pub fn mb_func_set_name(func: MbValue, name: MbValue) {
    let fname = extract_str(name).unwrap_or_default();
    let key = func.to_bits();
    FUNC_NAMES.with(|m| m.borrow_mut().insert(key, fname));
}

/// Get a function's registered name. Returns None-MbValue if not registered.
pub fn mb_func_get_name(func: MbValue) -> MbValue {
    let key = func.to_bits();
    FUNC_NAMES.with(|m| {
        m.borrow()
            .get(&key)
            .map(|s| MbValue::from_ptr(MbObject::new_str(s.clone())))
            .unwrap_or(MbValue::none())
    })
}

/// Register a function's docstring (for `f.__doc__`). Called at module init
/// for every top-level def whose body starts with a bare string literal.
pub fn mb_func_set_doc(func: MbValue, doc: MbValue) {
    let fdoc = extract_str(doc).unwrap_or_default();
    let key = func.to_bits();
    FUNC_DOCS.with(|m| m.borrow_mut().insert(key, fdoc));
}

/// Get a function's registered docstring. Returns None-MbValue if not registered.
pub fn mb_func_get_doc(func: MbValue) -> MbValue {
    let key = func.to_bits();
    FUNC_DOCS.with(|m| {
        m.borrow()
            .get(&key)
            .map(|s| MbValue::from_ptr(MbObject::new_str(s.clone())))
            .unwrap_or(MbValue::none())
    })
}

/// Register a function's module name (for `f.__module__`).
pub fn mb_func_set_module(func: MbValue, module: MbValue) {
    let module_name = extract_str(module).unwrap_or_default();
    let key = func.to_bits();
    FUNC_MODULES.with(|m| m.borrow_mut().insert(key, module_name));
}

/// Get a function's registered module. Returns None-MbValue if not registered.
pub fn mb_func_get_module(func: MbValue) -> MbValue {
    let key = func.to_bits();
    FUNC_MODULES.with(|m| {
        m.borrow()
            .get(&key)
            .map(|s| MbValue::from_ptr(MbObject::new_str(s.clone())))
            .unwrap_or(MbValue::none())
    })
}

/// Register a function's positional argument count (for `f.__code__.co_argcount`).
/// Called at module init for every user-defined `def`. `argcount` excludes
/// `*args` / `**kwargs` (CPython counts those in co_varnames but not co_argcount).
pub fn mb_func_set_argcount(func: MbValue, argcount: MbValue) {
    let n = argcount.as_int().unwrap_or(0);
    let key = func.to_bits();
    FUNC_ARGCOUNTS.with(|m| m.borrow_mut().insert(key, n));
}

/// Get a function's registered argument count. Returns None-MbValue if not
/// registered (so callers can distinguish a real `def` from an arbitrary value).
pub fn mb_func_get_argcount(func: MbValue) -> MbValue {
    let key = func.to_bits();
    FUNC_ARGCOUNTS.with(|m| {
        m.borrow()
            .get(&key)
            .map(|n| MbValue::from_int(*n))
            .unwrap_or(MbValue::none())
    })
}

/// Register a function's local variable names (for `f.__code__.co_varnames`).
/// The list is the parameter names in declaration order (CPython also appends
/// other locals, but parameters come first and are what fixtures assert on).
/// Names are passed packed as a tuple/list MbValue of strings.
pub fn mb_func_set_varnames(func: MbValue, names: MbValue) {
    let mut collected: Vec<String> = Vec::new();
    if let Some(ptr) = names.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => {
                    for item in items.iter() {
                        if let Some(s) = extract_str(*item) {
                            collected.push(s);
                        }
                    }
                }
                ObjData::List(lock) => {
                    let items = lock.read().unwrap();
                    for item in items.iter() {
                        if let Some(s) = extract_str(*item) {
                            collected.push(s);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let key = func.to_bits();
    FUNC_VARNAMES.with(|m| m.borrow_mut().insert(key, collected));
}

/// Get a function's registered varnames as a tuple MbValue (CPython returns a
/// tuple for `co_varnames`). Returns None-MbValue if not registered.
pub fn mb_func_get_varnames(func: MbValue) -> MbValue {
    let key = func.to_bits();
    FUNC_VARNAMES.with(|m| {
        m.borrow()
            .get(&key)
            .map(|names| {
                let items: Vec<MbValue> = names
                    .iter()
                    .map(|s| MbValue::from_ptr(MbObject::new_str(s.clone())))
                    .collect();
                MbValue::from_ptr(MbObject::new_tuple(items))
            })
            .unwrap_or(MbValue::none())
    })
}

/// True if `func` is a registered user-defined function (present in any of the
/// function metadata registries). Used to gate `__code__` synthesis so we don't
/// fabricate a code object for arbitrary ints / pointers.
pub fn mb_func_is_registered(func: MbValue) -> bool {
    let key = func.to_bits();
    FUNC_NAMES.with(|m| m.borrow().contains_key(&key))
        || FUNC_ARGCOUNTS.with(|m| m.borrow().contains_key(&key))
        || FUNC_VARNAMES.with(|m| m.borrow().contains_key(&key))
}

/// Register a function's source location (`co_firstlineno` / `co_filename`).
/// Called at module init alongside the other metadata priming calls; lambdas
/// register at closure-creation time.
pub fn mb_func_set_srcinfo(func: MbValue, line: MbValue, filename: MbValue) {
    let key = func.to_bits();
    if let Some(n) = line.as_int() {
        if n > 0 {
            FUNC_LINES.with(|m| m.borrow_mut().insert(key, n));
        }
    }
    if let Some(f) = extract_str(filename) {
        if !f.is_empty() {
            FUNC_FILES.with(|m| m.borrow_mut().insert(key, f));
        }
    }
}

/// First source line of a registered function, or None when unknown.
pub fn func_line(func: MbValue) -> Option<i64> {
    let key = func.to_bits();
    FUNC_LINES.with(|m| m.borrow().get(&key).copied())
}

/// Source filename of a registered function, or None when unknown.
pub fn func_file(func: MbValue) -> Option<String> {
    let key = func.to_bits();
    FUNC_FILES.with(|m| m.borrow().get(&key).cloned())
}

// ── Cell Variables (for nonlocal/closure mutable capture) ──
// Vec-indexed by cell ID for O(1) lookup (#1199). ID N maps to index N-1.

thread_local! {
    static CELLS: std::cell::RefCell<Vec<Option<MbValue>>> =
        std::cell::RefCell::new(Vec::new());
}

/// Create a new cell variable initialized with a value.
/// Returns a handle (integer) that can be shared between scopes.
pub fn mb_cell_new(value: MbValue) -> MbValue {
    // Fix C-prime: CELLS takes its own +1 so JIT epilogue release of the
    // source VReg cannot UAF the raw reference stored in the cell slot.
    // Mirrors the symmetric pattern in mb_cell_set.
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    CELLS.with(|cells| {
        let mut vec = cells.borrow_mut();
        let id = (vec.len() + 1) as u64; // IDs start at 1
        vec.push(Some(value));
        MbValue::from_int(id as i64)
    })
}

/// Get the value stored in a cell.
pub fn mb_cell_get(cell_handle: MbValue) -> MbValue {
    if let Some(id) = cell_handle.as_int() {
        CELLS.with(|cells| {
            let vec = cells.borrow();
            let idx = (id as u64).wrapping_sub(1) as usize;
            let val = vec
                .get(idx)
                .and_then(|slot| *slot)
                .unwrap_or(MbValue::none());
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            val
        })
    } else {
        MbValue::none()
    }
}

/// Set the value stored in a cell.
pub fn mb_cell_set(cell_handle: MbValue, value: MbValue) {
    if let Some(id) = cell_handle.as_int() {
        // Retain so value survives the JIT epilogue releasing the source VReg.
        unsafe {
            super::rc::retain_if_ptr(value);
        }
        CELLS.with(|cells| {
            let mut vec = cells.borrow_mut();
            let idx = (id as u64).wrapping_sub(1) as usize;
            if idx < vec.len() {
                // Release the old cell value being overwritten.
                if let Some(prev) = vec[idx] {
                    unsafe {
                        super::rc::release_if_ptr(prev);
                    }
                }
                vec[idx] = Some(value);
            }
        });
    }
}

// ── nonlocal/global support ──

// Thread-local global namespace for the current module.
thread_local! {
    static GLOBAL_NAMESPACE: std::cell::RefCell<HashMap<String, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
    static GLOBAL_ID_NAMESPACE: std::cell::RefCell<HashMap<i64, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Get a global variable by name.
pub fn mb_global_get(name: MbValue) -> MbValue {
    let var_name = extract_str(name).unwrap_or_default();
    GLOBAL_NAMESPACE.with(|ns| {
        let val = ns
            .borrow()
            .get(&var_name)
            .copied()
            .unwrap_or(MbValue::none());
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        val
    })
}

/// Set a global variable.
pub fn mb_global_set(name: MbValue, value: MbValue) {
    let var_name = extract_str(name).unwrap_or_default();
    // Retain so value survives the JIT epilogue releasing the source VReg.
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    GLOBAL_NAMESPACE.with(|ns| {
        let old = ns.borrow_mut().insert(var_name, value);
        if let Some(prev) = old {
            unsafe {
                super::rc::release_if_ptr(prev);
            }
        }
    });
}

/// Get a global variable by integer id (SymbolId). Used by REPL since
/// MirConst::Str is not yet compiled to actual string pointers.
/// The id is passed as raw i64 (not NaN-boxed).
pub fn mb_global_get_id(id: MbValue) -> MbValue {
    let key = id.to_bits() as i64;
    GLOBAL_ID_NAMESPACE.with(|ns| {
        let val = ns.borrow().get(&key).copied().unwrap_or(MbValue::none());
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        val
    })
}

/// Set a global variable by integer id (SymbolId).
/// The id is passed as raw i64 (not NaN-boxed).
pub fn mb_global_set_id(id: MbValue, value: MbValue) {
    let key = id.to_bits() as i64;
    // Retain the value so it survives the JIT epilogue releasing the source VReg.
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    GLOBAL_ID_NAMESPACE.with(|ns| {
        let old = ns.borrow_mut().insert(key, value);
        // Release the previous value being overwritten.
        if let Some(prev) = old {
            unsafe {
                super::rc::release_if_ptr(prev);
            }
        }
    });
}

/// Delete a global variable by integer id (SymbolId).
/// The id is passed as raw i64 (not NaN-boxed).
pub fn mb_global_del_id(id: MbValue) {
    let key = id.to_bits() as i64;
    GLOBAL_ID_NAMESPACE.with(|ns| {
        let old = ns.borrow_mut().remove(&key);
        if let Some(prev) = old {
            unsafe { super::rc::release_if_ptr(prev); }
        }
    });
}

// ── Helpers ──

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn extract_list(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                return lock.read().unwrap().to_vec();
            }
        }
    }
    Vec::new()
}

// ── Global namespace save/restore for module import isolation (#1190) ──

/// Save and clear the current GLOBAL_ID_NAMESPACE, returning the previous contents.
/// Used by module import to isolate module-level globals from the caller's globals.
pub fn save_and_clear_global_id_namespace() -> HashMap<i64, MbValue> {
    GLOBAL_ID_NAMESPACE.with(|ns| {
        let mut ns = ns.borrow_mut();
        let saved = ns.clone();
        ns.clear();
        saved
    })
}

/// Restore a previously saved GLOBAL_ID_NAMESPACE.
pub fn restore_global_id_namespace(saved: HashMap<i64, MbValue>) {
    GLOBAL_ID_NAMESPACE.with(|ns| {
        *ns.borrow_mut() = saved;
    });
}

/// Snapshot the current GLOBAL_ID_NAMESPACE (non-destructive).
pub fn snapshot_global_id_namespace() -> HashMap<i64, MbValue> {
    GLOBAL_ID_NAMESPACE.with(|ns| ns.borrow().clone())
}

/// Merge entries into the current GLOBAL_ID_NAMESPACE without clearing it.
///
/// Used after an imported module finishes executing: its module-level global
/// bindings (constants and `import` results) are merged back so the module's
/// own functions can read them when invoked later from the caller. SymbolIds
/// are unique per compilation, so these keys never collide with the caller's.
/// Existing entries are not overwritten (the caller's namespace wins on the
/// off chance of a shared id), and contained pointer values are retained.
pub fn merge_global_id_namespace(entries: &HashMap<i64, MbValue>) {
    GLOBAL_ID_NAMESPACE.with(|ns| {
        let mut ns = ns.borrow_mut();
        for (k, v) in entries {
            ns.entry(*k).or_insert_with(|| {
                unsafe {
                    super::rc::retain_if_ptr(*v);
                }
                *v
            });
        }
    });
}

// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-closure-rs" tracker="standardize-gap-projects-mamba-src-runtime-closure-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
// Runtime SymbolId → (name, type-tag) registry for `globals()` / `locals()`.
// Populated by the driver (driver/mod.rs::Driver::run/run_stdin) and by
// module imports (module.rs) before the JIT entry point runs. The type tag
// records just enough info for `mb_globals` to NaN-box raw GLOBAL_ID_NAMESPACE
// values without needing a TypeContext at runtime.
//
// Function pointers (user-defined functions / closures) are tracked in
// MODULE_FUNC_INFO — they live outside GLOBAL_ID_NAMESPACE because the JIT
// calls them via direct CallExtern, not through globals.
// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#globals_impl

#[derive(Clone, Copy, Debug)]
pub enum SymTy {
    Int,
    Float,
    Bool,
    /// Already-boxed values (str, list, dict, instance, etc.) — no NaN-boxing
    /// fixup needed; the JIT writes a proper MbValue directly.
    Boxed,
}

thread_local! {
    /// SymbolId.0 → (name, type tag). Used by mb_globals/mb_locals.
    static MODULE_SYM_INFO: std::cell::RefCell<HashMap<i64, (String, SymTy)>> =
        std::cell::RefCell::new(HashMap::new());
    /// User-defined function/closure name → MbValue (TAG_FUNC pointer).
    /// Populated alongside MODULE_SYM_INFO so functions show up in globals().
    static MODULE_FUNC_INFO: std::cell::RefCell<HashMap<String, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Replace the current MODULE_SYM_INFO with a new one.
pub fn set_module_sym_info(info: HashMap<i64, (String, SymTy)>) {
    MODULE_SYM_INFO.with(|m| *m.borrow_mut() = info);
}

/// Replace the current MODULE_FUNC_INFO with a new one.
pub fn set_module_func_info(info: HashMap<String, MbValue>) {
    MODULE_FUNC_INFO.with(|m| *m.borrow_mut() = info);
}

/// Snapshot for save_and_restore around module imports.
pub fn save_and_clear_module_sym_info() -> (HashMap<i64, (String, SymTy)>, HashMap<String, MbValue>)
{
    let syms = MODULE_SYM_INFO.with(|m| {
        let mut b = m.borrow_mut();
        let saved = b.clone();
        b.clear();
        saved
    });
    let funcs = MODULE_FUNC_INFO.with(|m| {
        let mut b = m.borrow_mut();
        let saved = b.clone();
        b.clear();
        saved
    });
    (syms, funcs)
}

/// Restore a previously saved sym_info / func_info pair.
pub fn restore_module_sym_info(saved: (HashMap<i64, (String, SymTy)>, HashMap<String, MbValue>)) {
    MODULE_SYM_INFO.with(|m| *m.borrow_mut() = saved.0);
    MODULE_FUNC_INFO.with(|m| *m.borrow_mut() = saved.1);
}

/// Build a dict containing the current module's globals, drawing from
/// MODULE_SYM_INFO + GLOBAL_ID_NAMESPACE + MODULE_FUNC_INFO. Skips dunder
/// names except the standard CPython-visible ones.
pub fn build_globals_dict() -> MbValue {
    use super::dict_ops;
    let dict = dict_ops::mb_dict_new();

    let id_ns = GLOBAL_ID_NAMESPACE.with(|ns| ns.borrow().clone());
    let sym_info = MODULE_SYM_INFO.with(|m| m.borrow().clone());

    for (id, raw) in &id_ns {
        let Some((name, ty)) = sym_info.get(id) else {
            continue;
        };
        if name.starts_with("__") && name != "__name__" && name != "__doc__" && name != "__all__" {
            continue;
        }
        let boxed = match ty {
            SymTy::Int => {
                if raw.as_int().is_some() {
                    *raw
                } else {
                    let raw_i64 = raw.to_bits() as i64;
                    if (-(1i64 << 47)..(1i64 << 47)).contains(&raw_i64) {
                        MbValue::from_int(raw_i64)
                    } else {
                        *raw
                    }
                }
            }
            SymTy::Float => MbValue::from_float(f64::from_bits(raw.to_bits())),
            SymTy::Bool => {
                if raw.is_bool() {
                    *raw
                } else {
                    MbValue::from_bool(raw.to_bits() != 0)
                }
            }
            SymTy::Boxed => *raw,
        };
        let key = MbValue::from_ptr(super::rc::MbObject::new_str(name.clone()));
        dict_ops::mb_dict_setitem(dict, key, boxed);
    }

    let func_info = MODULE_FUNC_INFO.with(|m| m.borrow().clone());
    for (name, fv) in &func_info {
        if name.starts_with("__") && name != "__name__" && name != "__doc__" && name != "__all__" {
            continue;
        }
        let key = MbValue::from_ptr(super::rc::MbObject::new_str(name.clone()));
        dict_ops::mb_dict_setitem(dict, key, *fv);
    }
    dict
}
// HANDWRITE-END

// ── Cleanup ──

/// Reset all closure-related thread_local state to defaults.
/// Called as part of centralized runtime cleanup between test executions.
/// Values are cleared without releasing — refcount imbalance from mixed
/// code paths makes release unsafe. Leaked objects reclaimed at process exit.
pub(crate) fn cleanup_all_closures() {
    let _ = CLOSURES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = CELLS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = GLOBAL_NAMESPACE.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = GLOBAL_ID_NAMESPACE.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_NAMES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_DOCS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_MODULES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_ARGCOUNTS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_VARNAMES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_PARAMS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_RET_ANNOS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_LINES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FUNC_FILES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure_create_and_capture() {
        let name = MbValue::from_ptr(MbObject::new_str("my_closure".to_string()));
        let func = MbValue::from_int(100); // placeholder
        let captures = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(42),
            MbValue::from_int(99),
        ]));

        let closure = mb_closure_new(name, func, captures);
        assert_eq!(
            mb_closure_get_capture(closure, MbValue::from_int(0)).as_int(),
            Some(42),
        );
        assert_eq!(
            mb_closure_get_capture(closure, MbValue::from_int(1)).as_int(),
            Some(99),
        );

        mb_closure_set_capture(closure, MbValue::from_int(0), MbValue::from_int(100));
        assert_eq!(
            mb_closure_get_capture(closure, MbValue::from_int(0)).as_int(),
            Some(100),
        );

        mb_closure_release(closure);
    }

    #[test]
    fn test_global_namespace() {
        let name = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_global_set(name, MbValue::from_int(42));
        let name2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert_eq!(mb_global_get(name2).as_int(), Some(42));
    }

    // ── Additional tests ──

    #[test]
    fn test_closure_get_func() {
        let name = MbValue::from_ptr(MbObject::new_str("fn_closure".into()));
        let func = MbValue::from_int(555);
        let captures = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, captures);
        assert_eq!(mb_closure_get_func(closure).as_int(), Some(555));
        mb_closure_release(closure);
    }

    #[test]
    fn test_closure_release_removes() {
        let name = MbValue::from_ptr(MbObject::new_str("temp".into()));
        let func = MbValue::from_int(1);
        let captures = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(7)]));
        let closure = mb_closure_new(name, func, captures);
        mb_closure_release(closure);
        // After release, get_capture should return none
        assert!(mb_closure_get_capture(closure, MbValue::from_int(0)).is_none());
        assert!(mb_closure_get_func(closure).is_none());
    }

    #[test]
    fn test_closure_set_capture_expands() {
        let name = MbValue::from_ptr(MbObject::new_str("grow".into()));
        let func = MbValue::from_int(1);
        let captures = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, captures);
        // Set index 5 on an empty captures vec -- should expand
        mb_closure_set_capture(closure, MbValue::from_int(5), MbValue::from_int(99));
        assert_eq!(
            mb_closure_get_capture(closure, MbValue::from_int(5)).as_int(),
            Some(99),
        );
        // Intermediate indices should be None
        assert!(mb_closure_get_capture(closure, MbValue::from_int(3)).is_none());
        mb_closure_release(closure);
    }

    #[test]
    fn test_closure_out_of_bounds_get_returns_none() {
        let name = MbValue::from_ptr(MbObject::new_str("oob".into()));
        let func = MbValue::from_int(1);
        let captures = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let closure = mb_closure_new(name, func, captures);
        assert!(mb_closure_get_capture(closure, MbValue::from_int(100)).is_none());
        mb_closure_release(closure);
    }

    #[test]
    fn test_closure_invalid_handle_returns_none() {
        let bad_handle = MbValue::from_int(999999);
        assert!(mb_closure_get_capture(bad_handle, MbValue::from_int(0)).is_none());
        assert!(mb_closure_get_func(bad_handle).is_none());
    }

    #[test]
    fn test_closure_non_int_handle() {
        let bad = MbValue::from_bool(true);
        assert!(mb_closure_get_capture(bad, MbValue::from_int(0)).is_none());
        assert!(mb_closure_get_func(bad).is_none());
        mb_closure_release(bad); // should not panic
    }

    #[test]
    fn test_closure_default_name() {
        // Pass non-string name, should default to "<closure>"
        let name = MbValue::from_int(0);
        let func = MbValue::from_int(1);
        let captures = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, captures);
        // Just verify it doesn't panic and we can get func
        assert_eq!(mb_closure_get_func(closure).as_int(), Some(1));
        mb_closure_release(closure);
    }

    #[test]
    fn test_global_get_missing_returns_none() {
        let name = MbValue::from_ptr(MbObject::new_str("nonexistent_var_xyz".into()));
        assert!(mb_global_get(name).is_none());
    }

    #[test]
    fn test_global_set_overwrite() {
        let name = MbValue::from_ptr(MbObject::new_str("overwrite_var".into()));
        mb_global_set(name, MbValue::from_int(1));
        let name2 = MbValue::from_ptr(MbObject::new_str("overwrite_var".into()));
        mb_global_set(name2, MbValue::from_int(2));
        let name3 = MbValue::from_ptr(MbObject::new_str("overwrite_var".into()));
        assert_eq!(mb_global_get(name3).as_int(), Some(2));
    }

    #[test]
    fn test_global_id_get_set() {
        let id = MbValue::from_bits(42);
        mb_global_set_id(id, MbValue::from_int(100));
        assert_eq!(mb_global_get_id(id).as_int(), Some(100));
    }

    #[test]
    fn test_global_id_missing_returns_none() {
        let id = MbValue::from_bits(99999);
        assert!(mb_global_get_id(id).is_none());
    }

    #[test]
    fn test_apply_decorator_staticmethod() {
        let dec = MbValue::from_ptr(MbObject::new_str("staticmethod".into()));
        let func = MbValue::from_int(42);
        let result = mb_apply_decorator(dec, func);
        assert_eq!(result, func); // staticmethod returns func unchanged
    }

    #[test]
    fn test_apply_decorator_classmethod() {
        let dec = MbValue::from_ptr(MbObject::new_str("classmethod".into()));
        let func = MbValue::from_int(42);
        let result = mb_apply_decorator(dec, func);
        assert_eq!(result, func);
    }

    #[test]
    fn test_apply_decorator_property() {
        let dec = MbValue::from_ptr(MbObject::new_str("property".into()));
        let func = MbValue::from_int(42);
        let result = mb_apply_decorator(dec, func);
        assert!(result.is_ptr()); // property creates an Instance
    }

    #[test]
    fn test_apply_decorator_unknown_returns_func() {
        let dec = MbValue::from_ptr(MbObject::new_str("unknown_dec".into()));
        let func = MbValue::from_int(42);
        let result = mb_apply_decorator(dec, func);
        assert_eq!(result, func);
    }

    #[test]
    fn test_cell_new_get_set() {
        let cell = mb_cell_new(MbValue::from_int(10));
        assert_eq!(mb_cell_get(cell).as_int(), Some(10));
        mb_cell_set(cell, MbValue::from_int(20));
        assert_eq!(mb_cell_get(cell).as_int(), Some(20));
    }

    #[test]
    fn test_cell_shared_between_handles() {
        // Two "scopes" sharing the same cell handle
        let cell = mb_cell_new(MbValue::from_int(0));
        let cell_copy = cell; // same handle value
        mb_cell_set(cell, MbValue::from_int(42));
        assert_eq!(mb_cell_get(cell_copy).as_int(), Some(42));
    }

    #[test]
    fn test_cell_invalid_handle() {
        let bad = MbValue::from_int(999999);
        assert!(mb_cell_get(bad).is_none());
    }

    #[test]
    fn test_property_new() {
        let fget = MbValue::from_int(10);
        let fset = MbValue::from_int(20);
        let fdel = MbValue::none();
        let prop = mb_property_new(fget, fset, fdel);
        assert!(prop.is_ptr());
        unsafe {
            let ptr = prop.as_ptr().unwrap();
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                assert_eq!(class_name, "property");
                let f = fields.read().unwrap();
                assert_eq!(f["fget"], fget);
                assert_eq!(f["fset"], fset);
            } else {
                panic!("expected Instance");
            }
        }
    }

    // ── Cleanup tests (R1: per-module cleanup for closures) ──

    #[test]
    fn test_cleanup_all_closures_clears_closures() {
        let name = MbValue::from_ptr(MbObject::new_str("cleanup_cl".into()));
        let func = MbValue::from_int(1);
        let caps = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(10)]));
        let handle = mb_closure_new(name, func, caps);
        assert_eq!(mb_closure_get_func(handle).as_int(), Some(1));

        cleanup_all_closures();

        assert!(
            mb_closure_get_func(handle).is_none(),
            "closures should be empty after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_closures_clears_cells() {
        let cell = mb_cell_new(MbValue::from_int(42));
        assert_eq!(mb_cell_get(cell).as_int(), Some(42));

        cleanup_all_closures();

        assert!(
            mb_cell_get(cell).is_none(),
            "cells should be empty after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_closures_clears_globals() {
        let name = MbValue::from_ptr(MbObject::new_str("cleanup_var".into()));
        mb_global_set(name, MbValue::from_int(77));

        cleanup_all_closures();

        let name2 = MbValue::from_ptr(MbObject::new_str("cleanup_var".into()));
        assert!(
            mb_global_get(name2).is_none(),
            "global namespace should be empty after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_closures_clears_global_id_namespace() {
        let id = MbValue::from_bits(12345);
        mb_global_set_id(id, MbValue::from_int(88));
        assert_eq!(mb_global_get_id(id).as_int(), Some(88));

        cleanup_all_closures();

        assert!(
            mb_global_get_id(id).is_none(),
            "global ID namespace should be empty after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_closures_resets_id_counters() {
        // Create some closures to advance the ID counter
        let name = MbValue::from_ptr(MbObject::new_str("c1".into()));
        let func = MbValue::from_int(1);
        let caps = MbValue::from_ptr(MbObject::new_list(vec![]));
        let h1 = mb_closure_new(name, func, caps);

        cleanup_all_closures();

        // After cleanup, the next closure should get ID 1 again
        let name2 = MbValue::from_ptr(MbObject::new_str("c2".into()));
        let func2 = MbValue::from_int(2);
        let caps2 = MbValue::from_ptr(MbObject::new_list(vec![]));
        let h2 = mb_closure_new(name2, func2, caps2);
        // Both should have the same ID (1) since counter was reset
        assert_eq!(
            h1.as_int(),
            h2.as_int(),
            "closure ID counter should reset to 1 after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_closures_on_empty_state() {
        // Should not panic when there's nothing to clean
        cleanup_all_closures();
    }

    // ── Refcount symmetry regression tests (R3) ──

    /// REQ: R3(a) — set_defaults overwrite releases prior ptr values.
    /// Uses a heap-allocated string as a default so release_if_ptr will act.
    /// After overwrite the new defaults are visible and the function still works.
    #[test]
    fn test_set_defaults_overwrite_releases_prior() {
        let name = MbValue::from_ptr(MbObject::new_str("defclose".into()));
        let func = MbValue::from_int(7);
        let caps = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, caps);

        // First set: one string default (heap-allocated)
        let str_val = MbValue::from_ptr(MbObject::new_str("first_default".into()));
        let list1 = MbValue::from_ptr(MbObject::new_list(vec![str_val]));
        mb_closure_set_defaults(closure, list1);
        {
            let got = closure_defaults(closure);
            assert_eq!(got.len(), 1);
        }

        // Second set: two integer defaults — replaces the prior string default.
        // The old string's refcount should be decremented (released) without crash.
        let list2 = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(100),
            MbValue::from_int(200),
        ]));
        mb_closure_set_defaults(closure, list2);
        let got2 = closure_defaults(closure);
        assert_eq!(got2.len(), 2, "new defaults should replace prior ones");
        assert_eq!(got2[0].as_int(), Some(100));
        assert_eq!(got2[1].as_int(), Some(200));

        // Func still readable
        assert_eq!(mb_closure_get_func(closure).as_int(), Some(7));
        mb_closure_release(closure);
    }

    /// REQ: R3(b) — set_capture overwrite releases prior heap-allocated value.
    /// Verifies that overwriting a ptr-valued capture slot does not leak.
    #[test]
    fn test_set_capture_overwrite_releases_prior() {
        let name = MbValue::from_ptr(MbObject::new_str("capclose".into()));
        let func = MbValue::from_int(3);
        // Initial capture: one string value
        let str_cap = MbValue::from_ptr(MbObject::new_str("initial_cap".into()));
        let caps = MbValue::from_ptr(MbObject::new_list(vec![str_cap]));
        let closure = mb_closure_new(name, func, caps);

        // Read initial capture
        let v0 = mb_closure_get_capture(closure, MbValue::from_int(0));
        assert!(v0.is_ptr(), "initial capture is a heap ptr");

        // Overwrite slot 0 with an integer — release of the old string happens here
        mb_closure_set_capture(closure, MbValue::from_int(0), MbValue::from_int(42));
        assert_eq!(
            mb_closure_get_capture(closure, MbValue::from_int(0)).as_int(),
            Some(42),
            "capture should reflect new value after overwrite",
        );

        mb_closure_release(closure);
    }

    /// REQ: R3(c) — get_func retains: after releasing the closure the returned
    /// func value is still valid (its rc was bumped by get_func).
    /// This test uses integer func values (non-ptr) — the retain is a no-op for
    /// non-ptr values, so we verify correctness of the return value and that
    /// mb_closure_release after get_func does not panic or corrupt anything.
    #[test]
    fn test_get_func_retain_survives_closure_release() {
        let name = MbValue::from_ptr(MbObject::new_str("retain_test".into()));
        let func = MbValue::from_int(999);
        let caps = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, caps);

        // get_func now retains the returned value
        let returned_func = mb_closure_get_func(closure);
        assert_eq!(returned_func.as_int(), Some(999));

        // Release the closure — for integer func, no rc change; verifies no panic
        mb_closure_release(closure);

        // returned_func is still valid (it was retained)
        assert_eq!(returned_func.as_int(), Some(999));
    }

    /// REQ: R3(c) bonus — get_func with a heap-allocated func ptr is retained.
    /// After releasing the closure, the returned ptr value should have been
    /// retain'd so it is still valid.
    #[test]
    fn test_get_func_retain_ptr_func() {
        let name = MbValue::from_ptr(MbObject::new_str("ptr_func_test".into()));
        // Use a heap string as a stand-in for a func ptr so rc tracking is visible
        let ptr_func = MbValue::from_ptr(MbObject::new_str("my_func_ptr".into()));
        let caps = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, ptr_func, caps);

        // get_func should retain ptr_func
        let returned_func = mb_closure_get_func(closure);
        assert!(returned_func.is_ptr());

        // Release the closure (removes it from CLOSURES vec; does NOT cascade-release func
        // since mb_closure_release is out of scope for this change)
        mb_closure_release(closure);

        // returned_func was retained by get_func — still points to valid object
        assert!(returned_func.is_ptr());
        // Clean up the extra retain from get_func
        unsafe {
            crate::runtime::rc::release_if_ptr(returned_func);
        }
    }

    #[test]
    fn test_closure_set_defaults_round_trip() {
        let name = MbValue::from_ptr(MbObject::new_str("defs".into()));
        let func = MbValue::from_int(7);
        let captures = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, captures);
        let defaults = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
        ]));
        mb_closure_set_defaults(closure, defaults);
        let got = closure_defaults(closure);
        assert_eq!(got.len(), 2, "defaults vec must have 2 entries");
        assert_eq!(got[0].as_int(), Some(10));
        assert_eq!(got[1].as_int(), Some(20));
        mb_closure_release(closure);
    }
}
