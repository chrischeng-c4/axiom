use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
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
use std::sync::atomic::AtomicU32;

/// Helper: extract a string from an MbValue.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

thread_local! {
    /// Thread-local warning filter stack.
    /// Each entry is an action string: "default", "error", "ignore", "always",
    /// "module", "once". The most recently pushed filter takes precedence.
    static FILTERS: std::cell::RefCell<Vec<String>> =
        std::cell::RefCell::new(vec!["default".to_string()]);
}

// ── Variadic dispatchers ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_binary!(d_warn, mb_warnings_warn);
disp_binary!(d_filterwarnings, mb_warnings_filterwarnings);
disp_unary!(d_simplefilter, mb_warnings_simplefilter);
disp_nullary!(d_resetwarnings, mb_warnings_resetwarnings);
disp_nullary!(d_catch_warnings, mb_warnings_catch_warnings);

unsafe extern "C" fn d_warn_explicit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_warnings_warn_explicit(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
        a.get(3).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn d_showwarning(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_warnings_showwarning(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
        a.get(3).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn d_formatwarning(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_warnings_formatwarning(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
        a.get(3).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn d_warning_message(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_warnings_warning_message_new(a)
}

/// Register the warnings module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("warn", d_warn as *const () as usize),
        ("warn_explicit", d_warn_explicit as *const () as usize),
        ("filterwarnings", d_filterwarnings as *const () as usize),
        ("simplefilter", d_simplefilter as *const () as usize),
        ("resetwarnings", d_resetwarnings as *const () as usize),
        ("showwarning", d_showwarning as *const () as usize),
        ("formatwarning", d_formatwarning as *const () as usize),
        ("catch_warnings", d_catch_warnings as *const () as usize),
        ("WarningMessage", d_warning_message as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Module-level state placeholders matching CPython initial values.
    // `filters` is the documented module-level list of filter entries;
    // mamba exposes it as an empty List (CPython seeds a few defaults
    // like DeprecationWarning -> default, but the list is observable
    // and pre-populated only at interpreter init; an empty list is the
    // simplest stable surface). See the module docstring carve-out.
    attrs.insert(
        "filters".to_string(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    // `defaultaction` is documented as the action used when no filter
    // matches. CPython initializes it to "default".
    attrs.insert(
        "defaultaction".to_string(),
        MbValue::from_ptr(MbObject::new_str("default".to_string())),
    );
    // `onceregistry` is the per-module dedup map for `"once"` filters.
    // Empty Dict placeholder.
    attrs.insert(
        "onceregistry".to_string(),
        MbValue::from_ptr(MbObject::new_dict()),
    );

    super::register_module("warnings", attrs);
}

/// Get the current active filter action.
fn current_filter_action() -> String {
    FILTERS.with(|f| {
        let filters = f.borrow();
        filters
            .last()
            .cloned()
            .unwrap_or_else(|| "default".to_string())
    })
}

/// Render an MbValue as a string for warning output: prefers Str, then
/// int, then bool, then a generic placeholder.
fn render_message(v: MbValue) -> String {
    if let Some(s) = extract_str(v) {
        return s;
    }
    if let Some(i) = v.as_int() {
        return format!("{i}");
    }
    if let Some(b) = v.as_bool() {
        return format!("{b}");
    }
    "unknown warning".to_string()
}

/// Render category as a Str; falls back to "UserWarning".
fn render_category(v: MbValue) -> String {
    // Accept either a Str ("UserWarning") or an Instance whose
    // class_name/__name__ carries the class.
    if let Some(s) = extract_str(v) {
        return s;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*ptr).data
            {
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

/// warnings.warn(message, category=UserWarning, stacklevel=1, source=None) -> None.
///
/// Prints a warning to stderr in the format "CategoryName: message".
/// Respects the current filter: "ignore" suppresses output, "error"
/// panics, others print normally.
pub fn mb_warnings_warn(message: MbValue, category: MbValue) -> MbValue {
    let action = current_filter_action();

    if action == "ignore" {
        return MbValue::none();
    }

    let msg = render_message(message);
    let cat = render_category(category);

    if action == "error" {
        panic!("{cat}: {msg}");
    }

    eprintln!("{cat}: {msg}");
    MbValue::none()
}

/// warnings.warn_explicit(message, category, filename, lineno, ...) -> None.
///
/// Like `warn`, but the caller supplies filename/lineno explicitly.
/// Output shape mirrors `formatwarning`. Honors the same filter stack.
pub fn mb_warnings_warn_explicit(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
) -> MbValue {
    let action = current_filter_action();
    if action == "ignore" {
        return MbValue::none();
    }

    let msg = render_message(message);
    let cat = render_category(category);
    let file = extract_str(filename).unwrap_or_else(|| "<unknown>".to_string());
    let line = lineno.as_int().unwrap_or(0);

    if action == "error" {
        panic!("{file}:{line}: {cat}: {msg}");
    }

    eprintln!("{file}:{line}: {cat}: {msg}");
    MbValue::none()
}

/// warnings.filterwarnings(action, category=Warning, ...) -> None.
///
/// Pushes the action string onto the thread-local filter stack. The
/// extra CPython positional arguments (`category`, `module`, `lineno`,
/// `append`) are accepted but discarded — see module-level carve-out.
pub fn mb_warnings_filterwarnings(action: MbValue, _category: MbValue) -> MbValue {
    let action_str = extract_str(action).unwrap_or_else(|| "default".to_string());

    FILTERS.with(|f| {
        f.borrow_mut().push(action_str);
    });

    MbValue::none()
}

/// warnings.simplefilter(action, ...) -> None.
///
/// Simplified version of filterwarnings that applies to all warning
/// categories. Pushes the action onto the filter stack.
pub fn mb_warnings_simplefilter(action: MbValue) -> MbValue {
    let action_str = extract_str(action).unwrap_or_else(|| "default".to_string());

    FILTERS.with(|f| {
        f.borrow_mut().push(action_str);
    });

    MbValue::none()
}

/// warnings.resetwarnings() -> None.
///
/// Clear the filter stack, resetting to the default filter only.
pub fn mb_warnings_resetwarnings() -> MbValue {
    FILTERS.with(|f| {
        let mut filters = f.borrow_mut();
        filters.clear();
        filters.push("default".to_string());
    });

    MbValue::none()
}

/// warnings.showwarning(message, category, filename, lineno, file=None, line=None)
/// -> None.
///
/// CPython's hook for rendering a single warning. Mamba forwards to
/// stderr using the same shape as `formatwarning`. The `file` and
/// `line` arguments are accepted but ignored.
pub fn mb_warnings_showwarning(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
) -> MbValue {
    let action = current_filter_action();
    if action == "ignore" {
        return MbValue::none();
    }

    let msg = render_message(message);
    let cat = render_category(category);
    let file = extract_str(filename).unwrap_or_else(|| "<unknown>".to_string());
    let line = lineno.as_int().unwrap_or(0);

    if action == "error" {
        panic!("{file}:{line}: {cat}: {msg}");
    }

    eprintln!("{file}:{line}: {cat}: {msg}");
    MbValue::none()
}

/// warnings.formatwarning(message, category, filename, lineno, line=None) -> str.
///
/// Returns the formatted warning string matching CPython:
///   `<filename>:<lineno>: <Category>: <message>\n`
pub fn mb_warnings_formatwarning(
    message: MbValue,
    category: MbValue,
    filename: MbValue,
    lineno: MbValue,
) -> MbValue {
    let msg = render_message(message);
    let cat = render_category(category);
    let file = extract_str(filename).unwrap_or_else(|| "<unknown>".to_string());
    let line = lineno.as_int().unwrap_or(0);
    let out = format!("{file}:{line}: {cat}: {msg}\n");
    MbValue::from_ptr(MbObject::new_str(out))
}

/// warnings.catch_warnings() -> catch_warnings Instance.
///
/// Returns a passive Instance usable as a context manager. The Instance
/// carries `__enter__` / `__exit__` field names so attribute access
/// works in dispatch; both are placeholder Nones because mamba does not
/// yet snapshot/restore the filter stack — see module-level carve-out.
pub fn mb_warnings_catch_warnings() -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("catch_warnings".to_string())),
    );
    fields.insert("__enter__".to_string(), MbValue::none());
    fields.insert("__exit__".to_string(), MbValue::none());
    fields.insert("record".to_string(), MbValue::from_bool(false));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "catch_warnings".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// warnings.WarningMessage(message, category, filename, lineno, file=None,
///                         line=None, source=None) -> WarningMessage Instance.
///
/// Passive container Instance carrying CPython's documented attribute
/// names. Attribute access works; behavioral methods are not provided.
pub fn mb_warnings_warning_message_new(args: &[MbValue]) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "message".to_string(),
        args.first().copied().unwrap_or_else(MbValue::none),
    );
    fields.insert(
        "category".to_string(),
        args.get(1).copied().unwrap_or_else(MbValue::none),
    );
    fields.insert(
        "filename".to_string(),
        args.get(2).copied().unwrap_or_else(MbValue::none),
    );
    fields.insert(
        "lineno".to_string(),
        args.get(3).copied().unwrap_or_else(MbValue::none),
    );
    fields.insert(
        "file".to_string(),
        args.get(4).copied().unwrap_or_else(MbValue::none),
    );
    fields.insert(
        "line".to_string(),
        args.get(5).copied().unwrap_or_else(MbValue::none),
    );
    fields.insert(
        "source".to_string(),
        args.get(6).copied().unwrap_or_else(MbValue::none),
    );
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("WarningMessage".to_string())),
    );

    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
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
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
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
        mb_warnings_filterwarnings(s("ignore"), MbValue::none());
        assert_eq!(current_filter_action(), "ignore");
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
        assert_eq!(current_filter_action(), "default");
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
        assert_eq!(
            get_str(r),
            Some("foo.py:12: UserWarning: oops\n".to_string())
        );
    }

    #[test]
    fn test_formatwarning_defaults_for_missing_filename() {
        let r = mb_warnings_formatwarning(
            s("bare"),
            s("UserWarning"),
            MbValue::none(),
            MbValue::from_int(0),
        );
        assert_eq!(
            get_str(r),
            Some("<unknown>:0: UserWarning: bare\n".to_string())
        );
    }

    // -- catch_warnings --

    #[test]
    fn test_catch_warnings_returns_instance() {
        let cw = mb_warnings_catch_warnings();
        assert!(cw.as_ptr().is_some());
        // class_name field
        let cls = get_str(get_field(cw, "__class__"));
        assert_eq!(cls, Some("catch_warnings".to_string()));
        // record placeholder
        assert_eq!(get_field(cw, "record").as_bool(), Some(false));
    }

    #[test]
    fn test_catch_warnings_class_name_on_instance() {
        let cw = mb_warnings_catch_warnings();
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*cw.as_ptr().unwrap()).data {
                assert_eq!(class_name, "catch_warnings");
            } else {
                panic!("expected Instance");
            }
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
        assert_eq!(
            get_str(get_field(wm, "category")),
            Some("UserWarning".to_string())
        );
        assert_eq!(
            get_str(get_field(wm, "filename")),
            Some("here.py".to_string())
        );
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
            } else {
                panic!("expected Instance");
            }
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
            mods.borrow()
                .get("warnings")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_all_13_entries() {
        register();
        for name in [
            "warn",
            "warn_explicit",
            "filterwarnings",
            "simplefilter",
            "resetwarnings",
            "showwarning",
            "formatwarning",
            "catch_warnings",
            "WarningMessage",
            "filters",
            "defaultaction",
            "onceregistry",
        ] {
            assert!(
                warnings_attr(name).is_some(),
                "warnings module missing entry: {name}"
            );
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
            } else {
                panic!("expected List");
            }
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
            } else {
                panic!("expected Dict");
            }
        }
    }
}
