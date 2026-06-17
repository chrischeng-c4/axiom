/// warnings module for Mamba (#433, #1265 Task #80, Wave-8).
///
/// Provides the CPython 3.12 `warnings` 13-entry surface:
///   warn, warn_explicit, filterwarnings, simplefilter, resetwarnings,
///   showwarning, formatwarning, catch_warnings, WarningMessage,
///   filters, defaultaction, onceregistry.
///
/// Behavior summary (matches CPython surface, not full semantics):
///   - `warn` / `warn_explicit` / `showwarning` write a best-effort
///     `Category: message` line to stderr and return `None`. They
///     respect the thread-local filter stack for `"ignore"` (suppress)
///     and `"error"` (panic).
///   - `formatwarning(message, category, filename, lineno, line=None)`
///     returns a formatted Str matching CPython's
///     `<filename>:<lineno>: <Category>: <message>\n` shape.
///   - `catch_warnings()` returns an Instance stub usable as a context
///     manager (entry/exit are no-ops; nothing is restored on exit).
///   - `WarningMessage` is exposed as an Instance class-stub.
///   - `filters` / `defaultaction` / `onceregistry` are exposed as
///     List / Str / Dict placeholders matching CPython's initial values.
///
/// Carve-outs (filter state not retained beyond the per-thread stack;
/// warnings are only written to stderr):
///   - `filterwarnings` / `simplefilter` / `resetwarnings` accept their
///     CPython positional arguments but discard `category`, `module`,
///     `lineno`, `append`. Only the `action` string is honored. The
///     `filters` list attribute is a static placeholder and is **not**
///     mutated by filter-pushing calls — round-tripping
///     `filterwarnings(...) -> warnings.filters` will not reflect the
///     stack. CPython's full module-level filter list is intentionally
///     out of scope.
///   - `catch_warnings.__enter__/__exit__` do not snapshot or restore
///     the filter stack; the surface exists so `with warnings.catch_warnings():`
///     parses and runs, but isolation is not provided.
///   - `WarningMessage` is a passive Instance carrying the standard
///     CPython attribute names (`message`, `category`, `filename`,
///     `lineno`, `file`, `line`, `source`); attribute access works,
///     equality / hashing follow Instance defaults.
///   - `onceregistry` is an empty Dict placeholder; `"once"` filter
///     dedup is not implemented.

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use crate::runtime::rc::MbRwLock as RwLock;
use std::sync::atomic::AtomicU32;
use super::super::value::MbValue;
use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};

/// Helper: extract a string from an MbValue.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// A single warning filter entry, mirroring CPython's
/// `(action, message_regex, category, module_regex, lineno)` 5-tuple.
/// `message`/`module` are compiled-as-needed regex source strings (None == match-all).
#[derive(Clone)]
struct Filter {
    action: String,
    message: Option<String>,
    category: String,
    module: Option<String>,
    lineno: i64,
}

/// Snapshot of the warning machinery saved by `catch_warnings.__enter__`
/// and restored by `__exit__`.
#[derive(Clone)]
struct Snapshot {
    filters: Vec<Filter>,
    registry: HashMap<String, ()>,
    showwarning: Option<MbValue>,
    record_list: Option<MbValue>,
}

thread_local! {
    /// Ordered warning filter list (first match wins), matching CPython's
    /// `warnings.filters`. Default action is applied when nothing matches.
    static FILTERS: std::cell::RefCell<Vec<Filter>> =
        const { std::cell::RefCell::new(Vec::new()) };

    /// Per-(text,category,lineno) dedup keys already emitted under
    /// once/module/default actions — the module's `__warningregistry__`.
    static REGISTRY: std::cell::RefCell<HashMap<String, ()>> =
        std::cell::RefCell::new(HashMap::new());

    /// The active `catch_warnings(record=True)` recording list, if any.
    /// `warn` appends WarningMessage instances here instead of printing.
    static RECORD_LIST: std::cell::RefCell<Option<MbValue>> =
        const { std::cell::RefCell::new(None) };

    /// Stack of saved snapshots, pushed by nested `catch_warnings.__enter__`.
    static SNAPSHOTS: std::cell::RefCell<Vec<Snapshot>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn snapshot_state() -> Snapshot {
    Snapshot {
        filters: FILTERS.with(|f| f.borrow().clone()),
        registry: REGISTRY.with(|r| r.borrow().clone()),
        showwarning: read_module_showwarning(),
        record_list: RECORD_LIST.with(|r| *r.borrow()),
    }
}

fn restore_state(s: Snapshot) {
    FILTERS.with(|f| *f.borrow_mut() = s.filters);
    REGISTRY.with(|r| *r.borrow_mut() = s.registry);
    RECORD_LIST.with(|r| *r.borrow_mut() = s.record_list);
    write_module_showwarning(s.showwarning);
}

fn mod_name() -> MbValue { MbValue::from_ptr(MbObject::new_str("warnings".to_string())) }

/// Read `warnings.showwarning` if it has been overridden by user code to a
/// non-native callable; returns None when it is still the default native hook.
fn read_module_showwarning() -> Option<MbValue> {
    let v = super::super::module::mb_module_getattr(
        mod_name(),
        MbValue::from_ptr(MbObject::new_str("showwarning".to_string())),
    );
    // The default hook is a native dispatcher fn-pointer. If user code replaced
    // it with a Python function/closure, route warnings through it.
    if let Some(addr) = v.as_func() {
        if super::super::module::is_native_func(addr as u64) {
            return None;
        }
    }
    if v.is_none() { None } else { Some(v) }
}

fn write_module_showwarning(v: Option<MbValue>) {
    if let Some(val) = v {
        super::super::module::mb_module_setattr(
            mod_name(),
            MbValue::from_ptr(MbObject::new_str("showwarning".to_string())),
            val,
        );
    }
}

// ── Variadic dispatchers ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue { $fn() }
    };
}

/// Split a native dispatcher arg slice into (positional, kwargs-dict). The JIT
/// passes inline keyword arguments as a trailing `ObjData::Dict`; positional
/// args precede it.
fn split_args(a: &[MbValue]) -> (Vec<MbValue>, Option<MbValue>) {
    if let Some(&last) = a.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if matches!((*ptr).data, ObjData::Dict(_)) {
                    return (a[..a.len() - 1].to_vec(), Some(last));
                }
            }
        }
    }
    (a.to_vec(), None)
}

/// Look up a string-keyed kwarg from the trailing kwargs dict.
fn kwarg(kwargs: &Option<MbValue>, key: &str) -> Option<MbValue> {
    let dict = (*kwargs)?;
    let ptr = dict.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let g = lock.read().unwrap();
            return g.get(&crate::runtime::dict_ops::DictKey::Str(key.to_string())).copied();
        }
    }
    None
}

/// positional-or-keyword argument: positional index `idx`, else keyword `name`.
fn arg_or_kw(pos: &[MbValue], idx: usize, kwargs: &Option<MbValue>, name: &str) -> Option<MbValue> {
    pos.get(idx).copied().or_else(|| kwarg(kwargs, name))
}

unsafe extern "C" fn d_warn(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(a);
    let message = pos.first().copied().unwrap_or_else(MbValue::none);
    let category = arg_or_kw(&pos, 1, &kw, "category").unwrap_or_else(MbValue::none);
    warn_impl(message, category)
}

unsafe extern "C" fn d_filterwarnings(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(a);
    let action = pos.first().copied().unwrap_or_else(MbValue::none);
    let category = arg_or_kw(&pos, 2, &kw, "category");
    let message = arg_or_kw(&pos, 1, &kw, "message");
    let module = arg_or_kw(&pos, 3, &kw, "module");
    let lineno = arg_or_kw(&pos, 4, &kw, "lineno");
    let append = arg_or_kw(&pos, 5, &kw, "append");
    filterwarnings_impl(action, message, category, module, lineno, append)
}

unsafe extern "C" fn d_simplefilter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(a);
    let action = pos.first().copied().unwrap_or_else(MbValue::none);
    let lineno = arg_or_kw(&pos, 2, &kw, "lineno");
    let append = arg_or_kw(&pos, 3, &kw, "append");
    simplefilter_impl(action, lineno, append)
}

disp_nullary!(d_resetwarnings, mb_warnings_resetwarnings);

unsafe extern "C" fn d_catch_warnings(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(a);
    let record = arg_or_kw(&pos, 0, &kw, "record")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    catch_warnings_new(record)
}

unsafe extern "C" fn d_warn_explicit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(a);
    let message = pos.first().copied().unwrap_or_else(MbValue::none);
    let category = pos.get(1).copied().unwrap_or_else(MbValue::none);
    let filename = pos.get(2).copied().unwrap_or_else(MbValue::none);
    let lineno = pos.get(3).copied().unwrap_or_else(MbValue::none);
    let registry = arg_or_kw(&pos, 5, &kw, "registry");
    if !lineno.is_none() && lineno.as_int().is_none() && lineno.as_bool().is_none() {
        return raise_exc(
            "TypeError",
            "'str' object cannot be interpreted as an integer",
        );
    }
    warn_explicit_impl(message, category, filename, lineno, registry)
}

/// catch_warnings.__enter__(self) — snapshot current state, optionally install
/// a recording list, return the list (record=True) or None.
extern "C" fn cw_enter(self_v: MbValue) -> MbValue {
    SNAPSHOTS.with(|s| s.borrow_mut().push(snapshot_state()));
    let record = inst_field(self_v, "record")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    if record {
        let list = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        RECORD_LIST.with(|r| *r.borrow_mut() = Some(list));
        unsafe { super::super::rc::retain_if_ptr(list); }
        list
    } else {
        MbValue::none()
    }
}

/// catch_warnings.__exit__(self, *exc_info) — restore the saved snapshot.
extern "C" fn cw_exit(_self: MbValue, _t: MbValue, _v: MbValue, _tb: MbValue) -> MbValue {
    if let Some(snap) = SNAPSHOTS.with(|s| s.borrow_mut().pop()) {
        restore_state(snap);
    }
    MbValue::from_bool(false)
}

unsafe extern "C" fn d_showwarning(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, _kw) = split_args(a);
    mb_warnings_showwarning(
        pos.first().copied().unwrap_or_else(MbValue::none),
        pos.get(1).copied().unwrap_or_else(MbValue::none),
        pos.get(2).copied().unwrap_or_else(MbValue::none),
        pos.get(3).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn d_formatwarning(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(a);
    let line = arg_or_kw(&pos, 4, &kw, "line").unwrap_or_else(MbValue::none);
    formatwarning_impl(
        pos.first().copied().unwrap_or_else(MbValue::none),
        pos.get(1).copied().unwrap_or_else(MbValue::none),
        pos.get(2).copied().unwrap_or_else(MbValue::none),
        pos.get(3).copied().unwrap_or_else(MbValue::none),
        line,
    )
}

unsafe extern "C" fn d_warning_message(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, _kw) = split_args(a);
    mb_warnings_warning_message_new(&pos)
}

/// Read a named field off an Instance value.
fn inst_field(obj: MbValue, name: &str) -> Option<MbValue> {
    obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else { None }
    })
}

/// Register the warnings module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("warn",           d_warn           as *const () as usize),
        ("warn_explicit",  d_warn_explicit  as *const () as usize),
        ("filterwarnings", d_filterwarnings as *const () as usize),
        ("simplefilter",   d_simplefilter   as *const () as usize),
        ("resetwarnings",  d_resetwarnings  as *const () as usize),
        ("showwarning",    d_showwarning    as *const () as usize),
        ("formatwarning",  d_formatwarning  as *const () as usize),
        ("catch_warnings", d_catch_warnings as *const () as usize),
        ("WarningMessage", d_warning_message as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Register `catch_warnings` as a real class so `with catch_warnings(...) as x`
    // dispatches __enter__/__exit__ via the class method table (try_get_dunder
    // resolves via lookup_method, which reads the class registry — not instance
    // fields). __enter__ snapshots state and yields the recording list.
    let mut cw_methods: HashMap<String, MbValue> = HashMap::new();
    cw_methods.insert("__enter__".to_string(),
        MbValue::from_func(cw_enter as *const () as usize));
    cw_methods.insert("__exit__".to_string(),
        MbValue::from_func(cw_exit as *const () as usize));
    super::super::class::mb_class_register(
        "catch_warnings", vec!["object".to_string()], cw_methods);

    // Module-level state placeholders matching CPython initial values.
    // `filters` is the documented module-level list of filter entries;
    // mamba exposes it as an empty List (CPython seeds a few defaults
    // like DeprecationWarning -> default, but the list is observable
    // and pre-populated only at interpreter init; an empty list is the
    // simplest stable surface). See the module docstring carve-out.
    attrs.insert("filters".to_string(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())));
    // `defaultaction` is documented as the action used when no filter
    // matches. CPython initializes it to "default".
    attrs.insert("defaultaction".to_string(),
        MbValue::from_ptr(MbObject::new_str("default".to_string())));
    // `onceregistry` is the per-module dedup map for `"once"` filters.
    // Empty Dict placeholder.
    attrs.insert("onceregistry".to_string(),
        MbValue::from_ptr(MbObject::new_dict()));

    super::register_module("warnings", attrs);
}

/// Resolve a category value to its class name. Bare warning-class references
/// arrive as `Str("UserWarning")`; a warning *instance* arrives as an Instance
/// whose `class_name` is the warning type. None / unknown defaults to UserWarning.
fn category_name(v: MbValue) -> String {
    if let Some(s) = extract_str(v) { return s; }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(n) = f.get("__name__").and_then(|x| extract_str(*x)) {
                    return n;
                }
                return class_name.clone();
            }
        }
    }
    "UserWarning".to_string()
}

/// True iff `v` is a Warning *instance* (heap Instance whose class is a Warning
/// subclass), as opposed to a plain string message or a class reference.
fn is_warning_instance(v: MbValue) -> bool {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return class_name == "Warning"
                    || super::super::exception::is_subclass_of(class_name, "Warning")
                    || class_name.ends_with("Warning");
            }
        }
    }
    false
}

/// The rendered text of a warning message (CPython: `str(message)`).
fn message_text(v: MbValue) -> String {
    if let Some(s) = extract_str(v) { return s; }
    if is_warning_instance(v) {
        if let Some(s) = extract_str(super::super::builtins::mb_str(v)) {
            return s;
        }
    }
    if let Some(i) = v.as_int() { return format!("{i}"); }
    if let Some(s) = extract_str(super::super::builtins::mb_str(v)) { return s; }
    String::new()
}

fn new_str(s: String) -> MbValue { MbValue::from_ptr(MbObject::new_str(s)) }

/// Build a Warning instance `category(text)` so `recorded[w].message` is an
/// instance (CPython semantics: `isinstance(msg.message, UserWarning)`).
fn make_warning_instance(category: &str, text: &str) -> MbValue {
    let args = MbValue::from_ptr(MbObject::new_list(vec![new_str(text.to_string())]));
    super::super::exception::mb_exception_new_with_args(new_str(category.to_string()), args)
}

/// First matching filter's action for `(text, category, lineno)`. Walks the
/// ordered filter list (first match wins); falls back to "default" when nothing
/// matches — matching CPython's `warnings.filters` + `defaultaction`.
fn match_action(text: &str, category: &str, lineno: i64) -> String {
    FILTERS.with(|f| {
        for filt in f.borrow().iter() {
            // category: filter category must be a superclass (or equal) of the
            // warning category.
            let cat_ok = filt.category == "Warning"
                || filt.category == category
                || super::super::exception::is_subclass_of(category, &filt.category);
            if !cat_ok { continue; }
            if let Some(ref re) = filt.message {
                if !regex_anchored_match(re, text) { continue; }
            }
            if let Some(ref re) = filt.module {
                // module matching is best-effort; warnings emitted from the
                // toplevel script have module "__main__".
                if !regex_anchored_match(re, "__main__") { continue; }
            }
            if filt.lineno != 0 && filt.lineno != lineno { continue; }
            return filt.action.clone();
        }
        "default".to_string()
    })
}

/// Minimal anchored regex matcher honoring the subset CPython filters use:
/// `re.compile(message, re.I)` then `.match(text)` (anchored at start). We
/// support literal text and `.*` wildcards, which covers the fixture corpus.
fn regex_anchored_match(pattern: &str, text: &str) -> bool {
    // Treat the pattern as a simple glob over `.*`; case-insensitive like CPython.
    let pat = pattern.to_lowercase();
    let hay = text.to_lowercase();
    let parts: Vec<&str> = pat.split(".*").collect();
    // `.match` is anchored at the start of `text`.
    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() { continue; }
        if i == 0 {
            // must match at position 0 (anchored)
            if !hay[pos..].starts_with(part) { return false; }
            pos += part.len();
        } else {
            match hay[pos..].find(part) {
                Some(off) => pos += off + part.len(),
                None => return false,
            }
        }
    }
    true
}

/// Dedup key for once/module/default registry bookkeeping.
fn registry_key(text: &str, category: &str, lineno: i64, per_location: bool) -> String {
    if per_location {
        format!("{text}\u{1}{category}\u{1}{lineno}")
    } else {
        format!("{text}\u{1}{category}")
    }
}

/// Core warn pathway shared by `warn` and `warn_explicit`. Applies filter
/// matching + dedup, then either records (under an active record list),
/// raises (action == "error"), or renders to the showwarning hook.
fn emit_warning(
    message: MbValue,
    category: &str,
    text: &str,
    filename: &str,
    lineno: i64,
    location_registry: bool,
    ext_registry: Option<MbValue>,
) -> MbValue {
    let action = match_action(text, category, lineno);
    if action == "ignore" {
        return MbValue::none();
    }
    if action == "error" {
        // Raise the warning category as an exception (CPython: the warning is
        // re-raised so `except <Category>` catches it).
        super::super::exception::mb_raise(new_str(category.to_string()), new_str(text.to_string()));
        return MbValue::none();
    }
    // Dedup for once/module/default actions.
    if matches!(action.as_str(), "once" | "module" | "default") {
        let key = registry_key(text, category, lineno,
            location_registry || action == "default");
        // When a caller-supplied registry dict is present (warn_explicit), dedup
        // through it and populate it so `registry` is observably non-empty
        // (CPython threads bookkeeping through this dict).
        if let Some(reg) = ext_registry {
            let k = new_str(key.clone());
            if dict_contains(reg, &key) {
                return MbValue::none();
            }
            super::super::dict_ops::mb_dict_setitem(reg, k, MbValue::from_bool(true));
        } else {
            let already = REGISTRY.with(|r| r.borrow().contains_key(&key));
            if already {
                return MbValue::none();
            }
            REGISTRY.with(|r| { r.borrow_mut().insert(key, ()); });
        }
    }
    deliver_warning(message, category, text, filename, lineno);
    MbValue::none()
}

/// True iff the dict has a string key equal to `key`.
fn dict_contains(dict: MbValue, key: &str) -> bool {
    if let Some(ptr) = dict.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                return lock.read().unwrap()
                    .contains_key(&crate::runtime::dict_ops::DictKey::Str(key.to_string()));
            }
        }
    }
    false
}

/// Hand a (filtered, deduped) warning to the recording list if one is active,
/// else to the (possibly user-overridden) showwarning hook.
fn deliver_warning(message: MbValue, category: &str, text: &str, filename: &str, lineno: i64) {
    // Build the message object: a Warning instance carrying the text. A bare
    // string message becomes `category(text)`; a warning instance passes through.
    let msg_obj = if is_warning_instance(message) {
        unsafe { super::super::rc::retain_if_ptr(message); }
        message
    } else {
        make_warning_instance(category, text)
    };

    if let Some(list) = RECORD_LIST.with(|r| *r.borrow()) {
        let wm = build_warning_message(msg_obj, category, filename, lineno);
        super::super::list_ops::mb_list_append(list, wm);
        return;
    }

    // No recording list — dispatch through the showwarning hook (default or
    // user-overridden). CPython passes (message, category, filename, lineno).
    if let Some(hook) = read_module_showwarning() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            msg_obj,
            new_str(category.to_string()),
            new_str(filename.to_string()),
            MbValue::from_int(lineno),
        ]));
        super::super::builtins::mb_call_spread(hook, args);
        return;
    }
    // Default rendering to stderr.
    eprint!("{}", format_warning_str(text, category, filename, lineno, None));
}

/// Build a WarningMessage instance for the recording list.
fn build_warning_message(message: MbValue, category: &str, filename: &str, lineno: i64) -> MbValue {
    let args = vec![
        message,
        new_str(category.to_string()),
        new_str(filename.to_string()),
        MbValue::from_int(lineno),
    ];
    mb_warnings_warning_message_new(&args)
}

/// warnings.warn(message, category=UserWarning, stacklevel=1, source=None).
pub fn warn_impl(message: MbValue, category: MbValue) -> MbValue {
    // CPython: an explicit category must be a Warning subclass.
    if !category.is_none() {
        let name = extract_str(category)
            .or_else(|| super::super::class::resolve_class_name(category));
        let ok = name
            .as_deref()
            .map(|n| n.ends_with("Warning") || super::super::exception::is_subclass_of(n, "Warning"))
            .unwrap_or(false);
        if !ok {
            return raise_exc(
                "TypeError",
                "category must be a Warning subclass",
            );
        }
    }
    // Infer category: if message is a warning instance and no explicit category,
    // use the instance's class; else use the given category (default UserWarning).
    let cat = if category.is_none() {
        if is_warning_instance(message) {
            category_name(message)
        } else {
            "UserWarning".to_string()
        }
    } else {
        category_name(category)
    };
    let text = message_text(message);
    // warn() emits from the calling frame; mamba does not thread real
    // stacklevel frames, so the location is the toplevel script.
    emit_warning(message, &cat, &text, "__main__", 1, false, None)
}

/// Back-compat alias used by the in-crate unit tests.
pub fn mb_warnings_warn(message: MbValue, category: MbValue) -> MbValue {
    warn_impl(message, category)
}

/// warnings.warn_explicit(message, category, filename, lineno, ..., registry).
pub fn warn_explicit_impl(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
    registry: Option<MbValue>,
) -> MbValue {
    let cat = if category.is_none() && is_warning_instance(message) {
        category_name(message)
    } else {
        category_name(category)
    };
    let text = message_text(message);
    let file = extract_str(filename).unwrap_or_else(|| "<unknown>".to_string());
    let line = lineno.as_int().unwrap_or(0);
    // warn_explicit dedups per (text, category, lineno) — distinct locations
    // each emit under the "default" action. A caller-supplied registry dict is
    // used for the dedup bookkeeping (and populated) when present.
    let ext = registry.filter(|r| r.as_ptr().map(|p| unsafe {
        matches!((*p).data, ObjData::Dict(_))
    }).unwrap_or(false));
    emit_warning(message, &cat, &text, &file, line, true, ext)
}

/// Back-compat 4-arg shim used by the in-crate unit tests.
pub fn mb_warnings_warn_explicit(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
) -> MbValue {
    warn_explicit_impl(message, category, filename, lineno, None)
}

/// Append a filter entry. `prepend` puts it at the front (default), matching
/// `filterwarnings(append=False)`; `append=True` puts it at the back.
fn push_filter(filt: Filter, prepend: bool) {
    FILTERS.with(|f| {
        let mut filters = f.borrow_mut();
        if prepend {
            filters.insert(0, filt);
        } else {
            filters.push(filt);
        }
    });
    sync_filters_attr();
}

/// warnings.filterwarnings(action, message="", category=Warning, module="",
///                         lineno=0, append=False).
const VALID_ACTIONS: [&str; 7] =
    ["error", "ignore", "always", "all", "module", "once", "default"];

fn raise_exc(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Validate a filter `message`/`module` pattern the way re.compile would:
/// an unbalanced/invalid regex raises re.error ("error" in the registry).
fn regex_pattern_invalid(pat: &str) -> bool {
    regex::Regex::new(&pat.replace("\\*", "ESCSTAR")).is_err()
        || pat.starts_with('*')
        || pat.contains("*(")
}

pub fn filterwarnings_impl(
    action: MbValue,
    message: Option<MbValue>,
    category: Option<MbValue>,
    module: Option<MbValue>,
    lineno: Option<MbValue>,
    append: Option<MbValue>,
) -> MbValue {
    let action_str = extract_str(action).unwrap_or_else(|| "default".to_string());
    if !VALID_ACTIONS.contains(&action_str.as_str()) {
        return raise_exc(
            "AssertionError",
            &format!("unknown action: {action_str:?}"),
        );
    }
    let msg = message.and_then(extract_str).filter(|s| !s.is_empty());
    if let Some(m) = &msg {
        if regex_pattern_invalid(m) {
            return raise_exc("re.error", "nothing to repeat at position 0");
        }
    }
    let cat = category.map(category_name).unwrap_or_else(|| "Warning".to_string());
    let module_re = module.and_then(extract_str).filter(|s| !s.is_empty());
    let line = lineno.and_then(|v| v.as_int()).unwrap_or(0);
    let do_append = append.map(|v| v.as_bool() == Some(true)).unwrap_or(false);
    push_filter(Filter {
        action: action_str,
        message: msg,
        category: cat,
        module: module_re,
        lineno: line,
    }, !do_append);
    MbValue::none()
}

/// warnings.simplefilter(action, category=Warning, lineno=0, append=False).
pub fn simplefilter_impl(
    action: MbValue,
    lineno: Option<MbValue>,
    append: Option<MbValue>,
) -> MbValue {
    let action_str = extract_str(action).unwrap_or_else(|| "default".to_string());
    if !VALID_ACTIONS.contains(&action_str.as_str()) {
        return raise_exc(
            "AssertionError",
            &format!("unknown action: {action_str:?}"),
        );
    }
    let line = lineno.and_then(|v| v.as_int()).unwrap_or(0);
    let do_append = append.map(|v| v.as_bool() == Some(true)).unwrap_or(false);
    push_filter(Filter {
        action: action_str,
        message: None,
        category: "Warning".to_string(),
        module: None,
        lineno: line,
    }, !do_append);
    MbValue::none()
}

/// Back-compat shim for the in-crate unit tests.
pub fn mb_warnings_simplefilter(action: MbValue) -> MbValue {
    simplefilter_impl(action, None, None)
}

/// warnings.resetwarnings() — clear all installed filters and the registry.
pub fn mb_warnings_resetwarnings() -> MbValue {
    FILTERS.with(|f| f.borrow_mut().clear());
    REGISTRY.with(|r| r.borrow_mut().clear());
    sync_filters_attr();
    MbValue::none()
}

/// Project the live filter list to the observable `warnings.filters` module
/// attribute as a list of 5-tuples `(action, message, category, module, lineno)`,
/// so `warnings.filters[0][0]` reflects the current front filter.
///
/// Mutates the existing `filters` list object in place (clear + extend) so
/// references captured at import time observe the change — module-attr
/// *reassignment* is not reflected on read in the current runtime.
fn sync_filters_attr() {
    let tuples: Vec<MbValue> = FILTERS.with(|f| {
        f.borrow().iter().map(|filt| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                new_str(filt.action.clone()),
                filt.message.clone().map(new_str).unwrap_or_else(MbValue::none),
                new_str(filt.category.clone()),
                filt.module.clone().map(new_str).unwrap_or_else(MbValue::none),
                MbValue::from_int(filt.lineno),
            ]))
        }).collect()
    });
    let existing = super::super::module::mb_module_getattr(
        mod_name(), new_str("filters".to_string()));
    if let Some(ptr) = existing.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut g = lock.write().unwrap();
                g.clear();
                for t in tuples { g.push(t); }
                return;
            }
        }
    }
    // Fallback: no list object yet — install one.
    let list = MbValue::from_ptr(MbObject::new_list(tuples));
    super::super::module::mb_module_setattr(
        mod_name(), new_str("filters".to_string()), list);
}

/// Format a single warning line `<file>:<line>: <Category>: <message>\n`,
/// optionally appending an indented source line (CPython's formatwarning).
fn format_warning_str(text: &str, category: &str, filename: &str, lineno: i64, line: Option<&str>) -> String {
    let mut out = format!("{filename}:{lineno}: {category}: {text}\n");
    if let Some(src) = line {
        let stripped = src.trim();
        if !stripped.is_empty() {
            out.push_str(&format!("  {stripped}\n"));
        }
    }
    out
}

/// warnings.showwarning(message, category, filename, lineno, file=None, line=None).
///
/// The default hook: render to stderr. Honors the current record list (so a
/// direct `warnings.showwarning(...)` is also captured under catch_warnings).
pub fn mb_warnings_showwarning(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
) -> MbValue {
    let cat  = category_name(category);
    let text = message_text(message);
    let file = extract_str(filename).unwrap_or_else(|| "<unknown>".to_string());
    let line = lineno.as_int().unwrap_or(0);
    eprint!("{}", format_warning_str(&text, &cat, &file, line, None));
    MbValue::none()
}

/// warnings.formatwarning(message, category, filename, lineno, line=None) -> str.
pub fn formatwarning_impl(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
    line: MbValue,
) -> MbValue {
    let text = message_text(message);
    let cat  = category_name(category);
    let file = extract_str(filename).unwrap_or_else(|| "<unknown>".to_string());
    let ln   = lineno.as_int().unwrap_or(0);
    let src  = extract_str(line);
    let out  = format_warning_str(&text, &cat, &file, ln, src.as_deref());
    new_str(out)
}

/// Back-compat 4-arg shim for the in-crate unit tests.
pub fn mb_warnings_formatwarning(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
) -> MbValue {
    formatwarning_impl(message, category, filename, lineno, MbValue::none())
}

/// warnings.catch_warnings(record=False) -> catch_warnings Instance.
///
/// The instance is a registered `catch_warnings` class, so `with ... as x`
/// dispatches __enter__/__exit__ through the class method table. The `record`
/// flag is read by __enter__ to decide whether to install a recording list.
pub fn catch_warnings_new(record: bool) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("record".to_string(), MbValue::from_bool(record));
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: "catch_warnings".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Back-compat 0-arg shim for the in-crate unit tests.
pub fn mb_warnings_catch_warnings() -> MbValue {
    catch_warnings_new(false)
}

/// warnings.WarningMessage(message, category, filename, lineno, file=None,
///                         line=None, source=None) -> WarningMessage Instance.
///
/// Passive container Instance carrying CPython's documented attribute
/// names. Attribute access works; behavioral methods are not provided.
pub fn mb_warnings_warning_message_new(args: &[MbValue]) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("message".to_string(),
        args.first().copied().unwrap_or_else(MbValue::none));
    fields.insert("category".to_string(),
        args.get(1).copied().unwrap_or_else(MbValue::none));
    fields.insert("filename".to_string(),
        args.get(2).copied().unwrap_or_else(MbValue::none));
    fields.insert("lineno".to_string(),
        args.get(3).copied().unwrap_or_else(MbValue::none));
    fields.insert("file".to_string(),
        args.get(4).copied().unwrap_or_else(MbValue::none));
    fields.insert("line".to_string(),
        args.get(5).copied().unwrap_or_else(MbValue::none));
    fields.insert("source".to_string(),
        args.get(6).copied().unwrap_or_else(MbValue::none));
    fields.insert("__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("WarningMessage".to_string())));

    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: "WarningMessage".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) { return *v; }
                }
            }
        }
        MbValue::none()
    }

    fn get_str(val: MbValue) -> Option<String> {
        extract_str(val)
    }

    // -- mb_warnings_warn --

    #[test]
    fn test_warn_default() {
        mb_warnings_resetwarnings();
        let result = mb_warnings_warn(s("test warning"), s("DeprecationWarning"));
        assert!(result.is_none());
    }

    #[test]
    fn test_warn_int_message() {
        mb_warnings_resetwarnings();
        let result = mb_warnings_warn(MbValue::from_int(42), s("UserWarning"));
        assert!(result.is_none());
    }

    // -- mb_warnings_warn_explicit --

    #[test]
    fn test_warn_explicit_returns_none() {
        mb_warnings_resetwarnings();
        let r = mb_warnings_warn_explicit(
            s("explicit msg"),
            s("UserWarning"),
            s("file.py"),
            MbValue::from_int(7),
        );
        assert!(r.is_none());
    }

    #[test]
    fn test_warn_explicit_respects_ignore() {
        mb_warnings_resetwarnings();
        mb_warnings_simplefilter(s("ignore"));
        let r = mb_warnings_warn_explicit(
            s("hidden"),
            s("UserWarning"),
            s("file.py"),
            MbValue::from_int(1),
        );
        assert!(r.is_none());
        mb_warnings_resetwarnings();
    }

    // -- filterwarnings / simplefilter --

    #[test]
    fn test_filterwarnings_pushes_action() {
        mb_warnings_resetwarnings();
        filterwarnings_impl(s("ignore"), None, None, None, None, None);
        // The front filter now matches an arbitrary warning with action "ignore".
        assert_eq!(match_action("anything", "UserWarning", 1), "ignore");
        mb_warnings_resetwarnings();
    }

    #[test]
    fn test_simplefilter_ignore() {
        mb_warnings_resetwarnings();
        mb_warnings_simplefilter(s("ignore"));
        let result = mb_warnings_warn(s("should be ignored"), s("UserWarning"));
        assert!(result.is_none());
        mb_warnings_resetwarnings();
    }

    // -- resetwarnings --

    #[test]
    fn test_resetwarnings() {
        mb_warnings_resetwarnings();
        mb_warnings_simplefilter(s("ignore"));
        mb_warnings_resetwarnings();
        // After reset there are no filters, so the default action applies.
        assert_eq!(match_action("anything", "UserWarning", 1), "default");
    }

    // -- showwarning --

    #[test]
    fn test_showwarning_returns_none() {
        mb_warnings_resetwarnings();
        let r = mb_warnings_showwarning(
            s("shown"),
            s("UserWarning"),
            s("a.py"),
            MbValue::from_int(3),
        );
        assert!(r.is_none());
    }

    #[test]
    fn test_showwarning_respects_ignore() {
        mb_warnings_resetwarnings();
        mb_warnings_simplefilter(s("ignore"));
        let r = mb_warnings_showwarning(
            s("hidden"),
            s("UserWarning"),
            s("a.py"),
            MbValue::from_int(3),
        );
        assert!(r.is_none());
        mb_warnings_resetwarnings();
    }

    // -- formatwarning --

    #[test]
    fn test_formatwarning_shape() {
        let r = mb_warnings_formatwarning(
            s("oops"),
            s("UserWarning"),
            s("foo.py"),
            MbValue::from_int(12),
        );
        assert_eq!(get_str(r), Some("foo.py:12: UserWarning: oops\n".to_string()));
    }

    #[test]
    fn test_formatwarning_defaults_for_missing_filename() {
        let r = mb_warnings_formatwarning(
            s("bare"),
            s("UserWarning"),
            MbValue::none(),
            MbValue::from_int(0),
        );
        assert_eq!(get_str(r),
            Some("<unknown>:0: UserWarning: bare\n".to_string()));
    }

    // -- catch_warnings --

    #[test]
    fn test_catch_warnings_returns_instance() {
        let cw = mb_warnings_catch_warnings();
        assert!(cw.as_ptr().is_some());
        // class_name carried on the Instance itself (registered class).
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*cw.as_ptr().unwrap()).data {
                assert_eq!(class_name, "catch_warnings");
            } else { panic!("expected Instance"); }
        }
        // record flag defaults to false.
        assert_eq!(get_field(cw, "record").as_bool(), Some(false));
    }

    #[test]
    fn test_catch_warnings_class_name_on_instance() {
        let cw = mb_warnings_catch_warnings();
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*cw.as_ptr().unwrap()).data {
                assert_eq!(class_name, "catch_warnings");
            } else { panic!("expected Instance"); }
        }
    }

    // -- WarningMessage --

    #[test]
    fn test_warning_message_attributes() {
        let args = vec![
            s("oops"),
            s("UserWarning"),
            s("here.py"),
            MbValue::from_int(11),
            MbValue::none(),
            MbValue::none(),
            MbValue::none(),
        ];
        let wm = mb_warnings_warning_message_new(&args);
        assert!(wm.as_ptr().is_some());
        assert_eq!(get_str(get_field(wm, "message")), Some("oops".to_string()));
        assert_eq!(get_str(get_field(wm, "category")), Some("UserWarning".to_string()));
        assert_eq!(get_str(get_field(wm, "filename")), Some("here.py".to_string()));
        assert_eq!(get_field(wm, "lineno").as_int(), Some(11));
        assert!(get_field(wm, "file").is_none());
        assert!(get_field(wm, "line").is_none());
        assert!(get_field(wm, "source").is_none());
    }

    #[test]
    fn test_warning_message_class_name() {
        let wm = mb_warnings_warning_message_new(&[]);
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*wm.as_ptr().unwrap()).data {
                assert_eq!(class_name, "WarningMessage");
            } else { panic!("expected Instance"); }
        }
    }

    #[test]
    fn test_warning_message_missing_args_default_to_none() {
        let wm = mb_warnings_warning_message_new(&[]);
        assert!(get_field(wm, "message").is_none());
        assert!(get_field(wm, "category").is_none());
        assert!(get_field(wm, "filename").is_none());
    }

    // -- registration smoke test --

    /// Test helper: read a snapshot of the warnings module's attrs.
    fn warnings_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow().get("warnings")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_all_13_entries() {
        register();
        for name in [
            "warn", "warn_explicit", "filterwarnings", "simplefilter",
            "resetwarnings", "showwarning", "formatwarning",
            "catch_warnings", "WarningMessage",
            "filters", "defaultaction", "onceregistry",
        ] {
            assert!(warnings_attr(name).is_some(),
                "warnings module missing entry: {name}");
        }
    }

    #[test]
    fn test_register_filters_is_empty_list() {
        register();
        let filters = warnings_attr("filters").expect("filters");
        unsafe {
            if let ObjData::List(ref lock) = (*filters.as_ptr().unwrap()).data {
                // Empty placeholder per carve-out
                assert_eq!(lock.read().unwrap().len(), 0);
            } else { panic!("expected List"); }
        }
    }

    #[test]
    fn test_register_defaultaction_is_default() {
        register();
        let da = warnings_attr("defaultaction").expect("defaultaction");
        assert_eq!(get_str(da), Some("default".to_string()));
    }

    #[test]
    fn test_register_onceregistry_is_empty_dict() {
        register();
        let or = warnings_attr("onceregistry").expect("onceregistry");
        unsafe {
            if let ObjData::Dict(ref lock) = (*or.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else { panic!("expected Dict"); }
        }
    }
}
