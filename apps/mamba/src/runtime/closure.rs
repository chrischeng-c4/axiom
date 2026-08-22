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
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const CLOSURE_HANDLE_BASE: i64 = 1i64 << 39;
const CELL_ID_BASE: i64 = 1i64 << 38;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ScopedSymbolKey {
    pub module: Arc<str>,
    pub symbol: i64,
}

/// A closure object — a function paired with its captured environment.
pub struct MbClosure {
    /// Lightweight handle refcount for runtime-owned closure values.
    pub refs: u32,
    /// Name of the function
    pub name: String,
    /// Qualified name of the function, when known.
    pub qualname: Option<String>,
    /// Docstring metadata copied onto this closure handle, when present.
    pub doc: Option<String>,
    /// Defining module metadata copied onto this closure handle, when present.
    pub module: Option<String>,
    /// `__wrapped__` back-reference for functools.wraps/update_wrapper.
    pub wrapped: Option<MbValue>,
    /// Captured variables: name → MbValue
    pub captures: Vec<MbValue>,
    /// Captured variable SymbolIds. These line up with `capture_cells`.
    pub capture_ids: Vec<i64>,
    /// Cell handles backing captured variables. Multiple closures created in
    /// the same factory call can share these handles.
    pub capture_cells: Vec<MbValue>,
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
    static ACTIVE_CELLS: std::cell::RefCell<HashMap<ScopedSymbolKey, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
    static ACTIVE_MODULE_NAMES: std::cell::RefCell<Vec<String>> =
        std::cell::RefCell::new(Vec::new());
    static ACTIVE_QUALNAME_CONTEXTS: std::cell::RefCell<Vec<QualnameContext>> =
        std::cell::RefCell::new(Vec::new());
}

#[derive(Clone)]
struct QualnameContext {
    prefix: String,
    uses_locals: bool,
}

fn derive_qualname(name: &str) -> String {
    ACTIVE_QUALNAME_CONTEXTS.with(|contexts| {
        let contexts = contexts.borrow();
        let Some(ctx) = contexts.last() else {
            return name.to_string();
        };
        if ctx.prefix.is_empty() {
            return name.to_string();
        }
        if ctx.uses_locals {
            format!("{}.<locals>.{name}", ctx.prefix)
        } else {
            format!("{}.{}", ctx.prefix, name)
        }
    })
}

pub(crate) fn current_definition_qualname(name: &str) -> String {
    derive_qualname(name)
}

fn push_qualname_context(prefix: String, uses_locals: bool) {
    if prefix.is_empty() {
        return;
    }
    ACTIVE_QUALNAME_CONTEXTS.with(|contexts| {
        contexts.borrow_mut().push(QualnameContext {
            prefix,
            uses_locals,
        });
    });
}

pub fn mb_pop_qualname_context() {
    ACTIVE_QUALNAME_CONTEXTS.with(|contexts| {
        contexts.borrow_mut().pop();
    });
}

pub fn mb_push_class_qualname(name: MbValue) {
    let class_name = extract_str(name).unwrap_or_default();
    if class_name.is_empty() {
        return;
    }
    push_qualname_context(derive_qualname(&class_name), false);
}

fn closure_slot_index(raw: i64) -> Option<usize> {
    if raw < CLOSURE_HANDLE_BASE {
        return None;
    }
    Some((raw - CLOSURE_HANDLE_BASE) as usize)
}

fn with_live_closure<R>(closure_handle: MbValue, read: impl FnOnce(&MbClosure) -> R) -> Option<R> {
    let id = closure_handle.as_int()?;
    let idx = closure_slot_index(id)?;
    let state = super::program_state::current_program_state();
    let vec = state.closures.read().ok()?;
    vec.get(idx).and_then(|slot| slot.as_ref()).map(read)
}

fn with_live_closure_mut<R>(
    closure_handle: MbValue,
    write: impl FnOnce(&mut MbClosure) -> R,
) -> Option<R> {
    let id = closure_handle.as_int()?;
    let idx = closure_slot_index(id)?;
    let state = super::program_state::current_program_state();
    let mut vec = state.closures.write().ok()?;
    vec.get_mut(idx).and_then(|slot| slot.as_mut()).map(write)
}

fn allocate_closure_slot(closure: MbClosure) -> MbValue {
    let state = super::program_state::current_program_state();
    let mut vec = state.closures.write().unwrap();
    if let Some((idx, slot)) = vec.iter_mut().enumerate().find(|(_, slot)| slot.is_none()) {
        *slot = Some(closure);
        return MbValue::from_int(CLOSURE_HANDLE_BASE + idx as i64);
    }
    let id = CLOSURE_HANDLE_BASE + vec.len() as i64;
    vec.push(Some(closure));
    MbValue::from_int(id)
}

fn teardown_closure(closure: MbClosure) {
    if let Some(wrapped) = closure.wrapped {
        unsafe {
            super::rc::release_if_ptr(wrapped);
        }
    }
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

fn force_take_live_closure(closure_handle: MbValue) -> Option<MbClosure> {
    let id = closure_handle.as_int()?;
    let idx = closure_slot_index(id)?;
    let state = super::program_state::current_program_state();
    let mut vec = state.closures.write().ok()?;
    if idx < vec.len() {
        vec[idx].take()
    } else {
        None
    }
}

fn dec_ref_or_take_live_closure(closure_handle: MbValue) -> Option<Option<MbClosure>> {
    let id = closure_handle.as_int()?;
    let idx = closure_slot_index(id)?;
    let state = super::program_state::current_program_state();
    let mut vec = state.closures.write().ok()?;
    let slot = vec.get_mut(idx)?;
    let closure = slot.as_mut()?;
    if closure.refs > 1 {
        closure.refs -= 1;
        Some(None)
    } else {
        Some(slot.take())
    }
}

static MAIN_MODULE_NAME: std::sync::OnceLock<Arc<str>> = std::sync::OnceLock::new();

fn main_module_name() -> Arc<str> {
    MAIN_MODULE_NAME
        .get_or_init(|| Arc::from("__main__"))
        .clone()
}

pub(crate) fn current_active_module_name_arc() -> Arc<str> {
    ACTIVE_MODULE_NAMES.with(|names| {
        let b = names.borrow();
        if let Some(s) = b.last() {
            Arc::from(s.as_str())
        } else {
            main_module_name()
        }
    })
}

pub(crate) fn current_active_module_name() -> String {
    current_active_module_name_arc().to_string()
}

pub(crate) fn active_module_matches(name: &str) -> bool {
    ACTIVE_MODULE_NAMES.with(|names| names.borrow().last().is_some_and(|current| current == name))
        || (name == "__main__" && ACTIVE_MODULE_NAMES.with(|names| names.borrow().last().is_none()))
}

pub(crate) fn caller_active_module_name() -> String {
    ACTIVE_MODULE_NAMES.with(|names| {
        let names = names.borrow();
        match names.len() {
            0 | 1 => "__main__".to_string(),
            len => names[len - 2].clone(),
        }
    })
}

fn scoped_symbol_key(symbol: i64) -> ScopedSymbolKey {
    ScopedSymbolKey {
        module: current_active_module_name_arc(),
        symbol,
    }
}

pub fn push_active_module_name(name: String) {
    ACTIVE_MODULE_NAMES.with(|names| names.borrow_mut().push(name));
}

pub fn pop_active_module_name() {
    ACTIVE_MODULE_NAMES.with(|names| {
        names.borrow_mut().pop();
    });
}

fn callable_module_name(func: MbValue) -> Option<String> {
    let target = if func.as_int().is_some() {
        let inner = mb_closure_get_func(func);
        if !inner.is_none() {
            inner
        } else {
            func
        }
    } else {
        func
    };
    let module = extract_str(mb_func_get_module(target)).filter(|name| !name.is_empty());
    if module.is_some() {
        return module;
    }
    if !mb_func_get_name(target).is_none() {
        return Some(current_active_module_name());
    }
    None
}

pub fn with_callable_module<R>(func: MbValue, call: impl FnOnce() -> R) -> R {
    let Some(module_name) = callable_module_name(func) else {
        return call();
    };
    struct CallContextGuard {
        pop_qualname: bool,
    }
    impl Drop for CallContextGuard {
        fn drop(&mut self) {
            if self.pop_qualname {
                mb_pop_qualname_context();
            }
            pop_active_module_name();
        }
    }
    let qualname =
        extract_str(mb_func_get_qualname(func)).or_else(|| extract_str(mb_func_get_name(func)));
    let pop_qualname = qualname.is_some();
    push_active_module_name(module_name);
    if let Some(qualname) = qualname {
        push_qualname_context(qualname, true);
    }
    let _guard = CallContextGuard { pop_qualname };
    call()
}

// ── Closure Creation ──

/// Create a new closure capturing the given variables.
pub fn mb_closure_new(name: MbValue, func: MbValue, captures: MbValue) -> MbValue {
    let closure_name = extract_str(name).unwrap_or_else(|| "<closure>".to_string());
    let captured_vars = extract_list(captures);

    let closure = MbClosure {
        refs: 1,
        name: closure_name,
        qualname: None,
        doc: None,
        module: Some(current_active_module_name()),
        wrapped: None,
        captures: captured_vars,
        capture_ids: Vec::new(),
        capture_cells: Vec::new(),
        func,
        defaults: Vec::new(),
        arity: 0,
    };
    allocate_closure_slot(closure)
}

/// Create a closure whose captures are backed by active cell handles.
///
/// `capture_ids` is a list of boxed SymbolId integers. Each id resolves to the
/// active cell for the current factory call, creating one from the current
/// global value if needed.
pub fn mb_closure_new_with_cells(name: MbValue, func: MbValue, capture_ids: MbValue) -> MbValue {
    let closure_name = extract_str(name).unwrap_or_else(|| "<closure>".to_string());
    let ids: Vec<i64> = extract_list(capture_ids)
        .into_iter()
        .filter_map(|v| v.as_int())
        .collect();
    let cells: Vec<MbValue> = ids.iter().map(|&id| active_cell_for_id(id)).collect();
    let captures: Vec<MbValue> = cells.iter().map(|&cell| mb_cell_get(cell)).collect();

    let closure = MbClosure {
        refs: 1,
        name: closure_name,
        qualname: None,
        doc: None,
        module: Some(current_active_module_name()),
        wrapped: None,
        captures,
        capture_ids: ids,
        capture_cells: cells,
        func,
        defaults: Vec::new(),
        arity: 0,
    };
    allocate_closure_slot(closure)
}

/// Set default argument values on an existing closure.
/// Called at lambda creation time after `mb_closure_new` to freeze default-arg
/// expressions into the closure. Takes a list MbValue whose elements are the
/// evaluated default values, in parameter order (defaults fill trailing params).
pub fn mb_closure_set_defaults(closure_handle: MbValue, defaults_list: MbValue) {
    set_closure_defaults_values(closure_handle, extract_list(defaults_list));
}

fn set_closure_defaults_values(closure_handle: MbValue, vals: Vec<MbValue>) {
    let old_defaults = with_live_closure_mut(closure_handle, |c| {
        let old = std::mem::take(&mut c.defaults);
        c.defaults = vals;
        old
    }).unwrap_or_default();
    for old_val in old_defaults {
        unsafe {
            super::rc::release_if_ptr(old_val);
        }
    }
    if let Some(defaults) = closure_defaults(closure_handle).get(..) {
        for &new_val in defaults {
            unsafe {
                super::rc::retain_if_ptr(new_val);
            }
        }
    }
}

/// Set the total parameter count on a closure. Codegen emits this whenever
/// a function/lambda has at least one default value, so the dispatcher can
/// fill missing trailing args from `defaults`.
pub fn mb_closure_set_arity(closure_handle: MbValue, arity: MbValue) {
    if let Some(n) = arity.as_int() {
        with_live_closure_mut(closure_handle, |c| {
            c.arity = n.max(0) as usize;
        });
    }
}

/// Get the recorded arity for a closure (0 = unset).
pub fn closure_arity(closure_handle: MbValue) -> usize {
    with_live_closure(closure_handle, |c| c.arity).unwrap_or(0)
}

/// Get a clone of the default argument values for a closure. Returns an
/// empty Vec if the closure has no defaults or the handle is invalid.
pub fn closure_defaults(closure_handle: MbValue) -> Vec<MbValue> {
    with_live_closure(closure_handle, |c| c.defaults.clone()).unwrap_or_default()
}

/// Get a captured variable by index.
pub fn mb_closure_get_capture(closure_handle: MbValue, index: MbValue) -> MbValue {
    if let Some(idx) = index.as_int() {
        let val = with_live_closure(closure_handle, |c| {
            if let Some(cell) = c.capture_cells.get(idx as usize).copied() {
                return mb_cell_get(cell);
            }
            c.captures.get(idx as usize).copied().unwrap_or(MbValue::none())
        }).unwrap_or_else(MbValue::none);
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        val
    } else {
        MbValue::none()
    }
}

/// Set a captured variable by index (for mutable closures).
pub fn mb_closure_set_capture(closure_handle: MbValue, index: MbValue, value: MbValue) {
    if let Some(idx) = index.as_int() {
        let old_capture = with_live_closure_mut(closure_handle, |c| {
            let idx = idx as usize;
            if let Some(cell) = c.capture_cells.get(idx).copied() {
                mb_cell_set(cell, value);
                return None;
            }
            if idx >= c.captures.len() {
                c.captures.resize(idx + 1, MbValue::none());
            }
            let old = c.captures[idx];
            c.captures[idx] = value;
            Some(old)
        }).flatten();
        if let Some(old_capture) = old_capture {
            unsafe {
                super::rc::release_if_ptr(old_capture);
            }
        }
    }
}

/// Cell handles captured by a closure handle.
pub fn closure_capture_cells(closure_handle: MbValue) -> Vec<MbValue> {
    with_live_closure(closure_handle, |c| c.capture_cells.clone()).unwrap_or_default()
}

/// Read a captured variable's live value directly from a closure's own
/// `capture_ids`/`capture_cells`, keyed by SymbolId. Returns `None` if the
/// handle isn't a closure, the id isn't one of its captures, or the cell is
/// unset.
///
/// Introspection (`inspect.getclosurevars`) must use this instead of the
/// module-scoped active-cell lookup (`mb_global_get_id_raw`): that lookup is
/// keyed to whichever module is currently "active" per `with_callable_module`,
/// which during the introspection call itself is the *introspecting*
/// dispatcher's own module (e.g. "inspect", since native dispatchers are
/// tagged with `__module__` like any other registered function) rather than
/// the module the inspected closure was defined in — so it misses the real
/// cell entirely and reads back as unset.
pub fn closure_capture_value_for_id(closure_handle: MbValue, id: i64) -> Option<MbValue> {
    let cell = with_live_closure(closure_handle, |c| {
        let pos = c.capture_ids.iter().position(|&cid| cid == id)?;
        c.capture_cells.get(pos).copied()
    })??;
    match mb_cell_contents_read(cell) {
        CellContentsRead::Value(v) => Some(v),
        CellContentsRead::Empty | CellContentsRead::NotACell => None,
    }
}

/// Run a closure body with its captured cells installed as the active cell map.
pub fn with_closure_cells<R>(closure_handle: MbValue, call: impl FnOnce() -> R) -> R {
    let (raw_pairs, closure_mod) = with_live_closure(closure_handle, |c| {
        let raw: Vec<(i64, MbValue)> = c.capture_ids
            .iter()
            .copied()
            .zip(c.capture_cells.iter().copied())
            .collect();
        (raw, c.module.clone())
    }).unwrap_or_default();

    struct ModuleGuard {
        pushed: bool,
    }
    impl Drop for ModuleGuard {
        fn drop(&mut self) {
            if self.pushed {
                pop_active_module_name();
            }
        }
    }
    let mut pushed = false;
    if let Some(mod_name) = closure_mod {
        push_active_module_name(mod_name);
        pushed = true;
    }
    let _mod_guard = ModuleGuard { pushed };

    if raw_pairs.is_empty() {
        return call();
    }

    let pairs: Vec<(ScopedSymbolKey, MbValue)> = raw_pairs
        .into_iter()
        .map(|(id, cell)| (scoped_symbol_key(id), cell))
        .collect();

    let saved = ACTIVE_CELLS.with(|cells| {
        let mut cells = cells.borrow_mut();
        pairs
            .iter()
            .map(|(id, cell)| (id.clone(), cells.insert(id.clone(), *cell)))
            .collect::<Vec<_>>()
    });
    let result = call();
    ACTIVE_CELLS.with(|cells| {
        let mut cells = cells.borrow_mut();
        for (id, old) in saved {
            if let Some(old_cell) = old {
                cells.insert(id, old_cell);
            } else {
                cells.remove(&id);
            }
        }
    });
    result
}

/// Opaque snapshot of the active cells captured while constructing a deferred
/// coroutine or generator. Cell handles remain owned by the closure runtime;
/// this context only preserves the scoped symbol-to-cell association.
#[derive(Clone)]
pub(crate) struct CapturedCellContext {
    pairs: Arc<Vec<(ScopedSymbolKey, MbValue)>>,
}

impl Default for CapturedCellContext {
    fn default() -> Self {
        Self {
            pairs: Arc::new(Vec::new()),
        }
    }
}

pub(crate) fn capture_active_cell_context(ids: &[i64]) -> CapturedCellContext {
    let pairs = ids
        .iter()
        .copied()
        .map(|id| (scoped_symbol_key(id), active_cell_for_id(id)))
        .collect();
    CapturedCellContext {
        pairs: Arc::new(pairs),
    }
}



pub(crate) fn with_captured_cell_context<R>(
    context: &CapturedCellContext,
    call: impl FnOnce() -> R,
) -> R {
    if context.pairs.is_empty() {
        return call();
    }
    let saved = ACTIVE_CELLS.with(|cells| {
        let mut cells = cells.borrow_mut();
        context
            .pairs
            .iter()
            .map(|(id, cell)| (id.clone(), cells.insert(id.clone(), *cell)))
            .collect::<Vec<_>>()
    });
    let result = call();
    ACTIVE_CELLS.with(|cells| {
        let mut cells = cells.borrow_mut();
        for (id, old) in saved {
            if let Some(old_cell) = old {
                cells.insert(id, old_cell);
            } else {
                cells.remove(&id);
            }
        }
    });
    result
}

/// Get the underlying function of a closure.
pub fn mb_closure_get_func(closure_handle: MbValue) -> MbValue {
    let val = with_live_closure(closure_handle, |c| c.func).unwrap_or_else(MbValue::none);
    unsafe {
        super::rc::retain_if_ptr(val);
    }
    val
}

/// Release a closure's resources, cascading rc releases on captures and
/// defaults so heap values referenced only via the closure get freed.
pub fn mb_closure_release(closure_handle: MbValue) {
    if let Some(closure) = force_take_live_closure(closure_handle) {
        teardown_closure(closure);
    }
}

pub(crate) fn retain_closure_handle_if_live(value: MbValue) -> bool {
    if with_live_closure(value, |_| ()).is_none() {
        return false;
    }
    with_live_closure_mut(value, |closure| {
        closure.refs = closure.refs.saturating_add(1);
    })
    .is_some()
}

pub(crate) fn release_closure_handle_if_live(value: MbValue) -> bool {
    match dec_ref_or_take_live_closure(value) {
        None => false,
        Some(None) => true,
        Some(Some(closure)) => {
            teardown_closure(closure);
            true
        }
    }
}

fn release_rebound_global_value(prev: MbValue) {
    unsafe {
        super::rc::release_if_ptr(prev);
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
                let decs = lock.read().unwrap().to_vec();
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

// Function metadata registries are stored in ProgramState (process-shared).

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
    /// Lowered entry representation. Older metadata is conservatively boxed.
    pub entry_abi: String,
    /// Resolved scalar contract; unlike annotation, this is authoritative for
    /// runtime rejection and is absent for Any/generic/container/forward refs.
    pub contract: Option<String>,
}

/// Register a function's declared parameters. `params` is a list of
/// seven-field (name, kind, has_default, default, annotation, entry_abi,
/// contract) tuples. Five/six-field tuples remain accepted for old modules.
/// lower_top_level priming loop in hir_to_mir.rs.
pub fn mb_func_set_params(func: MbValue, params: MbValue) {
    let mut infos: Vec<MbParamInfo> = Vec::new();
    if let Some(ptr) = params.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                for item in lock.read().unwrap().to_vec() {
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
                    let entry_abi = elems
                        .get(5)
                        .and_then(|value| extract_str(*value))
                        .unwrap_or_else(|| "boxed".to_string());
                    let contract = elems.get(6).and_then(|value| extract_str(*value));
                    infos.push(MbParamInfo {
                        name,
                        kind,
                        has_default,
                        default,
                        annotation,
                        entry_abi,
                        contract,
                    });
                }
            }
        }
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_params.write() {
        if let Some(prev) = m.insert(key, infos) {
            for p in prev {
                unsafe {
                    super::rc::release_if_ptr(p.default);
                }
            }
        }
    };
}

fn extract_defaults_assignment(value: MbValue) -> Option<Vec<MbValue>> {
    if value.is_none() {
        return Some(Vec::new());
    }
    let ptr = value.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => Some(items.clone()),
            _ => None,
        }
    }
}

/// Rewrite a function's live positional defaults from `f.__defaults__`.
/// Defaults are right-aligned to positional-only + positional-or-keyword
/// parameters, matching CPython's call-time rule.
pub fn mb_func_set_pos_defaults(func: MbValue, defaults_value: MbValue) -> bool {
    let Some(defaults) = extract_defaults_assignment(defaults_value) else {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "__defaults__ must be set to a tuple object".to_string(),
            )),
        );
        return true;
    };

    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let updated = if let Ok(mut map) = state.func_params.write() {
        if let Some(params) = map.get_mut(&key) {
            let pos_indices: Vec<usize> = params
                .iter()
                .enumerate()
                .filter(|(_, p)| p.kind <= 1)
                .map(|(i, _)| i)
                .collect();
            let first_default_ordinal = pos_indices.len().saturating_sub(defaults.len());
            let default_offset = defaults.len().saturating_sub(pos_indices.len());
            for (ordinal, param_idx) in pos_indices.into_iter().enumerate() {
                let p = &mut params[param_idx];
                let next_default = if ordinal >= first_default_ordinal {
                    Some(defaults[default_offset + ordinal - first_default_ordinal])
                } else {
                    None
                };
                if let Some(v) = next_default {
                    unsafe {
                        super::rc::retain_if_ptr(v);
                    }
                    let old = p.default;
                    p.default = v;
                    p.has_default = true;
                    unsafe {
                        super::rc::release_if_ptr(old);
                    }
                } else {
                    let old = p.default;
                    p.default = MbValue::none();
                    p.has_default = false;
                    unsafe {
                        super::rc::release_if_ptr(old);
                    }
                }
            }
            true
        } else {
            false
        }
    } else {
        false
    };
    if updated {
        set_closure_defaults_values(func, defaults);
    }
    updated
}

/// Look up the call-time default value for a positional parameter of `func`
/// at Python-style negative offset `offset` (`-1` = last positional default,
/// `-2` = second-to-last, ...), reading the function's LIVE declared-parameter
/// defaults — the same store `f.__defaults__ = ...` / `del f.__defaults__`
/// mutate via `mb_func_set_pos_defaults`. `build_mutated_defaults_call`
/// (ast_to_hir.rs) emits a call to this instead of indexing `f.__defaults__`
/// directly, so a still-missing default (after `del f.__defaults__`, or a
/// replacement tuple too short to cover every unsupplied argument) raises the
/// same "missing required positional argument" TypeError CPython raises at
/// call time (#897 R2), instead of silently reading `None`/indexing out of
/// bounds.
pub fn mb_func_default_at(func: MbValue, offset: MbValue) -> MbValue {
    let off = offset.as_int().unwrap_or(0);
    let params = func_params(func).unwrap_or_default();
    let defaults: Vec<MbValue> = params
        .iter()
        .filter(|p| p.kind <= 1 && p.has_default)
        .map(|p| p.default)
        .collect();
    let n = defaults.len() as i64;
    let idx = n + off;
    if idx >= 0 && idx < n {
        return defaults[idx as usize];
    }
    let fname = extract_str(mb_func_get_name(func)).unwrap_or_else(|| "<function>".to_string());
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "{fname}() missing required positional argument"
        ))),
    );
    MbValue::none()
}

/// Register a function's textual return annotation.
pub fn mb_func_set_retanno(func: MbValue, anno: MbValue) {
    if let Some(s) = extract_str(anno) {
        let key = func.to_bits();
        let state = super::program_state::current_program_state();
        if let Ok(mut m) = state.func_ret_annos.write() {
            m.insert(key, s);
        };
    }
}

/// Declared parameters for a registered function, or None when unknown.
pub fn func_params(func: MbValue) -> Option<Vec<MbParamInfo>> {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_params.read().ok()?;
    guard.get(&key).cloned()
}

pub fn mb_func_set_boxed_params(func: MbValue, flag: MbValue) {
    let enabled = flag.as_bool().unwrap_or(false) || flag.as_int().unwrap_or(0) != 0;
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut set) = state.func_boxed_params.write() {
        if enabled {
            set.insert(key);
        } else {
            set.remove(&key);
        }
    };
}

pub fn func_has_boxed_params(func: MbValue) -> bool {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_boxed_params.read().ok();
    guard.map(|m| m.contains(&key)).unwrap_or(false)
}

/// Textual return annotation for a registered function, or None.
pub fn func_ret_anno(func: MbValue) -> Option<String> {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_ret_annos.read().ok()?;
    guard.get(&key).cloned()
}

fn is_simple_annotation_name(annotation: &str) -> bool {
    let mut chars = annotation.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn lookup_annotation_type_param(params: MbValue, annotation: &str) -> Option<MbValue> {
    let params_ptr = params.as_ptr()?;
    unsafe {
        let ObjData::Tuple(items) = &(*params_ptr).data else {
            return None;
        };
        for item in items {
            let name_attr = MbValue::from_ptr(MbObject::new_str("__name__".to_string()));
            let name = extract_str(super::class::mb_getattr(*item, name_attr));
            if name.as_deref() == Some(annotation) {
                return Some(*item);
            }
        }
    }
    None
}

fn resolve_function_annotation_value(func: MbValue, annotation: String) -> MbValue {
    if is_simple_annotation_name(&annotation) {
        if let Some(type_params) = super::pep695::func_attrs_get(func, "__type_params__") {
            if let Some(param) = lookup_annotation_type_param(type_params, &annotation) {
                return param;
            }
        }
        if let Some(class_type_params) =
            super::pep695::func_attrs_get(func, "__mb_class_type_params__")
        {
            if let Some(param) = lookup_annotation_type_param(class_type_params, &annotation) {
                return param;
            }
        }
    }

    MbValue::from_ptr(MbObject::new_str(annotation))
}

/// Build a function's `__annotations__` dict from its registered parameter and
/// return annotations (PEP 3107 / 526). Values are the textual annotations,
/// matching mamba's module- and class-level `__annotations__`. Returns
/// None-MbValue when the function is unregistered, an (possibly empty) dict
/// otherwise — CPython exposes `__annotations__` on every function.
pub fn mb_func_get_annotations(func: MbValue) -> MbValue {
    if let Some(annotations) = super::pep695::func_attrs_get(func, "__annotations__") {
        return annotations;
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let known = state.func_params.read().map(|m| m.contains_key(&key)).unwrap_or(false)
        || state.func_ret_annos.read().map(|m| m.contains_key(&key)).unwrap_or(false);
    if !known {
        return MbValue::none();
    }
    let dict = MbValue::from_ptr(MbObject::new_dict());
    if let Some(params) = func_params(func) {
        for p in params {
            if let Some(anno) = p.annotation {
                let k = MbValue::from_ptr(MbObject::new_str(p.name.clone()));
                let v = resolve_function_annotation_value(func, anno);
                super::dict_ops::mb_dict_setitem(dict, k, v);
            }
        }
    }
    if let Some(ret) = func_ret_anno(func) {
        let k = MbValue::from_ptr(MbObject::new_str("return".to_string()));
        let v = resolve_function_annotation_value(func, ret);
        super::dict_ops::mb_dict_setitem(dict, k, v);
    }
    dict
}

/// Register a function's name (called at definition time so `f.__name__` works).
pub fn mb_func_set_name(func: MbValue, name: MbValue) {
    let fname = extract_str(name).unwrap_or_default();
    if with_live_closure_mut(func, |closure| closure.name = fname.clone()).is_some() {
        return;
    }
    if func.as_func().is_none() {
        return;
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_names.write() {
        m.insert(key, fname);
    };
}

/// Register a function's qualified name.
pub fn mb_func_set_qualname(func: MbValue, qualname: MbValue) {
    let qualname = extract_str(qualname).unwrap_or_default();
    if with_live_closure_mut(func, |closure| closure.qualname = Some(qualname.clone())).is_some() {
        return;
    }
    if func.as_func().is_none() {
        return;
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_qualnames.write() {
        m.insert(key, qualname);
    };
}

/// Register a function's simple name, its lexical qualname, and its defining
/// module — all visible at its definition site.
///
/// The module must be captured HERE, at define time, rather than derived
/// later at call time: `callable_module_name` (used by `with_callable_module`
/// to scope global-variable reads/writes for a dynamically-invoked callable)
/// falls back to "whatever module is currently active" when a function has
/// no registered `__module__`. That fallback is only correct when a function
/// is invoked from within its own defining module's activation; it silently
/// resolves to the WRONG module when a user-defined function/closure is
/// invoked as a callback from a different native module's own dispatch code
/// (e.g. a handler passed to `signal.signal` and delivered through
/// `os.kill`, or a reducer passed to `functools.reduce`), corrupting every
/// global read/write the callback performs (#239).
pub fn mb_func_prime_name(func: MbValue, name: MbValue) {
    let simple_name = extract_str(name).unwrap_or_default();
    mb_func_set_name(
        func,
        MbValue::from_ptr(MbObject::new_str(simple_name.clone())),
    );
    let qualname = derive_qualname(&simple_name);
    mb_func_set_qualname(func, MbValue::from_ptr(MbObject::new_str(qualname)));
    mb_func_set_module(
        func,
        MbValue::from_ptr(MbObject::new_str(current_active_module_name())),
    );
}

/// Get a function's registered name. Returns None-MbValue if not registered.
pub fn mb_func_get_name(func: MbValue) -> MbValue {
    if let Some(name) = with_live_closure(func, |closure| closure.name.clone()) {
        return MbValue::from_ptr(MbObject::new_str(name));
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_names.read().ok();
    guard
        .and_then(|m| m.get(&key).map(|s| MbValue::from_ptr(MbObject::new_str(s.clone()))))
        .unwrap_or_else(MbValue::none)
}

/// Get a function's registered qualified name. Returns None-MbValue if absent.
pub fn mb_func_get_qualname(func: MbValue) -> MbValue {
    if let Some(qualname) = with_live_closure(func, |closure| closure.qualname.clone()) {
        return qualname
            .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
            .unwrap_or_else(MbValue::none);
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_qualnames.read().ok();
    guard
        .and_then(|m| m.get(&key).map(|s| MbValue::from_ptr(MbObject::new_str(s.clone()))))
        .unwrap_or_else(MbValue::none)
}

/// Register a function's docstring (for `f.__doc__`). Called at module init
/// for every top-level def whose body starts with a bare string literal.
pub fn mb_func_set_doc(func: MbValue, doc: MbValue) {
    let fdoc = extract_str(doc).unwrap_or_default();
    if with_live_closure_mut(func, |closure| closure.doc = Some(fdoc.clone())).is_some() {
        return;
    }
    if func.as_func().is_none() {
        return;
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_docs.write() {
        m.insert(key, fdoc);
    };
}

/// Get a function's registered docstring. Returns None-MbValue if not registered.
pub fn mb_func_get_doc(func: MbValue) -> MbValue {
    if let Some(doc) = with_live_closure(func, |closure| closure.doc.clone()) {
        return doc
            .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
            .unwrap_or_else(MbValue::none);
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_docs.read().ok();
    guard
        .and_then(|m| m.get(&key).map(|s| MbValue::from_ptr(MbObject::new_str(s.clone()))))
        .unwrap_or_else(MbValue::none)
}

/// Register a function's module name (for `f.__module__`).
pub fn mb_func_set_module(func: MbValue, module: MbValue) {
    let module_name = extract_str(module).unwrap_or_default();
    if with_live_closure_mut(func, |closure| closure.module = Some(module_name.clone())).is_some() {
        return;
    }
    if func.as_func().is_none() {
        return;
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_modules.write() {
        m.insert(key, module_name);
    };
}

/// Get a function's registered module. Returns None-MbValue if not registered.
pub fn mb_func_get_module(func: MbValue) -> MbValue {
    if let Some(module) = with_live_closure(func, |closure| closure.module.clone()) {
        return module
            .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
            .unwrap_or_else(MbValue::none);
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_modules.read().ok();
    guard
        .and_then(|m| m.get(&key).map(|s| MbValue::from_ptr(MbObject::new_str(s.clone()))))
        .unwrap_or_else(MbValue::none)
}

/// Register a function's positional argument count (for `f.__code__.co_argcount`).
/// Called at module init for every user-defined `def`. `argcount` excludes
/// `*args` / `**kwargs` (CPython counts those in co_varnames but not co_argcount).
pub fn mb_func_set_argcount(func: MbValue, argcount: MbValue) {
    let n = argcount.as_int().unwrap_or(0);
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_argcounts.write() {
        m.insert(key, n);
    };
}

/// Get a function's registered argument count. Returns None-MbValue if not
/// registered (so callers can distinguish a real `def` from an arbitrary value).
pub fn mb_func_get_argcount(func: MbValue) -> MbValue {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_argcounts.read().ok();
    guard
        .and_then(|m| m.get(&key).map(|n| MbValue::from_int(*n)))
        .unwrap_or_else(MbValue::none)
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
                    let items = lock.read().unwrap().to_vec();
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
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_varnames.write() {
        m.insert(key, collected);
    };
}

/// Get a function's registered varnames as a tuple MbValue (CPython returns a
/// tuple for `co_varnames`). Returns None-MbValue if not registered.
pub fn mb_func_get_varnames(func: MbValue) -> MbValue {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_varnames.read().ok();
    guard
        .and_then(|m| {
            m.get(&key).map(|names| {
                let items: Vec<MbValue> = names
                    .iter()
                    .map(|s| MbValue::from_ptr(MbObject::new_str(s.clone())))
                    .collect();
                MbValue::from_ptr(MbObject::new_tuple(items))
            })
        })
        .unwrap_or_else(MbValue::none)
}

/// Register extra code-object flags for a function (`CO_COROUTINE`, etc.).
pub fn mb_func_set_flags(func: MbValue, flags: MbValue) {
    let n = flags.as_int().unwrap_or(0);
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_flags.write() {
        m.insert(key, n);
    };
}

pub fn func_flags(func: MbValue) -> i64 {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_flags.read().ok();
    guard
        .and_then(|m| m.get(&key).copied())
        .unwrap_or(0)
}

/// Register a function's free variables for inspect.getclosurevars.
///
/// The list contains `(name, symbol_id)` pairs. Values still live in the
/// runtime global-id namespace because nested functions share captured locals
/// through StoreGlobal/LoadGlobal on the captured SymbolId.
pub fn mb_func_set_freevars(func: MbValue, freevars: MbValue) {
    let mut collected: Vec<(String, i64)> = Vec::new();
    if let Some(ptr) = freevars.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                for item in lock.read().unwrap().to_vec() {
                    let Some(pair_ptr) = item.as_ptr() else {
                        continue;
                    };
                    let ObjData::Tuple(ref pair) = (*pair_ptr).data else {
                        continue;
                    };
                    if pair.len() < 2 {
                        continue;
                    }
                    let Some(name) = extract_str(pair[0]) else {
                        continue;
                    };
                    let Some(sym_id) = pair[1].as_int() else {
                        continue;
                    };
                    collected.push((name, sym_id));
                }
            }
        }
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.func_freevars.write() {
        m.insert(key, collected);
    };
}

/// Registered free variables for a function, if any metadata was primed.
pub fn func_freevars(func: MbValue) -> Option<Vec<(String, i64)>> {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_freevars.read().ok()?;
    guard.get(&key).cloned()
}

/// True if `func` is a registered user-defined function (present in any of the
/// function metadata registries). Used to gate `__code__` synthesis so we don't
/// fabricate a code object for arbitrary ints / pointers.
pub fn mb_func_is_registered(func: MbValue) -> bool {
    if with_live_closure(func, |_| ()).is_some() {
        return true;
    }
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    state.func_names.read().map(|m| m.contains_key(&key)).unwrap_or(false)
        || state.func_argcounts.read().map(|m| m.contains_key(&key)).unwrap_or(false)
        || state.func_varnames.read().map(|m| m.contains_key(&key)).unwrap_or(false)
}

pub(crate) fn mb_closure_set_wrapped(closure_handle: MbValue, wrapped: MbValue) -> bool {
    unsafe {
        super::rc::retain_if_ptr(wrapped);
    }
    let prev = with_live_closure_mut(closure_handle, |closure| closure.wrapped.replace(wrapped));
    match prev {
        Some(Some(prev)) => {
            unsafe {
                super::rc::release_if_ptr(prev);
            }
            true
        }
        Some(None) => true,
        None => {
            unsafe {
                super::rc::release_if_ptr(wrapped);
            }
            false
        }
    }
}

pub(crate) fn mb_closure_get_wrapped(closure_handle: MbValue) -> Option<MbValue> {
    with_live_closure(closure_handle, |closure| closure.wrapped).flatten()
}

/// Register a function's source location (`co_firstlineno` / `co_filename`).
/// Called at module init alongside the other metadata priming calls; lambdas
/// register at closure-creation time.
pub fn mb_func_set_srcinfo(func: MbValue, line: MbValue, filename: MbValue) {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    if let Some(n) = line.as_int() {
        if n > 0 {
            if let Ok(mut m) = state.func_lines.write() {
                m.insert(key, n);
            };
        }
    }
    if let Some(f) = extract_str(filename) {
        if !f.is_empty() {
            if let Ok(mut m) = state.func_files.write() {
                m.insert(key, f);
            };
        }
    }
}

/// First source line of a registered function, or None when unknown.
pub fn func_line(func: MbValue) -> Option<i64> {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_lines.read().ok()?;
    guard.get(&key).copied()
}

/// Source filename of a registered function, or None when unknown.
pub fn func_file(func: MbValue) -> Option<String> {
    let key = func.to_bits();
    let state = super::program_state::current_program_state();
    let guard = state.func_files.read().ok()?;
    guard.get(&key).cloned()
}

// ── Cell Variables (for nonlocal/closure mutable capture) ──
// Vec-indexed by cell slot for O(1) lookup (#1199). Raw handles are stored as
// `CELL_ID_BASE + slot_index` so ordinary small ints never alias a live cell.

fn cell_slot_index(raw: i64, len: usize) -> Option<usize> {
    let slot = raw.checked_sub(CELL_ID_BASE)?;
    let idx = usize::try_from(slot).ok()?;
    (idx < len).then_some(idx)
}

/// Create a new cell variable initialized with a value.
/// Returns a handle (integer) that can be shared between scopes.
pub fn mb_cell_new(value: MbValue) -> MbValue {
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    let state = super::program_state::current_program_state();
    let mut vec = state.cells.write().unwrap();
    let id = CELL_ID_BASE + vec.len() as i64;
    vec.push(Some(value));
    MbValue::from_int(id)
}

/// Create a new, genuinely EMPTY cell (`types.CellType()` with no argument;
/// #896). Distinct from `mb_cell_new(none)`, which holds the Python value
/// `None` — an empty cell's slot is `None` at the Rust level, so
/// `cell_contents` reads raise ValueError until the cell is set via
/// `mb_cell_set`.
pub fn mb_cell_new_empty() -> MbValue {
    let state = super::program_state::current_program_state();
    let mut vec = state.cells.write().unwrap();
    let id = CELL_ID_BASE + vec.len() as i64;
    vec.push(None);
    MbValue::from_int(id)
}

/// Get the value stored in a cell.
pub fn mb_cell_get(cell_handle: MbValue) -> MbValue {
    if let Some(id) = cell_handle.as_int() {
        let state = super::program_state::current_program_state();
        let vec = state.cells.read().unwrap();
        let Some(idx) = cell_slot_index(id, vec.len()) else {
            return MbValue::none();
        };
        let val = vec
            .get(idx)
            .and_then(|slot| *slot)
            .unwrap_or(MbValue::none());
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        val
    } else {
        MbValue::none()
    }
}

/// Result of a `cell_contents` attribute read (#896): distinguishes a handle
/// that isn't a live cell at all (so the generic attribute-lookup fallback
/// should run instead) from a genuinely empty cell (unset — reads raise
/// ValueError) from a filled cell (holds a real value, including Python
/// `None`).
pub enum CellContentsRead {
    NotACell,
    Empty,
    Value(MbValue),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CellCompareValue {
    NotACell,
    Empty,
    Value(MbValue),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ActiveCellSnapshot {
    Empty,
    Value(MbValue),
}

pub(crate) fn mb_cell_compare_value(cell_handle: MbValue) -> CellCompareValue {
    let Some(id) = cell_handle.as_int() else {
        return CellCompareValue::NotACell;
    };
    let state = super::program_state::current_program_state();
    let vec = state.cells.read().unwrap();
    let Some(idx) = cell_slot_index(id, vec.len()) else {
        return CellCompareValue::NotACell;
    };
    match vec.get(idx) {
        None => CellCompareValue::NotACell,
        Some(None) => CellCompareValue::Empty,
        Some(Some(val)) => CellCompareValue::Value(*val),
    }
}

pub fn mb_cell_handle_raw_is_live(raw: i64) -> i64 {
    let state = super::program_state::current_program_state();
    let vec = state.cells.read().unwrap();
    i64::from(cell_slot_index(raw, vec.len()).is_some())
}

/// Checked `cell_contents` read used by the `.cell_contents` attribute
/// accessor. Unlike `mb_cell_get`, this never silently collapses "empty" and
/// "invalid handle" into a `None` value — callers use the variant to decide
/// whether to raise ValueError (empty) or fall through (not a cell).
pub fn mb_cell_contents_read(cell_handle: MbValue) -> CellContentsRead {
    match mb_cell_compare_value(cell_handle) {
        CellCompareValue::NotACell => CellContentsRead::NotACell,
        CellCompareValue::Empty => CellContentsRead::Empty,
        CellCompareValue::Value(val) => {
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            CellContentsRead::Value(val)
        }
    }
}

/// Set the value stored in a cell.
pub fn mb_cell_set(cell_handle: MbValue, value: MbValue) {
    if let Some(id) = cell_handle.as_int() {
        unsafe {
            super::rc::retain_if_ptr(value);
        }
        let state = super::program_state::current_program_state();
        let prev = {
            let mut vec = state.cells.write().unwrap();
            if let Some(idx) = cell_slot_index(id, vec.len()) {
                let prev = vec[idx];
                vec[idx] = Some(value);
                prev
            } else {
                None
            }
        };
        if let Some(prev) = prev {
            unsafe {
                super::rc::release_if_ptr(prev);
            }
        }
    }
}

pub fn mb_cell_clear(cell_handle: MbValue) {
    if let Some(id) = cell_handle.as_int() {
        let state = super::program_state::current_program_state();
        let prev = {
            let mut vec = state.cells.write().unwrap();
            if let Some(idx) = cell_slot_index(id, vec.len()) {
                let prev = vec[idx];
                vec[idx] = None;
                prev
            } else {
                None
            }
        };
        if let Some(prev) = prev {
            unsafe {
                super::rc::release_if_ptr(prev);
            }
        }
    }
}

fn active_cell_for_id(key: i64) -> MbValue {
    let scoped = scoped_symbol_key(key);
    ACTIVE_CELLS.with(|cells| {
        let mut cells = cells.borrow_mut();
        if let Some(cell) = cells.get(&scoped).copied() {
            return cell;
        }
        let state = super::program_state::current_program_state();
        let initial = state
            .global_ids
            .read()
            .unwrap()
            .get(&scoped)
            .copied()
            .unwrap_or_else(MbValue::none);
        let cell = mb_cell_new(initial);
        cells.insert(scoped, cell);
        cell
    })
}

#[inline]
fn extract_symbol_key(id: MbValue) -> i64 {
    id.as_int().unwrap_or_else(|| id.to_bits() as i64)
}

pub fn mb_capture_cell_set_id(id: MbValue, value: MbValue) {
    let key = extract_symbol_key(id);
    let cell = active_cell_for_id(key);
    mb_cell_set(cell, value);
}

pub fn mb_capture_cell_reset_id(id: MbValue, value: MbValue) {
    let key = extract_symbol_key(id);
    let cell = mb_cell_new(value);
    ACTIVE_CELLS.with(|cells| {
        cells.borrow_mut().insert(scoped_symbol_key(key), cell);
    });
}

/// #1053: pre-vivify a fresh, genuinely EMPTY (unbound) cell for `id` — used
/// when a nested closure captures an enclosing local before that local's own
/// first textual assignment has run this invocation (the enclosing-scope
/// prescan fix lets a nested def capture a name assigned later in the
/// textual body, matching CPython's whole-body scoping). Unlike
/// `mb_capture_cell_reset_id`, this installs an empty cell (Rust-level
/// `None` slot, distinct from the Python value `None`) so a read through it
/// before the real assignment still raises NameError instead of silently
/// observing an unbound placeholder as `None`.
pub fn mb_capture_cell_reset_empty_id(id: MbValue) {
    let key = extract_symbol_key(id);
    let cell = mb_cell_new_empty();
    ACTIVE_CELLS.with(|cells| {
        cells.borrow_mut().insert(scoped_symbol_key(key), cell);
    });
}

fn active_cell_get_id_raw(key: i64) -> Option<MbValue> {
    let scoped = scoped_symbol_key(key);
    let cell = ACTIVE_CELLS.with(|cells| cells.borrow().get(&scoped).copied())?;
    // #1053: an empty cell (see `mb_capture_cell_reset_empty_id`) is a state
    // ONLY EVER produced by that function — it means a nested closure
    // captured this enclosing-scope local before the enclosing function's
    // own first textual assignment to it has run yet this invocation
    // (CPython's `LOAD_DEREF` does the equivalent runtime NULL-cell check
    // and raises `NameError` unconditionally; there's no legitimate reading
    // of an empty cell). Raise directly here rather than falling through to
    // `mb_global_get_id_raw`'s missing-global fallback, since that fallback
    // is gated by `missing_global_should_raise_name_error()` — a flag that's
    // off for ordinary function bodies (only ever enabled for the unrelated
    // pep695 generic-param scenario) and so would silently swallow this
    // as `None` instead of raising.
    match mb_cell_contents_read(cell) {
        CellContentsRead::Value(v) => Some(v),
        CellContentsRead::Empty => {
            let state = super::program_state::current_program_state();
            let name = state
                .module_sym_info
                .read()
                .ok()
                .and_then(|m| m.get(&key).map(|(name, _)| name.clone()))
                .unwrap_or_else(|| format!("<symbol {key}>"));
            raise_missing_global_name_error(&name);
            Some(MbValue::none())
        }
        CellContentsRead::NotACell => None,
    }
}

fn active_cell_set_id_raw(key: i64, value: MbValue) -> bool {
    let scoped = scoped_symbol_key(key);
    let cell = ACTIVE_CELLS.with(|cells| cells.borrow().get(&scoped).copied());
    if let Some(cell) = cell {
        mb_cell_set(cell, value);
        true
    } else {
        false
    }
}

// ── nonlocal/global support ──

// Thread-local runtime execution position / flags.
thread_local! {
    static MISSING_GLOBAL_RAISES_NAME_ERROR: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
    /// Stack of "active module" SymbolId sets (#983). The top entry is the
    /// set of SymbolIds owned by whichever module's top-level code is
    /// currently executing. `compile_and_exec_module` (module.rs) pushes a
    /// module's own id set before running its `__main__` and pops it after,
    /// so nested imports triggered mid-execution are bracketed by their
    /// parent's ownership set. Every module's SymbolTable restarts numbering
    /// from the same builtin baseline (resolve/scope.rs), so raw SymbolId
    /// integers are unique only WITHIN one compilation, not across nested
    /// compiles — `merge_global_id_namespace` consults the top of this stack
    /// to avoid donating a finished nested module's leftover raw global into
    /// a slot that numerically collides with the still-executing outer
    /// module's own (possibly not-yet-written) slot.
    static ACTIVE_MODULE_SYM_IDS: std::cell::RefCell<Vec<HashSet<i64>>> =
        std::cell::RefCell::new(Vec::new());
}

/// Push the set of SymbolIds owned by the module about to execute its
/// top-level code (#983). Must be paired with `pop_active_module_sym_ids`.
pub fn push_active_module_sym_ids(ids: HashSet<i64>) {
    ACTIVE_MODULE_SYM_IDS.with(|s| s.borrow_mut().push(ids));
}

/// Pop the SymbolId set pushed by the matching `push_active_module_sym_ids`.
pub fn pop_active_module_sym_ids() {
    ACTIVE_MODULE_SYM_IDS.with(|s| {
        s.borrow_mut().pop();
    });
}

pub(crate) fn set_missing_global_name_error_enabled(enabled: bool) -> bool {
    MISSING_GLOBAL_RAISES_NAME_ERROR.with(|flag| flag.replace(enabled))
}

pub(crate) fn restore_missing_global_name_error_enabled(previous: bool) {
    MISSING_GLOBAL_RAISES_NAME_ERROR.with(|flag| flag.set(previous));
}

fn missing_global_should_raise_name_error() -> bool {
    MISSING_GLOBAL_RAISES_NAME_ERROR.with(|flag| flag.get())
}

fn raise_missing_global_name_error(name: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("NameError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!("name '{name}' is not defined"))),
    );
}

fn lookup_named_runtime_global(name: &str) -> Option<MbValue> {
    let state = super::program_state::current_program_state();
    if let Some(value) = {
        let g = state.globals.read().unwrap();
        g.get(name).copied()
    } {
        return Some(value);
    }

    let current_module = current_active_module_name_arc();
    if let Some(value) = {
        let g_ids = state.global_ids.read().unwrap();
        let sym_info = state.module_sym_info.read().unwrap();
        g_ids.iter().find_map(|(key, value)| {
            if key.module != current_module {
                return None;
            }
            let (symbol_name, _) = sym_info.get(&key.symbol)?;
            (symbol_name == name).then_some(*value)
        })
    } {
        return Some(value);
    }

    state.module_func_info.read().ok().and_then(|funcs| funcs.get(name).copied())
}

/// Get a global variable by name.
pub fn mb_global_get(name: MbValue) -> MbValue {
    let var_name = extract_str(name).unwrap_or_default();
    let state = super::program_state::current_program_state();
    let val = {
        let g = state.globals.read().unwrap();
        g.get(&var_name).copied()
    };
    if val.is_none() && missing_global_should_raise_name_error() {
        raise_missing_global_name_error(&var_name);
    }
    let val = val.unwrap_or_else(MbValue::none);
    unsafe {
        super::rc::retain_if_ptr(val);
    }
    val
}

/// Guarded runtime read for an identifier the checker deferred to `Any`
/// because it could not resolve it ANYWHERE — not a local, not an
/// outer-scope capture, not a module/global/builtin symbol (#1048). This is
/// the terminal state of `allow_runtime_unresolved_names`'s run-mode
/// deferral (check_expr.rs): `mamba run` intentionally doesn't hard-error
/// on a name it can't statically resolve (CPython itself only resolves
/// free names at read time, not at compile time), but the terminal lowering
/// used to be a silent `None`-valued read (`ast_to_hir.rs`'s `Expr::Ident`
/// arm returned `None`, which unwound to a dropped statement or an inert
/// `None`). CPython instead raises `NameError` at the moment of the read.
///
/// Checks the string-keyed `GLOBAL_NAMESPACE` — the same namespace a
/// wildcard `from module import *` binds into via `mb_global_set`
/// (`runtime/module.rs`) — so a name that becomes legitimately bound at
/// runtime through that path still resolves; genuine forward references,
/// late-bound function calls, and builtins never reach this function at all
/// (they resolve through the checker's ordinary symbol table at HIR-lowering
/// time, same as `mamba build`). Anything still missing raises NameError
/// with CPython's exact message, mirroring `mb_unbound_local_error_value`'s
/// call-then-raise-then-return-None shape so the standard post-call
/// exception-check/try-except machinery picks it up unchanged.
pub fn mb_deferred_name_read(name: MbValue) -> MbValue {
    let var_name = extract_str(name).unwrap_or_default();
    let val = lookup_named_runtime_global(&var_name);
    if let Some(v) = val {
        unsafe {
            super::rc::retain_if_ptr(v);
        }
        return v;
    }
    raise_missing_global_name_error(&var_name);
    MbValue::none()
}

/// Set a global variable.
pub fn mb_global_set(name: MbValue, value: MbValue) {
    let var_name = extract_str(name).unwrap_or_default();
    // Retain so value survives the JIT epilogue releasing the source VReg.
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    let state = super::program_state::current_program_state();
    let old = {
        let mut g = state.globals.write().unwrap();
        g.insert(var_name, value)
    };
    if let Some(prev) = old {
        release_rebound_global_value(prev);
    }
}

/// Get a global variable by integer id (SymbolId). Used by REPL since
/// MirConst::Str is not yet compiled to actual string pointers.
/// The id is passed as raw i64 (not NaN-boxed).
pub fn mb_global_get_id(id: MbValue) -> MbValue {
    let key = extract_symbol_key(id);
    mb_global_get_id_raw(key)
}

/// Read a global-id namespace value by raw SymbolId.
pub fn mb_global_get_id_raw(key: i64) -> MbValue {
    if let Some(val) = active_cell_get_id_raw(key) {
        return val;
    }
    let scoped = scoped_symbol_key(key);
    let state = super::program_state::current_program_state();
    let val = {
        let g_ids = state.global_ids.read().unwrap();
        let v = g_ids.get(&scoped).copied();
        if let Some(val) = v {
            unsafe {
                super::rc::retain_if_ptr(val);
            }
        }
        v
    };
    if val.is_none() && missing_global_should_raise_name_error() {
        let name = state
            .module_sym_info
            .read()
            .ok()
            .and_then(|m| m.get(&key).map(|(name, _)| name.clone()))
            .unwrap_or_else(|| format!("<symbol {key}>"));
        raise_missing_global_name_error(&name);
    }
    val.unwrap_or_else(MbValue::none)
}

/// Set a global variable by integer id (SymbolId).
/// The id is passed as raw i64 (not NaN-boxed).
pub fn mb_global_set_id(id: MbValue, value: MbValue) {
    let key = extract_symbol_key(id);
    if active_cell_set_id_raw(key, value) {
        return;
    }
    let scoped = scoped_symbol_key(key);
    // Retain the value so it survives the JIT epilogue releasing the source VReg.
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    let state = super::program_state::current_program_state();
    let old = {
        let mut g_ids = state.global_ids.write().unwrap();
        g_ids.insert(scoped, value)
    };
    // Release the previous value being overwritten.
    if let Some(prev) = old {
        release_rebound_global_value(prev);
    }
}

/// Delete a global variable by integer id (SymbolId).
/// The id is passed as raw i64 (not NaN-boxed).
pub fn mb_global_del_id(id: MbValue) {
    let key = extract_symbol_key(id);
    let scoped = scoped_symbol_key(key);
    let state = super::program_state::current_program_state();
    let old = {
        let mut g_ids = state.global_ids.write().unwrap();
        g_ids.remove(&scoped)
    };
    if let Some(prev) = old {
        unsafe {
            super::rc::release_if_ptr(prev);
        }
    }
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

fn current_module_symbol_entries(ns: &HashMap<ScopedSymbolKey, MbValue>) -> HashMap<i64, MbValue> {
    let current_module = current_active_module_name_arc();
    ns.iter()
        .filter(|(key, _)| key.module == current_module)
        .map(|(key, value)| (key.symbol, *value))
        .collect()
}

/// Snapshot only the active module's raw SymbolId globals.
pub(crate) fn snapshot_current_module_global_id_namespace() -> HashMap<i64, MbValue> {
    let state = super::program_state::current_program_state();
    let g_ids = state.global_ids.read().unwrap();
    current_module_symbol_entries(&g_ids)
}

/// Check if any global in `ProgramState` matches a predicate.
pub(crate) fn global_id_namespace_any<F: FnMut(MbValue) -> bool>(mut f: F) -> bool {
    let state = super::program_state::current_program_state();
    let g_ids = state.global_ids.read().unwrap();
    g_ids.values().copied().any(|v| f(v))
}

/// Replace the full ACTIVE_CELLS map and return the previous contents.
pub(crate) fn replace_active_cells(
    next: HashMap<ScopedSymbolKey, MbValue>,
) -> HashMap<ScopedSymbolKey, MbValue> {
    ACTIVE_CELLS.with(|cells| {
        let mut b = cells.borrow_mut();
        let prev = b.clone();
        *b = next;
        prev
    })
}

/// Snapshot the full ACTIVE_CELLS map (cloning thread-local active bindings).
pub(crate) fn snapshot_active_cells() -> HashMap<ScopedSymbolKey, MbValue> {
    ACTIVE_CELLS.with(|cells| cells.borrow().clone())
}

/// Merge ACTIVE_CELLS from another thread snapshot into active cells.
pub(crate) fn merge_active_cells(entries: &HashMap<ScopedSymbolKey, MbValue>) {
    ACTIVE_CELLS.with(|cells| {
        let mut cells = cells.borrow_mut();
        for (key, cell) in entries {
            cells.entry(key.clone()).or_insert(*cell);
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

/// Replace the current MODULE_SYM_INFO with a new one.
pub fn set_module_sym_info(info: HashMap<i64, (String, SymTy)>) {
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.module_sym_info.write() {
        *m = info;
    };
}

/// Replace the current MODULE_FUNC_INFO with a new one.
pub fn set_module_func_info(info: HashMap<String, MbValue>) {
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.module_func_info.write() {
        *m = info;
    };
}

/// Snapshot for save_and_restore around module imports.
pub fn save_and_clear_module_sym_info() -> (HashMap<i64, (String, SymTy)>, HashMap<String, MbValue>)
{
    let state = super::program_state::current_program_state();
    let syms = state.module_sym_info.write().map(|mut m| {
        let saved = m.clone();
        m.clear();
        saved
    }).unwrap_or_default();
    let funcs = state.module_func_info.write().map(|mut m| {
        let saved = m.clone();
        m.clear();
        saved
    }).unwrap_or_default();
    (syms, funcs)
}

/// Restore a previously saved sym_info / func_info pair.
pub fn restore_module_sym_info(saved: (HashMap<i64, (String, SymTy)>, HashMap<String, MbValue>)) {
    let state = super::program_state::current_program_state();
    if let Ok(mut m) = state.module_sym_info.write() {
        *m = saved.0;
    };
    if let Ok(mut m) = state.module_func_info.write() {
        *m = saved.1;
    };
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1979" reason="logic section in closure.rs is hand-written pending codegen support">
/// Build a dict containing the current module's globals, drawing from
/// MODULE_SYM_INFO + GLOBAL_ID_NAMESPACE + MODULE_FUNC_INFO. Skips dunder
/// names except the standard CPython-visible ones.
pub fn build_globals_dict() -> MbValue {
    use super::dict_ops;
    let dict = dict_ops::mb_dict_new();
    let state = super::program_state::current_program_state();

    let id_ns = snapshot_current_module_global_id_namespace();
    let sym_info = state.module_sym_info.read().map(|m| m.clone()).unwrap_or_default();

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
        unsafe {
            super::rc::release_if_ptr(key);
        }
    }

    let func_info = state.module_func_info.read().map(|m| m.clone()).unwrap_or_default();
    for (name, fv) in &func_info {
        if name.starts_with("__") && name != "__name__" && name != "__doc__" && name != "__all__" {
            continue;
        }
        let key = MbValue::from_ptr(super::rc::MbObject::new_str(name.clone()));
        dict_ops::mb_dict_setitem(dict, key, *fv);
        unsafe {
            super::rc::release_if_ptr(key);
        }
    }
    dict
}
// </HANDWRITE>
// HANDWRITE-END

// ── Cleanup ──

/// Reset closure-related thread_local state and process-global ProgramState.
/// Called as part of centralized runtime cleanup between test executions.
/// Values are cleared without releasing — refcount imbalance from mixed
/// code paths makes release unsafe. Leaked objects reclaimed at process exit.
pub(crate) fn cleanup_all_closures() {
    cleanup_thread_local_closures();
    super::program_state::current_program_state().clear();
}

/// Reset closure-related thread_local state to defaults without clearing
/// process-global ProgramState.
pub(crate) fn cleanup_thread_local_closures() {
    let _ = ACTIVE_CELLS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = ACTIVE_MODULE_NAMES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = ACTIVE_QUALNAME_CONTEXTS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = ACTIVE_MODULE_SYM_IDS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    MISSING_GLOBAL_RAISES_NAME_ERROR.with(|flag| flag.set(false));
}

#[cfg(test)]
mod tests {
    use crate::runtime::builtins;

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

    #[test]
    fn test_module_scoped_global_ids_do_not_collide() {
        cleanup_all_closures();

        mb_global_set_id(MbValue::from_bits(7), MbValue::from_int(11));
        push_active_module_name("other.mod".to_string());
        mb_global_set_id(MbValue::from_bits(7), MbValue::from_int(22));
        assert_eq!(mb_global_get_id(MbValue::from_bits(7)).as_int(), Some(22));
        pop_active_module_name();

        assert_eq!(mb_global_get_id(MbValue::from_bits(7)).as_int(), Some(11));
        push_active_module_name("other.mod".to_string());
        assert_eq!(mb_global_get_id(MbValue::from_bits(7)).as_int(), Some(22));
        pop_active_module_name();

        cleanup_all_closures();
    }

    #[test]
    fn test_module_scoped_active_cells_do_not_collide() {
        cleanup_all_closures();

        mb_capture_cell_reset_id(MbValue::from_bits(9), MbValue::from_int(101));
        push_active_module_name("other.mod".to_string());
        mb_capture_cell_reset_id(MbValue::from_bits(9), MbValue::from_int(202));
        assert_eq!(mb_global_get_id(MbValue::from_bits(9)).as_int(), Some(202));
        pop_active_module_name();

        assert_eq!(mb_global_get_id(MbValue::from_bits(9)).as_int(), Some(101));
        push_active_module_name("other.mod".to_string());
        assert_eq!(mb_global_get_id(MbValue::from_bits(9)).as_int(), Some(202));
        pop_active_module_name();

        cleanup_all_closures();
    }

    #[test]
    fn test_closure_local_function_metadata_round_trips_without_plain_ints() {
        cleanup_all_closures();

        let closure = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("wrapper".to_string())),
            MbValue::from_int(7),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let wrapped = MbValue::from_ptr(MbObject::new_str("wrapped-target".to_string()));

        mb_func_set_name(
            closure,
            MbValue::from_ptr(MbObject::new_str("wrapped_name".to_string())),
        );
        mb_func_set_doc(
            closure,
            MbValue::from_ptr(MbObject::new_str("wrapped doc".to_string())),
        );
        mb_func_set_module(
            closure,
            MbValue::from_ptr(MbObject::new_str("wrapped.mod".to_string())),
        );
        assert!(mb_closure_set_wrapped(closure, wrapped));

        assert_eq!(
            extract_str(mb_func_get_name(closure)).as_deref(),
            Some("wrapped_name")
        );
        assert_eq!(
            extract_str(mb_func_get_doc(closure)).as_deref(),
            Some("wrapped doc")
        );
        assert_eq!(
            extract_str(mb_func_get_module(closure)).as_deref(),
            Some("wrapped.mod")
        );
        assert_eq!(mb_closure_get_wrapped(closure), Some(wrapped));

        let plain = MbValue::from_int(4242);
        mb_func_set_name(
            plain,
            MbValue::from_ptr(MbObject::new_str("plain".to_string())),
        );
        mb_func_set_doc(
            plain,
            MbValue::from_ptr(MbObject::new_str("plain doc".to_string())),
        );
        mb_func_set_module(
            plain,
            MbValue::from_ptr(MbObject::new_str("plain.mod".to_string())),
        );

        assert!(mb_func_get_name(plain).is_none());
        assert!(mb_func_get_doc(plain).is_none());
        assert!(mb_func_get_module(plain).is_none());
        assert_eq!(mb_closure_get_wrapped(plain), None);

        cleanup_all_closures();
    }

    #[test]
    fn test_global_set_releases_overwritten_live_closure_handles() {
        cleanup_all_closures();

        let name = MbValue::from_ptr(MbObject::new_str("wrapped_global".to_string()));
        let closure1 = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("c1".to_string())),
            MbValue::from_int(11),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let closure2 = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("c2".to_string())),
            MbValue::from_int(22),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );

        mb_global_set(name, closure1);
        assert!(release_closure_handle_if_live(closure1));
        assert_eq!(mb_closure_get_func(closure1).as_int(), Some(11));

        mb_global_set(name, closure2);
        assert!(release_closure_handle_if_live(closure2));
        assert!(mb_closure_get_func(closure1).is_none());
        assert_eq!(mb_closure_get_func(closure2).as_int(), Some(22));

        let plain = MbValue::from_int(77);
        assert!(!retain_closure_handle_if_live(plain));
        assert!(!release_closure_handle_if_live(plain));
        mb_global_set(name, plain);
        assert!(mb_closure_get_func(closure2).is_none());
        assert_eq!(mb_global_get(name).as_int(), Some(77));

        cleanup_all_closures();
    }

    #[test]
    fn test_global_set_id_releases_overwritten_live_closure_handles() {
        cleanup_all_closures();

        let id = MbValue::from_bits(707);
        let closure1 = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("c1".to_string())),
            MbValue::from_int(101),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let closure2 = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("c2".to_string())),
            MbValue::from_int(202),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );

        mb_global_set_id(id, closure1);
        assert!(release_closure_handle_if_live(closure1));
        assert_eq!(mb_closure_get_func(closure1).as_int(), Some(101));

        mb_global_set_id(id, closure2);
        assert!(release_closure_handle_if_live(closure2));
        assert!(mb_closure_get_func(closure1).is_none());
        assert_eq!(mb_closure_get_func(closure2).as_int(), Some(202));

        let plain = MbValue::from_int(88);
        assert!(!retain_closure_handle_if_live(plain));
        assert!(!release_closure_handle_if_live(plain));
        mb_global_set_id(id, plain);
        assert!(mb_closure_get_func(closure2).is_none());
        assert_eq!(mb_global_get_id(id).as_int(), Some(88));

        cleanup_all_closures();
    }

    #[test]
    fn test_released_closure_slot_is_reused_on_next_allocation() {
        cleanup_all_closures();

        let closure1 = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("c1".to_string())),
            MbValue::from_int(301),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let closure2 = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("c2".to_string())),
            MbValue::from_int(302),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        assert_eq!(closure1.as_int(), Some(CLOSURE_HANDLE_BASE));
        assert_eq!(closure2.as_int(), Some(CLOSURE_HANDLE_BASE + 1));

        mb_closure_release(closure1);
        assert!(mb_closure_get_func(closure1).is_none());

        let closure3 = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("c3".to_string())),
            MbValue::from_int(303),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        assert_eq!(closure3.as_int(), Some(CLOSURE_HANDLE_BASE));
        assert_eq!(mb_closure_get_func(closure3).as_int(), Some(303));
        assert_eq!(mb_closure_get_func(closure2).as_int(), Some(302));

        cleanup_all_closures();
    }

    #[test]
    fn test_closure_handle_retain_release_refcounts() {
        cleanup_all_closures();

        let closure = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("rc".to_string())),
            MbValue::from_int(401),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        assert!(retain_closure_handle_if_live(closure));
        assert!(release_closure_handle_if_live(closure));
        assert_eq!(mb_closure_get_func(closure).as_int(), Some(401));
        assert!(release_closure_handle_if_live(closure));
        assert!(mb_closure_get_func(closure).is_none());

        let plain = MbValue::from_int(123);
        assert!(!retain_closure_handle_if_live(plain));
        assert!(!release_closure_handle_if_live(plain));

        cleanup_all_closures();
    }

    #[test]
    fn test_global_overwrite_keeps_aliased_closure_live_until_last_release() {
        cleanup_all_closures();

        let name = MbValue::from_ptr(MbObject::new_str("aliased_global".to_string()));
        let closure = mb_closure_new(
            MbValue::from_ptr(MbObject::new_str("aliased".to_string())),
            MbValue::from_int(501),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );

        assert!(retain_closure_handle_if_live(closure));
        mb_global_set(name, closure);
        assert!(release_closure_handle_if_live(closure));
        assert_eq!(mb_closure_get_func(closure).as_int(), Some(501));

        mb_global_set(name, MbValue::from_int(0));
        assert_eq!(mb_closure_get_func(closure).as_int(), Some(501));

        assert!(release_closure_handle_if_live(closure));
        assert!(mb_closure_get_func(closure).is_none());

        cleanup_all_closures();
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
    fn test_deferred_name_read_falls_back_to_symbol_id_globals() {
        cleanup_all_closures();

        let mut sym_info = HashMap::new();
        sym_info.insert(4242, ("X".to_string(), SymTy::Boxed));
        set_module_sym_info(sym_info);

        let class_value = MbValue::from_ptr(MbObject::new_str("class-object".into()));
        mb_global_set_id(MbValue::from_bits(4242), class_value);

        let name = MbValue::from_ptr(MbObject::new_str("X".into()));
        let loaded = mb_deferred_name_read(name);
        assert_eq!(extract_str(loaded).as_deref(), Some("class-object"));

        cleanup_all_closures();
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
    fn test_cell_comparisons_follow_contents() {
        cleanup_all_closures();

        let two = mb_cell_new(MbValue::from_int(2));
        let three = mb_cell_new(MbValue::from_int(3));
        let neg_int = mb_cell_new(MbValue::from_int(-36));
        let neg_float = mb_cell_new(MbValue::from_float(-36.0));
        let truthy = mb_cell_new(MbValue::from_bool(true));
        let empty_a = mb_cell_new_empty();
        let empty_b = mb_cell_new_empty();
        let saturday = mb_cell_new(MbValue::from_ptr(MbObject::new_str("saturday".to_string())));

        assert_eq!(builtins::mb_lt(two, three).as_bool(), Some(true));
        assert_eq!(builtins::mb_eq(neg_int, neg_float).as_bool(), Some(true));
        assert_eq!(builtins::mb_gt(truthy, empty_a).as_bool(), Some(true));
        assert_eq!(builtins::mb_lt(empty_b, saturday).as_bool(), Some(true));
        assert_eq!(builtins::mb_eq(empty_a, empty_b).as_bool(), Some(true));

        cleanup_all_closures();
    }

    #[test]
    fn test_cell_handles_do_not_alias_plain_ints() {
        cleanup_all_closures();

        let one = mb_cell_new(MbValue::from_int(10));
        let two = mb_cell_new(MbValue::from_int(20));
        let three = mb_cell_new_empty();

        assert_eq!(mb_cell_handle_raw_is_live(1), 0);
        assert_eq!(mb_cell_handle_raw_is_live(2), 0);
        assert_eq!(mb_cell_handle_raw_is_live(3), 0);

        assert_eq!(mb_cell_handle_raw_is_live(one.as_int().unwrap()), 1);
        assert_eq!(mb_cell_handle_raw_is_live(two.as_int().unwrap()), 1);
        assert_eq!(mb_cell_handle_raw_is_live(three.as_int().unwrap()), 1);

        assert_eq!(
            builtins::mb_lt(MbValue::from_int(1), MbValue::from_int(2)).as_bool(),
            Some(true),
        );
        assert_eq!(
            builtins::mb_gt(MbValue::from_int(3), MbValue::from_int(2)).as_bool(),
            Some(true),
        );

        cleanup_all_closures();
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

    #[test]
    fn test_func_pos_defaults_assignment_updates_params_and_closure_defaults() {
        cleanup_all_closures();
        let name = MbValue::from_ptr(MbObject::new_str("pos_defaults".into()));
        let func = MbValue::from_int(11);
        let caps = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, caps);

        let params = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str("a".into())),
                MbValue::from_int(0),
                MbValue::from_int(0),
                MbValue::none(),
                MbValue::none(),
            ])),
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str("b".into())),
                MbValue::from_int(0),
                MbValue::from_int(1),
                MbValue::from_int(2),
                MbValue::none(),
            ])),
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str("c".into())),
                MbValue::from_int(1),
                MbValue::from_int(1),
                MbValue::from_int(3),
                MbValue::none(),
            ])),
        ]));
        mb_func_set_params(closure, params);

        let assigned = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        assert!(mb_func_set_pos_defaults(closure, assigned));

        let defaults = closure_defaults(closure);
        assert_eq!(
            defaults
                .iter()
                .filter_map(|v| v.as_int())
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        let params = func_params(closure).expect("params should remain registered");
        let got: Vec<(bool, Option<i64>)> = params
            .iter()
            .map(|p| (p.has_default, p.default.as_int()))
            .collect();
        assert_eq!(got, vec![(true, Some(1)), (true, Some(2)), (true, Some(3))]);

        cleanup_all_closures();
    }

    #[test]
    fn test_func_annotations_fall_back_to_enclosing_class_type_params() {
        cleanup_all_closures();
        let name = MbValue::from_ptr(MbObject::new_str("meth".into()));
        let func = MbValue::from_int(12);
        let caps = MbValue::from_ptr(MbObject::new_list(vec![]));
        let closure = mb_closure_new(name, func, caps);

        let params = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str("arg".into())),
                MbValue::from_int(0),
                MbValue::from_int(0),
                MbValue::none(),
                MbValue::from_ptr(MbObject::new_str("__T".into())),
            ])),
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str("arg2".into())),
                MbValue::from_int(0),
                MbValue::from_int(0),
                MbValue::none(),
                MbValue::from_ptr(MbObject::new_str("__U".into())),
            ])),
        ]));
        mb_func_set_params(closure, params);

        let class_t = crate::runtime::pep695::make_typevar_instance("__T", 0, vec![], None);
        let method_u = crate::runtime::pep695::make_typevar_instance("__U", 0, vec![], None);
        crate::runtime::pep695::func_attrs_set(
            closure,
            MbValue::from_ptr(MbObject::new_str("__type_params__".into())),
            MbValue::from_ptr(MbObject::new_tuple(vec![method_u])),
        );
        crate::runtime::pep695::func_attrs_set(
            closure,
            MbValue::from_ptr(MbObject::new_str("__mb_class_type_params__".into())),
            MbValue::from_ptr(MbObject::new_tuple(vec![class_t])),
        );

        let annotations = mb_func_get_annotations(closure);
        let arg = crate::runtime::dict_ops::mb_dict_get(
            annotations,
            MbValue::from_ptr(MbObject::new_str("arg".into())),
            MbValue::none(),
        );
        let arg2 = crate::runtime::dict_ops::mb_dict_get(
            annotations,
            MbValue::from_ptr(MbObject::new_str("arg2".into())),
            MbValue::none(),
        );
        assert_eq!(arg.to_bits(), class_t.to_bits());
        assert_eq!(arg2.to_bits(), method_u.to_bits());

        cleanup_all_closures();
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
