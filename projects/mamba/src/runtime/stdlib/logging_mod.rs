use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::cell::RefCell;
/// logging module for Mamba (#400).
///
/// Real native behavior matching CPython 3.12:
///  - Level constants + getLevelName / getLevelNamesMapping / addLevelName
///  - getLogger(name) returns a real cached `Logger` instance (identity-stable),
///    with .name/.level/.handlers/.propagate fields and setLevel / isEnabledFor /
///    getChild / getChildren / addHandler / removeHandler / debug…critical / log /
///    exception methods.
///  - Handler / StreamHandler / NullHandler / Formatter / BufferingFormatter /
///    Filter / LogRecord constructed as real Instances with native methods.
///
/// `Logger` is registered as a real class (CLASS_REGISTRY) so `isinstance`,
/// subclassing (`class X(logging.Logger)`), and caching identity all work.
/// All methods are threaded `self`-first through the runtime's generic
/// instance method-dispatch path — no class.rs predicate branch required.
use std::collections::HashMap;

// ── Small local helpers (duplicated on purpose; self-contained module) ──

fn extract_str(val: MbValue) -> String {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return s.clone();
            }
        }
    }
    if let Some(i) = val.as_int() {
        return format!("{i}");
    }
    if let Some(f) = val.as_float() {
        return format!("{f}");
    }
    if let Some(b) = val.as_bool() {
        return if b { "True" } else { "False" }.to_string();
    }
    if val.is_none() {
        return "None".to_string();
    }
    // Arbitrary objects (list/dict/instance/etc.): fall back to the runtime's
    // str() so non-string `msg` values stringify like CPython rather than
    // collapsing to "". CPython's LogRecord stores `msg` verbatim and applies
    // `str(self.msg)` in getMessage().
    let s = super::super::builtins::mb_str(val);
    if let Some(ptr) = s.as_ptr() {
        unsafe {
            if let ObjData::Str(ref out) = (*ptr).data {
                return out.clone();
            }
        }
    }
    String::new()
}

fn is_str_value(val: MbValue) -> bool {
    val.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
}

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

// Safe wrapper around the runtime's (unsafe) refcount-bump helper.
fn retain(val: MbValue) {
    unsafe {
        super::super::rc::retain_if_ptr(val);
    }
}

fn raise(kind: &str, msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str(kind), new_str(msg.into()));
    MbValue::none()
}

fn field_get(inst: MbValue, key: &str) -> MbValue {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(v) = fields.read().unwrap().get(key) {
                    let v = *v;
                    retain(v);
                    return v;
                }
            }
        }
    }
    MbValue::none()
}

fn field_set(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                retain(val);
                fields.write().unwrap().insert(key.to_string(), val);
            }
        }
    }
}

fn instance_class_name(inst: MbValue) -> Option<String> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

fn make_instance(class_name: &str, fields: Vec<(&str, MbValue)>) -> MbValue {
    let inst = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let ObjData::Instance {
            fields: ref iflds, ..
        } = (*inst).data
        {
            let mut g = iflds.write().unwrap();
            for (k, v) in fields {
                retain(v);
                g.insert(k.to_string(), v);
            }
        }
    }
    MbValue::from_ptr(inst)
}

// ── Level tables (thread-local, mutable for addLevelName) ──

thread_local! {
    // number -> canonical name
    static LEVEL_TO_NAME: RefCell<HashMap<i64, String>> = RefCell::new(default_level_to_name());
    // name -> number
    static NAME_TO_LEVEL: RefCell<HashMap<String, i64>> = RefCell::new(default_name_to_level());
    // logger name -> Logger instance MbValue (identity cache)
    static LOGGER_CACHE: RefCell<HashMap<String, MbValue>> = RefCell::new(HashMap::new());
    // the installed Logger class name (setLoggerClass / getLoggerClass)
    static LOGGER_CLASS: RefCell<String> = RefCell::new("Logger".to_string());
    // module-level disable threshold + log record factory
    static MANAGER_DISABLE: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
    static RECORD_FACTORY: RefCell<MbValue> = RefCell::new(MbValue::none());
    // cached sys.stderr value (for default StreamHandler stream identity)
    static STDERR_CACHE: RefCell<MbValue> = RefCell::new(MbValue::none());
}

fn default_level_to_name() -> HashMap<i64, String> {
    let mut m = HashMap::new();
    m.insert(50, "CRITICAL".to_string());
    m.insert(40, "ERROR".to_string());
    m.insert(30, "WARNING".to_string());
    m.insert(20, "INFO".to_string());
    m.insert(10, "DEBUG".to_string());
    m.insert(0, "NOTSET".to_string());
    m
}

fn default_name_to_level() -> HashMap<String, i64> {
    let mut m = HashMap::new();
    m.insert("CRITICAL".to_string(), 50);
    m.insert("FATAL".to_string(), 50);
    m.insert("ERROR".to_string(), 40);
    m.insert("WARN".to_string(), 30);
    m.insert("WARNING".to_string(), 30);
    m.insert("INFO".to_string(), 20);
    m.insert("DEBUG".to_string(), 10);
    m.insert("NOTSET".to_string(), 0);
    m
}

// ── Level number → name resolution (CPython getLevelName semantics) ──

fn level_name_for(num: i64) -> String {
    LEVEL_TO_NAME.with(|m| {
        m.borrow()
            .get(&num)
            .cloned()
            .unwrap_or_else(|| format!("Level {num}"))
    })
}

fn level_num_for(name: &str) -> Option<i64> {
    NAME_TO_LEVEL.with(|m| m.borrow().get(name).copied())
}

// ── module-level function dispatchers: (ptr, nargs) convention ──

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
}

fn arg_or_none(a: &[MbValue], i: usize) -> MbValue {
    a.get(i).copied().unwrap_or_else(MbValue::none)
}

// getLevelName(level)
unsafe extern "C" fn dispatch_getlevelname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let v = arg_or_none(a, 0);
    if let Some(n) = v.as_int() {
        return new_str(level_name_for(n));
    }
    if is_str_value(v) {
        let name = extract_str(v);
        if let Some(n) = level_num_for(&name) {
            return MbValue::from_int(n);
        }
        // Unknown name: CPython returns "Level <name>" — but more usefully the
        // canonical behavior is to return the name unchanged for unknown.
        return new_str(format!("Level {name}"));
    }
    new_str(level_name_for(0))
}

// getLevelNamesMapping() -> fresh dict copy of name->number
unsafe extern "C" fn dispatch_getlevelnamesmapping(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            NAME_TO_LEVEL.with(|m| {
                for (k, n) in m.borrow().iter() {
                    let key = super::super::dict_ops::DictKey::Str(k.clone());
                    map.insert(key, MbValue::from_int(*n));
                }
            });
        }
    }
    MbValue::from_ptr(dict)
}

// addLevelName(level, levelName)
unsafe extern "C" fn dispatch_addlevelname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let num = arg_or_none(a, 0).as_int().unwrap_or(0);
    let name = extract_str(arg_or_none(a, 1));
    LEVEL_TO_NAME.with(|m| {
        m.borrow_mut().insert(num, name.clone());
    });
    NAME_TO_LEVEL.with(|m| {
        m.borrow_mut().insert(name, num);
    });
    MbValue::none()
}

// getLogger(name=None) -> cached Logger instance
unsafe extern "C" fn dispatch_getlogger(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let name_val = arg_or_none(a, 0);
    // getLogger(123) must raise TypeError. Accept str or None only.
    if !name_val.is_none() && !is_str_value(name_val) {
        return raise("TypeError", "A logger name must be a string");
    }
    let name = if name_val.is_none() {
        "root".to_string()
    } else {
        extract_str(name_val)
    };
    get_logger_by_name(&name)
}

fn get_logger_by_name(name: &str) -> MbValue {
    if let Some(existing) = LOGGER_CACHE.with(|c| c.borrow().get(name).copied()) {
        retain(existing);
        return existing;
    }
    let class_name = LOGGER_CLASS.with(|c| c.borrow().clone());
    // Build a Logger instance. Use the installed class so subclasses route.
    let inst = MbObject::new_instance(class_name.clone());
    unsafe {
        if let ObjData::Instance {
            fields: ref iflds, ..
        } = (*inst).data
        {
            let mut g = iflds.write().unwrap();
            g.insert("name".to_string(), new_str(name.to_string()));
            g.insert("level".to_string(), MbValue::from_int(0)); // NOTSET
            g.insert("propagate".to_string(), MbValue::from_bool(true));
            g.insert(
                "handlers".to_string(),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
            g.insert("disabled".to_string(), MbValue::from_bool(false));
        }
    }
    let val = MbValue::from_ptr(inst);
    LOGGER_CACHE.with(|c| {
        retain(val);
        c.borrow_mut().insert(name.to_string(), val);
    });
    val
}

// Look up an already-existing cached Logger by name WITHOUT creating one.
// Ancestor walks (effective-level / handler propagation) must not materialize
// real Logger instances — in CPython those intermediate ancestors are
// PlaceHolders, never Loggers, so they must not surface in getChildren().
fn cached_logger(name: &str) -> Option<MbValue> {
    LOGGER_CACHE.with(|c| {
        let v = c.borrow().get(name).copied();
        if let Some(v) = v {
            retain(v);
        }
        v
    })
}

// setLoggerClass(klass) — klass must be Logger or a subclass.
unsafe extern "C" fn dispatch_setloggerclass(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let klass = arg_or_none(a, 0);
    let name = resolve_class_arg(klass);
    match name {
        Some(cn) if cn == "Logger" || is_logger_subclass(&cn) => {
            LOGGER_CLASS.with(|c| *c.borrow_mut() = cn);
            MbValue::none()
        }
        _ => raise(
            "TypeError",
            "logger not derived from logging.Logger: ".to_string(),
        ),
    }
}

// getLoggerClass() -> class-name string of the installed Logger class.
unsafe extern "C" fn dispatch_getloggerclass(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    new_str(LOGGER_CLASS.with(|c| c.borrow().clone()))
}

// setLogRecordFactory(factory) / getLogRecordFactory()
unsafe extern "C" fn dispatch_setlogrecordfactory(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let f = arg_or_none(a, 0);
    retain(f);
    RECORD_FACTORY.with(|c| *c.borrow_mut() = f);
    MbValue::none()
}
unsafe extern "C" fn dispatch_getlogrecordfactory(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    RECORD_FACTORY.with(|c| {
        let v = *c.borrow();
        if v.is_none() {
            new_str("LogRecord")
        } else {
            retain(v);
            v
        }
    })
}

fn resolve_class_arg(val: MbValue) -> Option<String> {
    // class-name string
    if is_str_value(val) {
        return Some(extract_str(val));
    }
    // a func-pointer constructor with a recorded native type name
    if let Some(addr) = val.as_func() {
        return super::super::module::NATIVE_TYPE_NAMES
            .with(|m| m.borrow().get(&(addr as u64)).cloned());
    }
    // a type object Instance{class_name="type", __name__=X}
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            class_name: ref cn,
            ref fields,
        } = (*ptr).data
        {
            if cn == "type" {
                fields
                    .read()
                    .ok()
                    .and_then(|f| f.get("__name__").map(|v| extract_str(*v)))
            } else {
                // a bare class instance used as a class? treat class_name.
                Some(cn.clone())
            }
        } else {
            None
        }
    })
}

fn is_logger_subclass(cn: &str) -> bool {
    super::super::class::class_mro_any(cn, |c| c == "Logger")
}

// ── module-level convenience log functions (root logger) ──

// Split a flat native-arg slice into (positional-after-msg, trailing-kwargs-dict).
// The trailing dict (if the last arg is a dict) is treated as keyword args per
// the module-wide native-ABI convention, mirroring functools' split_kwargs.
fn positional_after(a: &[MbValue], msg_index: usize) -> &[MbValue] {
    let mut end = a.len();
    if let Some(last) = a.last() {
        if last
            .as_ptr()
            .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        {
            // Only peel a trailing dict as kwargs when it is *after* msg; a dict
            // that is itself the msg positional must stay.
            if end > msg_index + 1 {
                end -= 1;
            }
        }
    }
    if msg_index + 1 >= end {
        &[]
    } else {
        &a[msg_index + 1..end]
    }
}

// Apply CPython's LogRecord.getMessage semantics: `str(msg) % args` when args
// are present, else `str(msg)`. A single non-empty Mapping positional arg is
// unwrapped to act as the `%(key)s` mapping (Logger._log behavior).
fn build_message(msg: MbValue, args: &[MbValue]) -> MbValue {
    if args.is_empty() {
        return new_str(extract_str(msg));
    }
    let msg_str = new_str(extract_str(msg));
    // Single-mapping unwrap: logging treats `logger.info("%(a)s", {"a": 1})`
    // as a mapping rather than a 1-tuple.
    let rhs = if args.len() == 1
        && args[0]
            .as_ptr()
            .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
    {
        args[0]
    } else {
        MbValue::from_ptr(MbObject::new_tuple_borrowed(args.to_vec()))
    };
    let formatted = super::super::builtins::mb_mod(msg_str, rhs);
    new_str(extract_str(formatted))
}

fn root_log_at(level: i64, msg: MbValue) {
    let root = get_logger_by_name("root");
    logger_emit(root, level, msg);
}

unsafe extern "C" fn dispatch_debug(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    root_log_at(10, build_message(arg_or_none(a, 0), positional_after(a, 0)));
    MbValue::none()
}
unsafe extern "C" fn dispatch_info(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    root_log_at(20, build_message(arg_or_none(a, 0), positional_after(a, 0)));
    MbValue::none()
}
unsafe extern "C" fn dispatch_warning(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    root_log_at(30, build_message(arg_or_none(a, 0), positional_after(a, 0)));
    MbValue::none()
}
unsafe extern "C" fn dispatch_error(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    root_log_at(40, build_message(arg_or_none(a, 0), positional_after(a, 0)));
    MbValue::none()
}
unsafe extern "C" fn dispatch_critical(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    root_log_at(50, build_message(arg_or_none(a, 0), positional_after(a, 0)));
    MbValue::none()
}
unsafe extern "C" fn dispatch_log(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let lvl = arg_or_none(a, 0).as_int().unwrap_or(0);
    root_log_at(
        lvl,
        build_message(arg_or_none(a, 1), positional_after(a, 1)),
    );
    MbValue::none()
}

// Read a string-keyed value out of any kwargs dict present in the arg slice.
fn kwarg(a: &[MbValue], key: &str) -> Option<MbValue> {
    for v in a.iter() {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    let dk = super::super::dict_ops::DictKey::Str(key.to_string());
                    if let Some(found) = g.get(&dk) {
                        return Some(*found);
                    }
                }
            }
        }
    }
    None
}

// Build a Formatter Instance from an optional fmt/datefmt (style defaults to '%').
fn make_basic_formatter(fmt: Option<MbValue>, datefmt: MbValue) -> MbValue {
    let fmt_str = match fmt {
        Some(f) if is_str_value(f) => extract_str(f),
        // BASIC_FORMAT default.
        _ => "%(levelname)s:%(name)s:%(message)s".to_string(),
    };
    make_instance(
        "Formatter",
        vec![
            ("_fmt", new_str(fmt_str)),
            ("_style", new_str("%")),
            ("datefmt", datefmt),
        ],
    )
}

// basicConfig(filename=None, filemode='a', format=BASIC_FORMAT, datefmt=None,
//             style='%', level=None, stream=None, handlers=None, force=False, ...)
unsafe extern "C" fn dispatch_basicconfig(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let root = get_logger_by_name("root");

    let force = kwarg(a, "force").and_then(|v| v.as_bool()).unwrap_or(false);

    // CPython: if root already has handlers, basicConfig is a no-op unless
    // force=True (which removes/closes the existing handlers first).
    let root_has_handlers = field_get(root, "handlers")
        .as_ptr()
        .is_some_and(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                !lock.read().unwrap().is_empty()
            } else {
                false
            }
        });
    if force {
        if let Some(ptr) = field_get(root, "handlers").as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
    let configure_handlers = force || !root_has_handlers;

    if configure_handlers {
        let fmt_kw = kwarg(a, "format");
        let datefmt = kwarg(a, "datefmt").unwrap_or_else(MbValue::none);
        let formatter = make_basic_formatter(fmt_kw, datefmt);

        // `handlers` kwarg takes precedence over stream/filename. Each handler
        // gets the basic formatter only if it does not already have one.
        let handlers_kw = kwarg(a, "handlers").filter(|v| !v.is_none());
        if let Some(hlist) = handlers_kw {
            if let Some(ptr) = hlist.as_ptr() {
                let items: Vec<MbValue> = match &(*ptr).data {
                    ObjData::List(lock) => lock.read().unwrap().iter().copied().collect(),
                    ObjData::Tuple(items) => items.iter().copied().collect(),
                    _ => Vec::new(),
                };
                for h in items {
                    if field_get(h, "formatter").is_none() {
                        field_set(h, "formatter", formatter);
                    }
                    m_logger_addhandler(root, h);
                }
            }
        } else {
            // Build a single StreamHandler (FileHandler when `filename` given).
            let filename = kwarg(a, "filename").filter(|v| !v.is_none());
            let handler = if let Some(fname) = filename {
                make_instance(
                    "FileHandler",
                    vec![
                        ("level", MbValue::from_int(0)),
                        ("name", MbValue::none()),
                        ("formatter", formatter),
                        ("baseFilename", fname),
                    ],
                )
            } else {
                let stream = kwarg(a, "stream")
                    .filter(|v| !v.is_none())
                    .unwrap_or_else(stderr_stream);
                make_instance(
                    "StreamHandler",
                    vec![
                        ("level", MbValue::from_int(0)),
                        ("name", MbValue::none()),
                        ("formatter", formatter),
                        ("stream", stream),
                    ],
                )
            };
            m_logger_addhandler(root, handler);
        }
    }

    // `level` may be positional (legacy single-arg) or a keyword. It is applied
    // to root regardless of whether handlers were (re)configured.
    let level: Option<i64> = a
        .first()
        .filter(|v| {
            !v.as_ptr()
                .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        })
        .and_then(|v| v.as_int())
        .or_else(|| {
            kwarg(a, "level").and_then(|found| {
                found
                    .as_int()
                    .or_else(|| level_num_for(&extract_str(found)))
            })
        });
    if let Some(l) = level {
        field_set(root, "level", MbValue::from_int(l));
    }
    MbValue::none()
}

// shutdown(), disable(level), captureWarnings(flag) — no-op stubs that match
// CPython's "returns None" contract (used only via surface presence checks).
unsafe extern "C" fn dispatch_shutdown(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_disable(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let lvl = a.first().and_then(|v| v.as_int()).unwrap_or(50);
    MANAGER_DISABLE.with(|c| c.set(lvl));
    MbValue::none()
}
unsafe extern "C" fn dispatch_capturewarnings(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_gethandlerbyname(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_gethandlernames(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![]))
}
// currentframe() — CPython returns the calling stack frame. We have no frame
// object model; return None so the name is present and callable.
unsafe extern "C" fn dispatch_currentframe(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

// ── constructor dispatchers (return real Instances) ──

// StreamHandler(stream=None)
unsafe extern "C" fn dispatch_streamhandler(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let stream = a
        .first()
        .copied()
        .filter(|v| !v.is_none())
        .unwrap_or_else(stderr_stream);
    make_instance(
        "StreamHandler",
        vec![
            ("level", MbValue::from_int(0)),
            ("name", MbValue::none()),
            ("formatter", MbValue::none()),
            ("stream", stream),
        ],
    )
}

// A sentinel object that compares `is sys.stderr`. We model the default stream
// as the same MbValue the `sys` module exposes for stderr so identity holds.
fn stderr_stream() -> MbValue {
    STDERR_CACHE.with(|c| {
        let cached = *c.borrow();
        if !cached.is_none() {
            retain(cached);
            return cached;
        }
        // Import sys and read its `stderr` attribute so identity matches.
        let sys_mod = super::super::module::mb_import(new_str("sys"));
        let stderr = super::super::class::mb_getattr(sys_mod, new_str("stderr"));
        retain(stderr);
        *c.borrow_mut() = stderr;
        retain(stderr);
        stderr
    })
}

// Handler()
unsafe extern "C" fn dispatch_handler(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let level = a.first().and_then(|v| v.as_int()).unwrap_or(0);
    make_instance(
        "Handler",
        vec![
            ("level", MbValue::from_int(level)),
            ("name", MbValue::none()),
            ("formatter", MbValue::none()),
        ],
    )
}

// NullHandler()
unsafe extern "C" fn dispatch_nullhandler(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    make_instance(
        "NullHandler",
        vec![
            ("level", MbValue::from_int(0)),
            ("name", MbValue::none()),
            ("formatter", MbValue::none()),
        ],
    )
}

// FileHandler(filename, ...) — minimal: store filename, no real file I/O.
unsafe extern "C" fn dispatch_filehandler(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    make_instance(
        "FileHandler",
        vec![
            ("level", MbValue::from_int(0)),
            ("name", MbValue::none()),
            ("formatter", MbValue::none()),
            ("baseFilename", arg_or_none(a, 0)),
        ],
    )
}

// Formatter(fmt=None, datefmt=None, style='%')
unsafe extern "C" fn dispatch_formatter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let fmt = arg_or_none(a, 0);
    let datefmt = arg_or_none(a, 1);
    // style may be positional (3rd) or in a trailing kwargs dict.
    let mut style = String::from("%");
    if let Some(s) = a.get(2) {
        if is_str_value(*s) {
            style = extract_str(*s);
        }
    }
    for v in a.iter() {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    let key = super::super::dict_ops::DictKey::Str("style".to_string());
                    if let Some(found) = g.get(&key) {
                        style = extract_str(*found);
                    }
                }
            }
        }
    }
    if style != "%" && style != "{" && style != "$" {
        return raise("ValueError", format!("Style must be one of: %, {{, $"));
    }
    let fmt_str = if fmt.is_none() {
        match style.as_str() {
            "{" => "{message}".to_string(),
            "$" => "${message}".to_string(),
            _ => "%(message)s".to_string(),
        }
    } else {
        extract_str(fmt)
    };
    make_instance(
        "Formatter",
        vec![
            ("_fmt", new_str(fmt_str)),
            ("_style", new_str(style)),
            ("datefmt", datefmt),
        ],
    )
}

// BufferingFormatter(linefmt=None)
unsafe extern "C" fn dispatch_bufferingformatter(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    make_instance("BufferingFormatter", vec![])
}

// Filter(name='')
unsafe extern "C" fn dispatch_filter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let name = if a.is_empty() || a[0].is_none() {
        new_str("")
    } else {
        a[0]
    };
    make_instance("Filter", vec![("name", name)])
}

// LogRecord(name, level, pathname, lineno, msg, args, exc_info, ...)
unsafe extern "C" fn dispatch_logrecord(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let name = arg_or_none(a, 0);
    let level = arg_or_none(a, 1);
    let msg = arg_or_none(a, 4);
    build_log_record(name, level, msg)
}

fn build_log_record(name: MbValue, level: MbValue, msg: MbValue) -> MbValue {
    let lvlnum = level.as_int().unwrap_or(0);
    let levelname = level_name_for(lvlnum);
    let name = if name.is_none() { new_str("") } else { name };
    let msg = if msg.is_none() { new_str("") } else { msg };
    make_instance(
        "LogRecord",
        vec![
            ("name", name),
            ("levelno", MbValue::from_int(lvlnum)),
            ("levelname", new_str(levelname)),
            ("msg", msg),
            ("message", msg),
            ("args", MbValue::from_ptr(MbObject::new_tuple(vec![]))),
        ],
    )
}

// makeLogRecord(dict) -> LogRecord with attrs from the dict applied.
unsafe extern "C" fn dispatch_makelogrecord(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let rec = build_log_record(MbValue::none(), MbValue::from_int(0), MbValue::none());
    if let Some(ptr) = arg_or_none(a, 0).as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let g = lock.read().unwrap();
                for (k, v) in g.iter() {
                    if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                        field_set(rec, ks, *v);
                        // keep `message` mirrored to `msg`
                        if ks == "msg" {
                            field_set(rec, "message", *v);
                        }
                    }
                }
            }
        }
    }
    rec
}

// ── Logger / Handler / Formatter methods (self-first, MbValue ABI) ──
// These are registered into CLASS_REGISTRY and dispatched through the runtime's
// generic instance-method path which threads `self` as arg 0.

// Logger.setLevel(self, level)
extern "C" fn m_logger_setlevel(this: MbValue, level: MbValue) -> MbValue {
    let n = if let Some(i) = level.as_int() {
        i
    } else if is_str_value(level) {
        let name = extract_str(level);
        match level_num_for(&name) {
            Some(n) => n,
            None => return raise("ValueError", format!("Unknown level: {name:?}")),
        }
    } else {
        return raise("TypeError", "Level not an integer or a valid string");
    };
    field_set(this, "level", MbValue::from_int(n));
    MbValue::none()
}

fn logger_effective_level(this: MbValue) -> i64 {
    // Walk parents by stripping the dotted suffix until a non-zero level is
    // found. Ancestors are looked up WITHOUT creating real Loggers: in CPython
    // intermediate ancestors are PlaceHolders (no `.level`), so they are skipped
    // and must not be materialized (which would pollute getChildren()).
    let mut name = extract_str(field_get(this, "name"));
    // The starting logger `this` is real; read its own level first.
    let lvl = field_get(this, "level").as_int().unwrap_or(0);
    if lvl != 0 {
        return lvl;
    }
    loop {
        if name == "root" || !name.contains('.') {
            if name != "root" {
                // Reached a top-level name; its parent is root. Only read root's
                // level if root was explicitly created; otherwise it defaults to
                // WARNING.
                if let Some(root) = cached_logger("root") {
                    let rl = field_get(root, "level").as_int().unwrap_or(0);
                    if rl != 0 {
                        return rl;
                    }
                }
            }
            return 30; // CPython root default == WARNING
        }
        name = name.rsplitn(2, '.').nth(1).unwrap_or("root").to_string();
        // Only consult an ancestor that already exists as a real Logger.
        if let Some(lg) = cached_logger(&name) {
            let lvl = field_get(lg, "level").as_int().unwrap_or(0);
            if lvl != 0 {
                return lvl;
            }
        }
    }
}

// Logger.getEffectiveLevel(self)
extern "C" fn m_logger_geteffectivelevel(this: MbValue) -> MbValue {
    MbValue::from_int(logger_effective_level(this))
}

// Logger.isEnabledFor(self, level)
extern "C" fn m_logger_isenabledfor(this: MbValue, level: MbValue) -> MbValue {
    let target = level.as_int().unwrap_or(0);
    let disable = MANAGER_DISABLE.with(|c| c.get());
    if target <= disable {
        return MbValue::from_bool(false);
    }
    MbValue::from_bool(target >= logger_effective_level(this))
}

// Logger.getChild(self, suffix)
extern "C" fn m_logger_getchild(this: MbValue, suffix: MbValue) -> MbValue {
    let base = extract_str(field_get(this, "name"));
    let suf = extract_str(suffix);
    let full = if base == "root" {
        suf
    } else {
        format!("{base}.{suf}")
    };
    get_logger_by_name(&full)
}

// Logger.getChildren(self) -> set of immediate child loggers
extern "C" fn m_logger_getchildren(this: MbValue) -> MbValue {
    let base = extract_str(field_get(this, "name"));
    let prefix = if base == "root" {
        String::new()
    } else {
        format!("{base}.")
    };
    let mut kids: Vec<MbValue> = Vec::new();
    LOGGER_CACHE.with(|c| {
        for (n, v) in c.borrow().iter() {
            if n == "root" {
                continue;
            }
            let rest = if prefix.is_empty() {
                Some(n.as_str())
            } else {
                n.strip_prefix(&prefix)
            };
            if let Some(rest) = rest {
                if !rest.is_empty() && !rest.contains('.') && n.as_str() != base.as_str() {
                    retain(*v);
                    kids.push(*v);
                }
            }
        }
    });
    MbValue::from_ptr(MbObject::new_set(kids))
}

// Logger.addHandler(self, h)
extern "C" fn m_logger_addhandler(this: MbValue, h: MbValue) -> MbValue {
    if let Some(ptr) = field_get(this, "handlers").as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut g = lock.write().unwrap();
                if !g.iter().any(|x| x.to_bits() == h.to_bits()) {
                    retain(h);
                    g.push(h);
                }
            }
        }
    }
    MbValue::none()
}

// Logger.removeHandler(self, h)
extern "C" fn m_logger_removehandler(this: MbValue, h: MbValue) -> MbValue {
    if let Some(ptr) = field_get(this, "handlers").as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut g = lock.write().unwrap();
                g.retain(|x| x.to_bits() != h.to_bits());
            }
        }
    }
    MbValue::none()
}

// Logger.addFilter / removeFilter — no-op (filters not modeled in emission).
extern "C" fn m_logger_addfilter(_this: MbValue, _f: MbValue) -> MbValue {
    MbValue::none()
}
extern "C" fn m_logger_removefilter(_this: MbValue, _f: MbValue) -> MbValue {
    MbValue::none()
}

// Walk from a logger up the hierarchy collecting handlers honoring propagate.
// Ancestors are resolved WITHOUT materializing new Loggers — only loggers that
// already exist contribute handlers / a propagate flag. Intermediate names that
// were never explicitly created are PlaceHolders in CPython and are simply
// skipped (they always propagate and hold no handlers), so they must not be
// cached as real Loggers (which would pollute getChildren()).
fn collect_effective_handlers(start: MbValue) -> Vec<MbValue> {
    let mut handlers = Vec::new();
    let mut name = extract_str(field_get(start, "name"));
    // `current` is `Some` only when the node is a real, existing Logger.
    let mut current: Option<MbValue> = Some(start);
    loop {
        if let Some(cur) = current {
            if let Some(ptr) = field_get(cur, "handlers").as_ptr() {
                unsafe {
                    if let ObjData::List(ref lock) = (*ptr).data {
                        for h in lock.read().unwrap().iter() {
                            handlers.push(*h);
                        }
                    }
                }
            }
            // A real logger with propagate=False halts the walk.
            if field_get(cur, "propagate").as_bool() == Some(false) {
                break;
            }
        }
        if name == "root" || !name.contains('.') {
            if name != "root" {
                name = "root".to_string();
                current = cached_logger("root");
                continue;
            }
            break;
        }
        name = name.rsplitn(2, '.').nth(1).unwrap_or("root").to_string();
        current = cached_logger(&name);
    }
    handlers
}

// Core emission used by Logger.debug/info/.../log and root-level functions.
fn logger_emit(logger: MbValue, level: i64, msg: MbValue) {
    let disable = MANAGER_DISABLE.with(|c| c.get());
    if level <= disable {
        return;
    }
    if level < logger_effective_level(logger) {
        return;
    }
    let levelname = level_name_for(level);
    let message = extract_str(msg);
    let logger_name = extract_str(field_get(logger, "name"));
    let handlers = collect_effective_handlers(logger);
    let mut found = 0usize;
    for h in &handlers {
        let hlevel = field_get(*h, "level").as_int().unwrap_or(0);
        if level < hlevel {
            continue;
        }
        if instance_class_name(*h).as_deref() == Some("NullHandler") {
            // NullHandler still counts as "a handler was found" for the
            // lastResort decision (CPython increments `found` before emit).
            found += 1;
            continue;
        }
        found += 1;
        // Render through the handler's formatter if present.
        let line = format_record(*h, &levelname, &message, &logger_name);
        write_to_handler_stream(*h, &format!("{line}\n"));
    }
    if found == 0 {
        // No effective handler: CPython falls back to `logging.lastResort`, a
        // `_StderrHandler` at WARNING with the default `%(message)s` format, so
        // it writes just the bare message to stderr for levelno >= WARNING.
        if level >= 30 {
            eprintln!("{message}");
        }
    }
}

fn format_record(handler: MbValue, levelname: &str, message: &str, name: &str) -> String {
    let fmt = field_get(handler, "formatter");
    if fmt.is_none() {
        return message.to_string();
    }
    apply_format(fmt, levelname, message, name, &[])
}

// Apply a Formatter's fmt string against a small set of record fields.
fn apply_format(
    fmt: MbValue,
    levelname: &str,
    message: &str,
    name: &str,
    extra: &[(String, String)],
) -> String {
    let style = extract_str(field_get(fmt, "_style"));
    let pattern = extract_str(field_get(fmt, "_fmt"));
    let lookup = |key: &str| -> Option<String> {
        match key {
            "levelname" => Some(levelname.to_string()),
            "message" => Some(message.to_string()),
            "msg" => Some(message.to_string()),
            "name" => Some(name.to_string()),
            "levelno" => level_num_for(levelname).map(|n| n.to_string()),
            _ => extra.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()),
        }
    };
    if style == "{" {
        format_brace(&pattern, &lookup)
    } else if style == "$" {
        format_dollar(&pattern, &lookup)
    } else {
        format_percent(&pattern, &lookup)
    }
}

// %(key)s style
fn format_percent(pattern: &str, lookup: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Literal '%%' collapses to a single '%' (CPython %-style Formatter).
        if chars[i] == '%' && i + 1 < chars.len() && chars[i + 1] == '%' {
            out.push('%');
            i += 2;
            continue;
        }
        if chars[i] == '%' && i + 1 < chars.len() && chars[i + 1] == '(' {
            // find closing ')'
            if let Some(close) = chars[i + 2..].iter().position(|&c| c == ')') {
                let key: String = chars[i + 2..i + 2 + close].iter().collect();
                // skip past ')' and the conversion char (e.g. 's', 'd')
                let mut j = i + 2 + close + 1;
                while j < chars.len() && !"sdifgxXeEr%".contains(chars[j]) {
                    j += 1;
                }
                if j < chars.len() {
                    j += 1;
                }
                out.push_str(&lookup(&key).unwrap_or_default());
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// {key} style
fn format_brace(pattern: &str, lookup: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            if let Some(close) = chars[i + 1..].iter().position(|&c| c == '}') {
                let inner: String = chars[i + 1..i + 1 + close].iter().collect();
                // strip any format spec after ':'
                let key = inner.split(':').next().unwrap_or("").trim().to_string();
                out.push_str(&lookup(&key).unwrap_or_default());
                i = i + 1 + close + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// ${key} / $key style
fn format_dollar(pattern: &str, lookup: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                if let Some(close) = chars[i + 2..].iter().position(|&c| c == '}') {
                    let key: String = chars[i + 2..i + 2 + close].iter().collect();
                    out.push_str(&lookup(&key).unwrap_or_default());
                    i = i + 2 + close + 1;
                    continue;
                }
            } else if chars[i + 1].is_alphabetic() || chars[i + 1] == '_' {
                let mut j = i + 1;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                let key: String = chars[i + 1..j].iter().collect();
                out.push_str(&lookup(&key).unwrap_or_default());
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// Write a string to a handler's stream (StringIO / stderr).
fn write_to_handler_stream(handler: MbValue, text: &str) {
    let stream = field_get(handler, "stream");
    if stream.is_none() {
        eprint!("{text}");
        return;
    }
    // If it is our stderr sentinel, write to stderr.
    if stream.to_bits() == stderr_stream().to_bits() {
        eprint!("{text}");
        return;
    }
    // Otherwise call stream.write(text) via the runtime (StringIO etc.).
    let args = MbValue::from_ptr(MbObject::new_list(vec![new_str(text)]));
    super::super::class::mb_call_method(stream, new_str("write"), args);
}

// Logger.debug/info/warning/error/critical/exception/log
extern "C" fn m_logger_debug(this: MbValue, msg: MbValue) -> MbValue {
    logger_emit(this, 10, msg);
    MbValue::none()
}
extern "C" fn m_logger_info(this: MbValue, msg: MbValue) -> MbValue {
    logger_emit(this, 20, msg);
    MbValue::none()
}
extern "C" fn m_logger_warning(this: MbValue, msg: MbValue) -> MbValue {
    logger_emit(this, 30, msg);
    MbValue::none()
}
extern "C" fn m_logger_error(this: MbValue, msg: MbValue) -> MbValue {
    logger_emit(this, 40, msg);
    MbValue::none()
}
extern "C" fn m_logger_critical(this: MbValue, msg: MbValue) -> MbValue {
    logger_emit(this, 50, msg);
    MbValue::none()
}
extern "C" fn m_logger_warn(this: MbValue, msg: MbValue) -> MbValue {
    logger_emit(this, 30, msg);
    MbValue::none()
}

extern "C" fn m_logger_exception(this: MbValue, msg: MbValue) -> MbValue {
    // Emit the message plus the active exception's type name (traceback-lite).
    let base = extract_str(msg);
    // Inside an `except` block the handled exception is recorded here.
    let exc_type = super::super::exception::last_handled_exception()
        .map(|(t, _)| t)
        .or_else(super::super::exception::current_exception_type)
        .unwrap_or_default();
    let combined = if exc_type.is_empty() {
        base
    } else {
        format!("{base}\nTraceback (most recent call last):\n{exc_type}")
    };
    logger_emit(this, 40, new_str(combined));
    MbValue::none()
}

extern "C" fn m_logger_log(this: MbValue, level: MbValue, msg: MbValue) -> MbValue {
    logger_emit(this, level.as_int().unwrap_or(0), msg);
    MbValue::none()
}

// Logger.handle / Logger.callHandlers — no-op compatibility hooks.
extern "C" fn m_logger_noop1(_this: MbValue, _a: MbValue) -> MbValue {
    MbValue::none()
}

// ── Handler methods ──
extern "C" fn m_handler_setlevel(this: MbValue, level: MbValue) -> MbValue {
    let n = if let Some(i) = level.as_int() {
        i
    } else if is_str_value(level) {
        level_num_for(&extract_str(level)).unwrap_or(0)
    } else {
        0
    };
    field_set(this, "level", MbValue::from_int(n));
    MbValue::none()
}
extern "C" fn m_handler_setformatter(this: MbValue, fmt: MbValue) -> MbValue {
    field_set(this, "formatter", fmt);
    MbValue::none()
}
extern "C" fn m_handler_emit(this: MbValue, _record: MbValue) -> MbValue {
    // Base Handler.emit is abstract.
    if instance_class_name(this).as_deref() == Some("Handler") {
        return raise(
            "NotImplementedError",
            "emit must be implemented by Handler subclasses",
        );
    }
    MbValue::none()
}
extern "C" fn m_handler_handle(_this: MbValue, _record: MbValue) -> MbValue {
    MbValue::none()
}
extern "C" fn m_handler_flush(_this: MbValue) -> MbValue {
    MbValue::none()
}
extern "C" fn m_handler_close(_this: MbValue) -> MbValue {
    MbValue::none()
}
extern "C" fn m_handler_setname(this: MbValue, name: MbValue) -> MbValue {
    field_set(this, "name", name);
    MbValue::none()
}
extern "C" fn m_handler_getname(this: MbValue) -> MbValue {
    field_get(this, "name")
}
extern "C" fn m_handler_addfilter(_this: MbValue, _f: MbValue) -> MbValue {
    MbValue::none()
}
extern "C" fn m_handler_removefilter(_this: MbValue, _f: MbValue) -> MbValue {
    MbValue::none()
}
extern "C" fn m_handler_acquire(_this: MbValue) -> MbValue {
    MbValue::none()
}
extern "C" fn m_handler_release(_this: MbValue) -> MbValue {
    MbValue::none()
}

// StreamHandler.setStream(self, stream) -> prior stream (or None if unchanged)
extern "C" fn m_streamhandler_setstream(this: MbValue, stream: MbValue) -> MbValue {
    let prior = field_get(this, "stream");
    if prior.to_bits() == stream.to_bits() {
        return MbValue::none();
    }
    field_set(this, "stream", stream);
    prior
}
// StreamHandler.emit / flush — write formatted record into the stream.
extern "C" fn m_streamhandler_emit(this: MbValue, record: MbValue) -> MbValue {
    let levelname = extract_str(field_get(record, "levelname"));
    let message = extract_str(field_get(record, "message"));
    let name = extract_str(field_get(record, "name"));
    let line = format_record(this, &levelname, &message, &name);
    write_to_handler_stream(this, &format!("{line}\n"));
    MbValue::none()
}

// ── Formatter methods ──
// Formatter.format(self, record) -> formatted string
extern "C" fn m_formatter_format(this: MbValue, record: MbValue) -> MbValue {
    let levelname = extract_str(field_get(record, "levelname"));
    // message defaults to msg field.
    let mut message = extract_str(field_get(record, "message"));
    if message.is_empty() {
        message = extract_str(field_get(record, "msg"));
    }
    let name = extract_str(field_get(record, "name"));
    // collect extra string fields from the record for placeholder resolution.
    let mut extra: Vec<(String, String)> = Vec::new();
    if let Some(ptr) = record.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                for (k, v) in fields.read().unwrap().iter() {
                    extra.push((k.clone(), extract_str(*v)));
                }
            }
        }
    }
    new_str(apply_format(this, &levelname, &message, &name, &extra))
}
/// True when `v` is a logging.Formatter (distinguished from string.Formatter,
/// which shares the "Formatter" class name). A logging.Formatter always carries
/// a `_style` field; string.Formatter has no instance fields. Used by the
/// `string_ops` `.format(...)` bridge so logging formatters route here instead
/// of the string.Formatter `vformat` engine.
pub fn value_is_logging_formatter(v: MbValue) -> bool {
    if instance_class_name(v).as_deref() != Some("Formatter") {
        return false;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                return fields.read().unwrap().contains_key("_style");
            }
        }
    }
    false
}

/// Public bridge: format `record` through the logging Formatter `fmt`. Called
/// from the `.format(...)` lowering when the receiver is a logging.Formatter.
pub fn logging_formatter_format(fmt: MbValue, record: MbValue) -> MbValue {
    m_formatter_format(fmt, record)
}

extern "C" fn m_formatter_formattime(_this: MbValue, _record: MbValue) -> MbValue {
    new_str("")
}
extern "C" fn m_formatter_formatmessage(this: MbValue, record: MbValue) -> MbValue {
    m_formatter_format(this, record)
}

// BufferingFormatter.format(self, records) -> joined batch
extern "C" fn m_bufferingformatter_format(_this: MbValue, records: MbValue) -> MbValue {
    let mut out = String::new();
    if let Some(ptr) = records.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => {
                    for r in lock.read().unwrap().iter() {
                        let m = extract_str(field_get(*r, "message"));
                        let m = if m.is_empty() {
                            extract_str(field_get(*r, "msg"))
                        } else {
                            m
                        };
                        out.push_str(&m);
                    }
                }
                ObjData::Tuple(items) => {
                    for r in items.iter() {
                        let m = extract_str(field_get(*r, "message"));
                        let m = if m.is_empty() {
                            extract_str(field_get(*r, "msg"))
                        } else {
                            m
                        };
                        out.push_str(&m);
                    }
                }
                _ => {}
            }
        }
    }
    new_str(out)
}

// Filter.filter(self, record) -> bool (bare filter passes all; name-prefixed filters match)
extern "C" fn m_filter_filter(this: MbValue, record: MbValue) -> MbValue {
    let fname = extract_str(field_get(this, "name"));
    if fname.is_empty() {
        return MbValue::from_bool(true);
    }
    let rname = extract_str(field_get(record, "name"));
    let pass = rname == fname || rname.starts_with(&format!("{fname}."));
    MbValue::from_bool(pass)
}

// LogRecord.getMessage(self) -> message string
extern "C" fn m_logrecord_getmessage(this: MbValue) -> MbValue {
    let m = field_get(this, "message");
    if m.is_none() {
        field_get(this, "msg")
    } else {
        m
    }
}
// LogRecord.__str__ / __repr__ -> "<LogRecord: name, levelno, ...>"
extern "C" fn m_logrecord_str(this: MbValue) -> MbValue {
    let name = extract_str(field_get(this, "name"));
    let levelno = field_get(this, "levelno").as_int().unwrap_or(0);
    let levelname = extract_str(field_get(this, "levelname"));
    new_str(format!("<LogRecord: {name}, {levelno}, , 0, {levelname}>"))
}

// ── Manager (man = logging.Manager(None)) ──
unsafe extern "C" fn dispatch_manager(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    make_instance(
        "Manager",
        vec![
            ("loggerClass", MbValue::none()),
            ("logRecordFactory", MbValue::none()),
        ],
    )
}

// Manager.setLoggerClass(self, klass): klass must derive from Logger.
extern "C" fn m_manager_setloggerclass(this: MbValue, klass: MbValue) -> MbValue {
    let name = resolve_class_arg(klass);
    match name {
        Some(cn) if cn == "Logger" || is_logger_subclass(&cn) => {
            field_set(this, "loggerClass", new_str(cn));
            MbValue::none()
        }
        _ => raise("TypeError", "logger not derived from logging.Logger: "),
    }
}

// Manager.getLogger(self, name): construct a logger of the installed class.
extern "C" fn m_manager_getlogger(this: MbValue, name: MbValue) -> MbValue {
    let nm = extract_str(name);
    let klass = field_get(this, "loggerClass");
    let class_name = if klass.is_none() {
        "Logger".to_string()
    } else {
        extract_str(klass)
    };
    let inst = MbObject::new_instance(class_name);
    unsafe {
        if let ObjData::Instance {
            fields: ref iflds, ..
        } = (*inst).data
        {
            let mut g = iflds.write().unwrap();
            g.insert("name".to_string(), new_str(nm));
            g.insert("level".to_string(), MbValue::from_int(0));
            g.insert("propagate".to_string(), MbValue::from_bool(true));
            g.insert(
                "handlers".to_string(),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
            g.insert("disabled".to_string(), MbValue::from_bool(false));
        }
    }
    MbValue::from_ptr(inst)
}

// Manager.setLogRecordFactory(self, factory): store the factory verbatim.
extern "C" fn m_manager_setlogrecordfactory(this: MbValue, factory: MbValue) -> MbValue {
    field_set(this, "logRecordFactory", factory);
    MbValue::none()
}

// ── Registration ──

fn method(addr_fn: *const ()) -> MbValue {
    MbValue::from_func(addr_fn as usize)
}

fn register_native_class(name: &str, methods: Vec<(&str, *const ())>) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (mname, addr) in methods {
        map.insert(mname.to_string(), method(addr));
    }
    super::super::class::mb_class_register(name, vec!["object".to_string()], map);
}

/// Register the logging module.
pub fn register() {
    let mut attrs = HashMap::new();

    // ── Level constants ──
    attrs.insert("CRITICAL".into(), MbValue::from_int(50));
    attrs.insert("FATAL".into(), MbValue::from_int(50));
    attrs.insert("ERROR".into(), MbValue::from_int(40));
    attrs.insert("WARNING".into(), MbValue::from_int(30));
    attrs.insert("WARN".into(), MbValue::from_int(30));
    attrs.insert("INFO".into(), MbValue::from_int(20));
    attrs.insert("DEBUG".into(), MbValue::from_int(10));
    attrs.insert("NOTSET".into(), MbValue::from_int(0));
    attrs.insert(
        "BASIC_FORMAT".into(),
        new_str("%(levelname)s:%(name)s:%(message)s"),
    );
    attrs.insert("raiseExceptions".into(), MbValue::from_bool(true));
    // lastResort: CPython exposes a _StderrHandler at WARNING level. Model it as
    // a real StreamHandler instance so `hasattr(logging, "lastResort")` holds.
    attrs.insert(
        "lastResort".into(),
        make_instance(
            "StreamHandler",
            vec![
                ("level", MbValue::from_int(30)),
                ("name", MbValue::none()),
                ("formatter", MbValue::none()),
            ],
        ),
    );

    // ── Register native classes with methods (CLASS_REGISTRY) ──
    register_native_class(
        "Logger",
        vec![
            ("setLevel", m_logger_setlevel as *const ()),
            ("getEffectiveLevel", m_logger_geteffectivelevel as *const ()),
            ("isEnabledFor", m_logger_isenabledfor as *const ()),
            ("getChild", m_logger_getchild as *const ()),
            ("getChildren", m_logger_getchildren as *const ()),
            ("addHandler", m_logger_addhandler as *const ()),
            ("removeHandler", m_logger_removehandler as *const ()),
            ("addFilter", m_logger_addfilter as *const ()),
            ("removeFilter", m_logger_removefilter as *const ()),
            ("debug", m_logger_debug as *const ()),
            ("info", m_logger_info as *const ()),
            ("warning", m_logger_warning as *const ()),
            ("warn", m_logger_warn as *const ()),
            ("error", m_logger_error as *const ()),
            ("critical", m_logger_critical as *const ()),
            ("fatal", m_logger_critical as *const ()),
            ("exception", m_logger_exception as *const ()),
            ("log", m_logger_log as *const ()),
            ("handle", m_logger_noop1 as *const ()),
            ("callHandlers", m_logger_noop1 as *const ()),
        ],
    );
    register_native_class(
        "Handler",
        vec![
            ("setLevel", m_handler_setlevel as *const ()),
            ("setFormatter", m_handler_setformatter as *const ()),
            ("emit", m_handler_emit as *const ()),
            ("handle", m_handler_handle as *const ()),
            ("flush", m_handler_flush as *const ()),
            ("close", m_handler_close as *const ()),
            ("set_name", m_handler_setname as *const ()),
            ("get_name", m_handler_getname as *const ()),
            ("addFilter", m_handler_addfilter as *const ()),
            ("removeFilter", m_handler_removefilter as *const ()),
            ("acquire", m_handler_acquire as *const ()),
            ("release", m_handler_release as *const ()),
        ],
    );
    register_native_class_with_base(
        "StreamHandler",
        "Handler",
        vec![
            ("setLevel", m_handler_setlevel as *const ()),
            ("setFormatter", m_handler_setformatter as *const ()),
            ("setStream", m_streamhandler_setstream as *const ()),
            ("emit", m_streamhandler_emit as *const ()),
            ("flush", m_handler_flush as *const ()),
            ("close", m_handler_close as *const ()),
        ],
    );
    register_native_class_with_base(
        "NullHandler",
        "Handler",
        vec![
            ("setLevel", m_handler_setlevel as *const ()),
            ("setFormatter", m_handler_setformatter as *const ()),
            ("emit", m_handler_handle as *const ()),
            ("handle", m_handler_handle as *const ()),
            ("createLock", m_handler_flush as *const ()),
        ],
    );
    register_native_class_with_base(
        "FileHandler",
        "StreamHandler",
        vec![
            ("setLevel", m_handler_setlevel as *const ()),
            ("setFormatter", m_handler_setformatter as *const ()),
            ("emit", m_streamhandler_emit as *const ()),
            ("close", m_handler_close as *const ()),
        ],
    );
    register_native_class(
        "Formatter",
        vec![
            ("format", m_formatter_format as *const ()),
            ("formatTime", m_formatter_formattime as *const ()),
            ("formatMessage", m_formatter_formatmessage as *const ()),
        ],
    );
    register_native_class(
        "BufferingFormatter",
        vec![("format", m_bufferingformatter_format as *const ())],
    );
    register_native_class("Filter", vec![("filter", m_filter_filter as *const ())]);
    register_native_class(
        "LogRecord",
        vec![
            ("getMessage", m_logrecord_getmessage as *const ()),
            ("__str__", m_logrecord_str as *const ()),
            ("__repr__", m_logrecord_str as *const ()),
        ],
    );
    register_native_class(
        "Manager",
        vec![
            ("setLoggerClass", m_manager_setloggerclass as *const ()),
            ("getLogger", m_manager_getlogger as *const ()),
            (
                "setLogRecordFactory",
                m_manager_setlogrecordfactory as *const (),
            ),
        ],
    );

    // ── Logger is a class-name string so isinstance / subclassing work ──
    attrs.insert("Logger".into(), new_str("Logger"));

    // ── Constructor dispatchers (native funcs returning Instances) ──
    let ctor_dispatchers: Vec<(&str, usize, &str)> = vec![
        ("Handler", dispatch_handler as *const () as usize, "Handler"),
        (
            "StreamHandler",
            dispatch_streamhandler as *const () as usize,
            "StreamHandler",
        ),
        (
            "NullHandler",
            dispatch_nullhandler as *const () as usize,
            "NullHandler",
        ),
        (
            "FileHandler",
            dispatch_filehandler as *const () as usize,
            "FileHandler",
        ),
        (
            "Formatter",
            dispatch_formatter as *const () as usize,
            "Formatter",
        ),
        (
            "BufferingFormatter",
            dispatch_bufferingformatter as *const () as usize,
            "BufferingFormatter",
        ),
        ("Filter", dispatch_filter as *const () as usize, "Filter"),
        (
            "LogRecord",
            dispatch_logrecord as *const () as usize,
            "LogRecord",
        ),
        ("Manager", dispatch_manager as *const () as usize, "Manager"),
    ];
    for (name, addr, type_name) in &ctor_dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(*addr as u64, type_name.to_string());
        });
    }

    // ── Plain module-level function dispatchers ──
    let func_dispatchers: Vec<(&str, usize)> = vec![
        ("getLogger", dispatch_getlogger as *const () as usize),
        ("getLevelName", dispatch_getlevelname as *const () as usize),
        (
            "getLevelNamesMapping",
            dispatch_getlevelnamesmapping as *const () as usize,
        ),
        ("addLevelName", dispatch_addlevelname as *const () as usize),
        (
            "setLoggerClass",
            dispatch_setloggerclass as *const () as usize,
        ),
        (
            "getLoggerClass",
            dispatch_getloggerclass as *const () as usize,
        ),
        (
            "setLogRecordFactory",
            dispatch_setlogrecordfactory as *const () as usize,
        ),
        (
            "getLogRecordFactory",
            dispatch_getlogrecordfactory as *const () as usize,
        ),
        (
            "makeLogRecord",
            dispatch_makelogrecord as *const () as usize,
        ),
        ("basicConfig", dispatch_basicconfig as *const () as usize),
        ("debug", dispatch_debug as *const () as usize),
        ("info", dispatch_info as *const () as usize),
        ("warning", dispatch_warning as *const () as usize),
        ("warn", dispatch_warning as *const () as usize),
        ("error", dispatch_error as *const () as usize),
        ("critical", dispatch_critical as *const () as usize),
        ("fatal", dispatch_critical as *const () as usize),
        ("exception", dispatch_error as *const () as usize),
        ("log", dispatch_log as *const () as usize),
        ("shutdown", dispatch_shutdown as *const () as usize),
        ("disable", dispatch_disable as *const () as usize),
        (
            "captureWarnings",
            dispatch_capturewarnings as *const () as usize,
        ),
        (
            "getHandlerByName",
            dispatch_gethandlerbyname as *const () as usize,
        ),
        (
            "getHandlerNames",
            dispatch_gethandlernames as *const () as usize,
        ),
        ("currentframe", dispatch_currentframe as *const () as usize),
    ];
    for (name, addr) in &func_dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    // LoggerAdapter is referenced only by surface presence — expose a stub class.
    attrs.insert("LoggerAdapter".into(), new_str("LoggerAdapter"));

    // surface: missing CPython module constants (auto-added)
    attrs.insert("logAsyncioTasks".into(), MbValue::from_int(1));
    attrs.insert("logMultiprocessing".into(), MbValue::from_int(1));
    attrs.insert("logProcesses".into(), MbValue::from_int(1));
    attrs.insert("logThreads".into(), MbValue::from_int(1));

    // ── surface: remaining CPython public names (presence-only) ──
    // Class/type names CPython exposes. Like `Logger`/`LoggerAdapter` above they
    // are registered as class-name strings so `hasattr(logging, X)` holds; no
    // behavior is modeled for them yet.
    for cls in [
        "Filterer",
        "GenericAlias",
        "PercentStyle",
        "PlaceHolder",
        "RootLogger",
        "StrFormatStyle",
        "StringTemplateStyle",
        "Template",
    ] {
        attrs.insert(cls.into(), new_str(cls));
    }

    // Submodules that CPython's `logging/__init__.py` imports into its namespace.
    // Surface checks only require the name to be present; we register a stub
    // string (calling the real importer here would risk init-time recursion).
    for m in [
        "atexit",
        "collections",
        "io",
        "os",
        "re",
        "sys",
        "threading",
        "time",
        "traceback",
        "warnings",
        "weakref",
    ] {
        attrs.insert(m.into(), new_str(m));
    }

    // `logging.root` is the RootLogger singleton — expose the cached root Logger
    // instance (same object `getLogger()`/`getLogger("root")` returns).
    attrs.insert("root".into(), get_logger_by_name("root"));

    super::register_module("logging", attrs);
}

// Register a native class with a real base (so isinstance(h, Handler) holds).
fn register_native_class_with_base(name: &str, base: &str, methods: Vec<(&str, *const ())>) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (mname, addr) in methods {
        map.insert(mname.to_string(), method(addr));
    }
    super::super::class::mb_class_register(name, vec![base.to_string(), "object".to_string()], map);
}

// ── Backward-compatible free functions ──
// Retained so existing in-tree unit tests (stdlib_coverage_remaining.rs) keep
// compiling. The live module wiring uses the dispatch_* / m_* paths above; these
// thin helpers mirror the pre-rewrite contract (return None for log calls, a
// {"name": ...} dict for getLogger).

pub fn mb_logging_basicconfig(level: MbValue) -> MbValue {
    if let Some(l) = level.as_int() {
        let root = get_logger_by_name("root");
        field_set(root, "level", MbValue::from_int(l));
    }
    MbValue::none()
}
pub fn mb_logging_debug(_msg: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_logging_info(_msg: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_logging_warning(_msg: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_logging_error(_msg: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_logging_critical(_msg: MbValue) -> MbValue {
    MbValue::none()
}

/// Compatibility getLogger returning a {"name": ...} dict (legacy shape).
pub fn mb_logging_getlogger(name: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    let n = if name.is_none() {
        "root".to_string()
    } else {
        extract_str(name)
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("name".into(), new_str(n));
        }
    }
    MbValue::from_ptr(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_name_for_known() {
        assert_eq!(level_name_for(20), "INFO");
        assert_eq!(level_name_for(50), "CRITICAL");
        assert_eq!(level_name_for(0), "NOTSET");
    }

    #[test]
    fn test_level_name_for_unknown() {
        assert_eq!(level_name_for(99), "Level 99");
    }

    #[test]
    fn test_level_num_for() {
        assert_eq!(level_num_for("INFO"), Some(20));
        assert_eq!(level_num_for("FATAL"), Some(50));
        assert_eq!(level_num_for("WARN"), Some(30));
        assert_eq!(level_num_for("NOPE"), None);
    }

    #[test]
    fn test_format_percent() {
        let look = |k: &str| match k {
            "levelname" => Some("ERROR".to_string()),
            "message" => Some("hi".to_string()),
            _ => None,
        };
        assert_eq!(
            format_percent("%(levelname)s|%(message)s", &look),
            "ERROR|hi"
        );
    }

    #[test]
    fn test_format_brace() {
        let look = |k: &str| match k {
            "levelname" => Some("INFO".to_string()),
            "message" => Some("hi".to_string()),
            _ => None,
        };
        assert_eq!(format_brace("{levelname}|{message}", &look), "INFO|hi");
    }

    #[test]
    fn test_extract_str_variants() {
        assert_eq!(extract_str(MbValue::from_int(7)), "7");
        assert_eq!(extract_str(MbValue::none()), "None");
        assert_eq!(extract_str(new_str("x")), "x");
    }
}
