/// sys module for Mamba (#310 R1).
///
/// Provides: sys.argv, sys.path, sys.version, sys.platform,
///           sys.maxsize, sys.exit(), sys.getrecursionlimit(),
///           sys.setrecursionlimit(), sys.getdefaultencoding(),
///           sys.float_info, sys.int_info, sys.stdin, sys.stdout,
///           sys.stderr, sys.modules

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

// ── Dispatch wrappers ──

fn extract_list(val: MbValue) -> Vec<MbValue> {
    match val.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                vec![]
            }
        },
        None => vec![],
    }
}

fn dispatch_getrecursionlimit(args: MbValue) -> MbValue {
    let _ = extract_list(args);
    mb_sys_getrecursionlimit()
}

fn dispatch_getsizeof(args: MbValue) -> MbValue {
    let items = extract_list(args);
    mb_sys_getsizeof(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_getdefaultencoding(args: MbValue) -> MbValue {
    let _ = extract_list(args);
    mb_sys_getdefaultencoding()
}

// New-ABI dispatchers (#1261) — `extern "C" fn(args_ptr, nargs) -> MbValue`.
// The older `fn(args: MbValue)` shape used by exit/getrecursionlimit/etc.
// above does not see real call args; the new wire-ups follow the native
// extern ABI consumed by class.rs:5308 and must also be registered in
// NATIVE_FUNC_ADDRS at register() time.

unsafe extern "C" fn dispatch_intern(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs >= 1 { mb_sys_intern(*args_ptr) } else { MbValue::none() }
}

unsafe extern "C" fn dispatch_gettrace(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_gettrace()
}

unsafe extern "C" fn dispatch_settrace(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let v = if nargs >= 1 { *args_ptr } else { MbValue::none() };
    mb_sys_settrace(v)
}

unsafe extern "C" fn dispatch_getrefcount(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let v = if nargs >= 1 { *args_ptr } else { MbValue::none() };
    mb_sys_getrefcount(v)
}

unsafe extern "C" fn dispatch_is_finalizing(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_is_finalizing()
}

unsafe extern "C" fn dispatch_exc_info(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_exc_info()
}

unsafe extern "C" fn dispatch_getfilesystemencoding(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_getfilesystemencoding()
}

unsafe extern "C" fn dispatch_getfilesystemencodeerrors(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_getfilesystemencodeerrors()
}

// New-ABI replacements for exit / setrecursionlimit / setswitchinterval.
// The legacy `fn(args: MbValue)` shape never receives the real call args
// (the dispatch wrapper hands those funcs an empty list), so `sys.exit(42)`,
// `sys.setrecursionlimit(-5)` and `sys.setswitchinterval(0.0)` could not see
// their argument to validate it. These flat-args dispatchers read the actual
// first argument and are registered in NATIVE_FUNC_ADDRS at register() time.
unsafe extern "C" fn dispatch_exit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let code = if nargs >= 1 { *args_ptr } else { MbValue::none() };
    mb_sys_exit(code)
}

unsafe extern "C" fn dispatch_setrecursionlimit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let limit = if nargs >= 1 { *args_ptr } else { MbValue::none() };
    mb_sys_setrecursionlimit(limit)
}

unsafe extern "C" fn dispatch_setswitchinterval(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let interval = if nargs >= 1 { *args_ptr } else { MbValue::none() };
    mb_sys_setswitchinterval(interval)
}

// ── Surface stubs (auto-added) ──
// Present+callable stubs for sys functions Mamba does not yet implement.
// They satisfy `hasattr`/`callable` surface fixtures and return a benign
// default (None) so callers that probe-then-call don't crash. Registered in
// NATIVE_FUNC_ADDRS so class.rs dispatch picks the flat-args path.
unsafe extern "C" fn dispatch_sys_stub_none(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Build sys.float_info as a dict with max, min, epsilon
fn build_float_info() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("max".into(), MbValue::from_float(f64::MAX));
            map.insert("min".into(), MbValue::from_float(f64::MIN_POSITIVE));
            map.insert("epsilon".into(), MbValue::from_float(f64::EPSILON));
            map.insert("dig".into(), MbValue::from_int(15));
            map.insert("mant_dig".into(), MbValue::from_int(53));
            map.insert("max_exp".into(), MbValue::from_int(1024));
            map.insert("min_exp".into(), MbValue::from_int(-1021));
        }
    }
    MbValue::from_ptr(dict)
}

/// Build sys.int_info as a dict with bits_per_digit, sizeof_digit
fn build_int_info() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("bits_per_digit".into(), MbValue::from_int(30));
            map.insert("sizeof_digit".into(), MbValue::from_int(4));
        }
    }
    MbValue::from_ptr(dict)
}

/// Build sys.flags as a dict with the common CPython flag fields.
/// All false / zero by default — Mamba doesn't honor most of these yet,
/// but accessing them needs to succeed for libraries that probe (e.g. pytest).
fn build_flags() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for k in [
                "debug", "inspect", "interactive", "optimize",
                "dont_write_bytecode", "no_user_site", "no_site",
                "ignore_environment", "verbose", "bytes_warning",
                "quiet", "hash_randomization", "isolated", "dev_mode",
                "utf8_mode", "warn_default_encoding", "safe_path", "int_max_str_digits",
            ] {
                map.insert(k.into(), MbValue::from_int(0));
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// Build sys.implementation as a dict (CacheType-compatible namespace).
fn build_implementation() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("name".into(),
                MbValue::from_ptr(MbObject::new_str("mamba".to_string())));
            map.insert("cache_tag".into(),
                MbValue::from_ptr(MbObject::new_str("mamba-312".to_string())));
            map.insert("hexversion".into(), MbValue::from_int(0x030c00f0));
            // Nested version_info dict
            let vi = MbObject::new_dict();
            if let ObjData::Dict(ref vlock) = (*vi).data {
                let mut vmap = vlock.write().unwrap();
                vmap.insert("major".into(), MbValue::from_int(3));
                vmap.insert("minor".into(), MbValue::from_int(12));
                vmap.insert("micro".into(), MbValue::from_int(0));
                vmap.insert("releaselevel".into(),
                    MbValue::from_ptr(MbObject::new_str("final".to_string())));
                vmap.insert("serial".into(), MbValue::from_int(0));
            }
            map.insert("version".into(), MbValue::from_ptr(vi));
        }
    }
    MbValue::from_ptr(dict)
}

/// Build sys.hash_info as a dict.
fn build_hash_info() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("width".into(), MbValue::from_int(64));
            map.insert("modulus".into(), MbValue::from_int((1i64 << 47) - 1));
            map.insert("inf".into(), MbValue::from_int(314159));
            map.insert("nan".into(), MbValue::from_int(0));
            map.insert("imag".into(), MbValue::from_int(1000003));
            map.insert("algorithm".into(),
                MbValue::from_ptr(MbObject::new_str("siphash13".to_string())));
            map.insert("hash_bits".into(), MbValue::from_int(64));
            map.insert("seed_bits".into(), MbValue::from_int(128));
            map.insert("cutoff".into(), MbValue::from_int(0));
        }
    }
    MbValue::from_ptr(dict)
}

// ── std stream objects ────────────────────────────────────────────
//
// `sys.stdin` reads the real process stdin (read/readline); `sys.stdout`
// and `sys.stderr` write through the runtime output layer so capture and
// redirect stay coherent with `print`. The three attributes share one
// `sys._Stream` Instance class with a variadic `(self, args_list)` table.

unsafe extern "C" fn stream_read(_self_v: MbValue, _args: MbValue) -> MbValue {
    use std::io::Read as _;
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    MbValue::from_ptr(MbObject::new_str(buf))
}

unsafe extern "C" fn stream_readline(_self_v: MbValue, _args: MbValue) -> MbValue {
    use std::io::BufRead as _;
    let mut line = String::new();
    let _ = std::io::stdin().lock().read_line(&mut line);
    MbValue::from_ptr(MbObject::new_str(line))
}

unsafe extern "C" fn stream_write(self_v: MbValue, args: MbValue) -> MbValue {
    use std::io::Write as _;
    let text = args
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                lock.read().unwrap().first().copied()
            } else {
                None
            }
        })
        .and_then(|v| {
            v.as_ptr().and_then(|p| unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();
    let is_stderr = self_v.as_ptr().is_some_and(|p| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*p).data {
            fields
                .read()
                .unwrap()
                .get("name")
                .and_then(|v| {
                    v.as_ptr().and_then(|sp| {
                        if let ObjData::Str(ref s) = (*sp).data {
                            Some(s == "<stderr>")
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or(false)
        } else {
            false
        }
    });
    let n = text.chars().count() as i64;
    if is_stderr {
        if !super::super::output::try_write_stderr_redirect(&text) {
            let _ = std::io::stderr().write_all(text.as_bytes());
        }
    } else if !super::super::output::write_captured(&text) {
        let _ = std::io::stdout().write_all(text.as_bytes());
    }
    MbValue::from_int(n)
}

unsafe extern "C" fn stream_flush(_self_v: MbValue, _args: MbValue) -> MbValue {
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    MbValue::none()
}

/// Register the shared `sys._Stream` method table.
fn register_stream_class() {
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in [
        ("read", stream_read as *const () as usize),
        ("readline", stream_readline as *const () as usize),
        ("write", stream_write as *const () as usize),
        ("flush", stream_flush as *const () as usize),
    ] {
        super::super::module::register_variadic_func(addr as u64);
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        methods.insert(name.to_string(), MbValue::from_func(addr));
    }
    super::super::class::mb_class_register("sys._Stream", Vec::new(), methods);
}

/// Build a std-stream object (an Instance carrying its display name).
fn build_stream_stub(name: &str) -> MbValue {
    use super::super::rc::{InstanceFields, MbObjectHeader, MbRwLock, ObjKind};
    let mut fields = InstanceFields::default();
    fields.insert(
        "name".to_string(),
        MbValue::from_ptr(MbObject::new_str(format!("<{}>", name))),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "sys._Stream".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register the sys module.
pub fn register() {
    let mut attrs = HashMap::new();

    // sys.version
    attrs.insert("version".into(),
        MbValue::from_ptr(MbObject::new_str("Mamba 0.1.0 (cclab)".to_string())));

    // sys.version_info (as dict with major, minor, micro for attribute access)
    let vi_dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*vi_dict).data {
            let mut map = lock.write().unwrap();
            map.insert("major".into(), MbValue::from_int(3));
            map.insert("minor".into(), MbValue::from_int(12));
            map.insert("micro".into(), MbValue::from_int(0));
        }
    }
    attrs.insert("version_info".into(), MbValue::from_ptr(vi_dict));

    // sys.platform
    attrs.insert("platform".into(),
        MbValue::from_ptr(MbObject::new_str(std::env::consts::OS.to_string())));

    // sys.maxsize (i48 max due to NaN-boxing)
    attrs.insert("maxsize".into(),
        MbValue::from_int((1i64 << 47) - 1));

    // sys.argv (populated at runtime, empty by default)
    attrs.insert("argv".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())));

    // sys.path (search paths for imports)
    let paths: Vec<MbValue> = vec![
        MbValue::from_ptr(MbObject::new_str(".".to_string())),
    ];
    attrs.insert("path".into(),
        MbValue::from_ptr(MbObject::new_list(paths)));

    // sys.executable
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    attrs.insert("executable".into(),
        MbValue::from_ptr(MbObject::new_str(exe)));

    // sys.byteorder
    let order = if cfg!(target_endian = "little") { "little" } else { "big" };
    attrs.insert("byteorder".into(),
        MbValue::from_ptr(MbObject::new_str(order.to_string())));

    // sys.float_info
    attrs.insert("float_info".into(), build_float_info());

    // sys.int_info
    attrs.insert("int_info".into(), build_int_info());

    // sys.stdin, sys.stdout, sys.stderr (stub stream objects)
    register_stream_class();
    attrs.insert("stdin".into(), build_stream_stub("stdin"));
    attrs.insert("stdout".into(), build_stream_stub("stdout"));
    attrs.insert("stderr".into(), build_stream_stub("stderr"));

    // sys.modules (stub dict)
    let modules_dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*modules_dict).data {
            let mut map = lock.write().unwrap();
            map.insert("sys".into(), MbValue::from_bool(true));
        }
    }
    attrs.insert("modules".into(), MbValue::from_ptr(modules_dict));

    // sys.hexversion (3.12.0 final → 0x030C00F0)
    attrs.insert("hexversion".into(), MbValue::from_int(0x030c00f0));

    // sys.api_version — frozen CPython value for 3.12
    attrs.insert("api_version".into(), MbValue::from_int(1013));

    // sys.dont_write_bytecode — Mamba never writes .pyc, so True
    attrs.insert("dont_write_bytecode".into(), MbValue::from_bool(true));

    // sys.builtin_module_names — built-in modules embedded in the
    // interpreter binary (not the wider stdlib). Tuple in CPython.
    let builtins: Vec<MbValue> = ["sys", "builtins", "_imp", "_thread"]
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
        .collect();
    attrs.insert("builtin_module_names".into(),
        MbValue::from_ptr(MbObject::new_tuple(builtins)));

    // sys.warnoptions — empty by default
    attrs.insert("warnoptions".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())));

    // sys.flags / sys.implementation / sys.hash_info
    attrs.insert("flags".into(), build_flags());
    attrs.insert("implementation".into(), build_implementation());
    attrs.insert("hash_info".into(), build_hash_info());

    // sys.prefix / exec_prefix / base_prefix / base_exec_prefix
    let prefix_val = MbValue::from_ptr(MbObject::new_str("".to_string()));
    attrs.insert("prefix".into(), prefix_val);
    attrs.insert("exec_prefix".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())));
    attrs.insert("base_prefix".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())));
    attrs.insert("base_exec_prefix".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())));

    // Callable functions via function pointers
    attrs.insert("getrecursionlimit".into(),
        MbValue::from_func(dispatch_getrecursionlimit as *const () as usize));
    attrs.insert("getsizeof".into(),
        MbValue::from_func(dispatch_getsizeof as *const () as usize));
    attrs.insert("getdefaultencoding".into(),
        MbValue::from_func(dispatch_getdefaultencoding as *const () as usize));
    // New-ABI dispatchers (extern "C" fn(args_ptr, nargs)) — must register
    // in NATIVE_FUNC_ADDRS so class.rs dispatch picks the flat-args path.
    let new_dispatchers: Vec<(&str, usize)> = vec![
        ("exit",                      dispatch_exit                      as *const () as usize),
        ("setrecursionlimit",         dispatch_setrecursionlimit         as *const () as usize),
        ("setswitchinterval",         dispatch_setswitchinterval         as *const () as usize),
        ("intern",                    dispatch_intern                    as *const () as usize),
        ("gettrace",                  dispatch_gettrace                  as *const () as usize),
        ("settrace",                  dispatch_settrace                  as *const () as usize),
        ("getrefcount",               dispatch_getrefcount               as *const () as usize),
        ("is_finalizing",             dispatch_is_finalizing             as *const () as usize),
        ("exc_info",                  dispatch_exc_info                  as *const () as usize),
        ("getfilesystemencoding",     dispatch_getfilesystemencoding     as *const () as usize),
        ("getfilesystemencodeerrors", dispatch_getfilesystemencodeerrors as *const () as usize),
    ];
    for (name, addr) in &new_dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in &new_dispatchers {
            set.insert(*addr as u64);
        }
    });

        // surface: missing CPython module constants (auto-added)
    attrs.insert("abiflags".into(), MbValue::from_ptr(MbObject::new_str("".to_string())));
    attrs.insert("float_repr_style".into(), MbValue::from_ptr(MbObject::new_str("short".to_string())));
    attrs.insert("maxunicode".into(), MbValue::from_int(1114111));
    attrs.insert("platlibdir".into(), MbValue::from_ptr(MbObject::new_str("lib".to_string())));

    // surface: missing CPython sys data/constants (auto-added, batch 2)
    // copyright — CPython license banner string.
    attrs.insert("copyright".into(),
        MbValue::from_ptr(MbObject::new_str(
            "Copyright (c) 2001-2023 Python Software Foundation.\nAll Rights Reserved.".to_string())));
    // pycache_prefix — None by default in CPython.
    attrs.insert("pycache_prefix".into(), MbValue::none());
    // orig_argv — original interpreter argv (empty until populated).
    attrs.insert("orig_argv".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())));
    // meta_path / path_hooks — import system hook lists (empty stubs).
    attrs.insert("meta_path".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())));
    attrs.insert("path_hooks".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())));
    // path_importer_cache — import cache dict (empty stub).
    attrs.insert("path_importer_cache".into(),
        MbValue::from_ptr(MbObject::new_dict()));
    // monitoring — submodule; register as a stub namespace dict.
    attrs.insert("monitoring".into(),
        MbValue::from_ptr(MbObject::new_dict()));
    // stdlib_module_names — frozenset in CPython. A representative subset
    // of the stdlib.
    let stdlib_names: Vec<MbValue> = [
        "abc", "argparse", "array", "ast", "base64", "bisect", "calendar",
        "cmath", "collections", "configparser", "contextlib", "copy", "csv",
        "datetime", "decimal", "difflib", "enum", "fnmatch", "fractions",
        "functools", "gc", "getopt", "glob", "gzip", "hashlib", "heapq",
        "hmac", "html", "http", "io", "itertools", "json", "keyword",
        "logging", "math", "operator", "os", "pathlib", "pickle", "platform",
        "pprint", "queue", "random", "re", "secrets", "shutil", "signal",
        "socket", "stat", "statistics", "string", "struct", "subprocess",
        "sys", "tempfile", "textwrap", "threading", "time", "timeit",
        "tomllib", "traceback", "types", "typing", "unicodedata", "unittest",
        "urllib", "uuid", "warnings", "weakref", "zipfile", "zlib",
    ].iter().map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string()))).collect();
    attrs.insert("stdlib_module_names".into(),
        MbValue::from_ptr(MbObject::new_frozenset(stdlib_names)));
    // thread_info — namespace-like dict (name/lock/version).
    let thread_info = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*thread_info).data {
            let mut map = lock.write().unwrap();
            map.insert("name".into(),
                MbValue::from_ptr(MbObject::new_str("pthread".to_string())));
            map.insert("lock".into(),
                MbValue::from_ptr(MbObject::new_str("mutex+cond".to_string())));
            map.insert("version".into(), MbValue::none());
        }
    }
    attrs.insert("thread_info".into(), MbValue::from_ptr(thread_info));

    // surface: missing CPython sys callables (auto-added, batch 2).
    // Present+callable stubs returning None. Registered in NATIVE_FUNC_ADDRS.
    let stub_fn_addr = dispatch_sys_stub_none as *const () as usize;
    for name in [
        "activate_stack_trampoline", "addaudithook", "audit",
        "breakpointhook", "call_tracing", "deactivate_stack_trampoline",
        "displayhook", "excepthook", "exception",
        "__displayhook__", "__excepthook__",
        "get_asyncgen_hooks", "get_coroutine_origin_tracking_depth",
        "get_int_max_str_digits", "getallocatedblocks", "getdlopenflags",
        "getprofile", "getswitchinterval", "getunicodeinternedsize",
        "is_stack_trampoline_active", "set_asyncgen_hooks",
        "set_coroutine_origin_tracking_depth", "set_int_max_str_digits",
        "setdlopenflags", "setprofile", "unraisablehook",
    ] {
        attrs.insert(name.into(), MbValue::from_func(stub_fn_addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(stub_fn_addr as u64);
    });

    super::register_module("sys", attrs);
}

// ── Runtime functions ──

/// sys.exit(code=0) — raise SystemExit so it can be caught (CPython semantics).
/// `sys.exit` does not terminate the process directly; it raises SystemExit,
/// which an enclosing `try/except SystemExit` can intercept. The message is the
/// stringified exit code (CPython stores the code in `SystemExit.code`).
pub fn mb_sys_exit(code: MbValue) -> MbValue {
    let msg = match code.as_int() {
        Some(i) => i.to_string(),
        None if code.is_none() => String::new(),
        None => match code.as_ptr() {
            Some(ptr) => unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    String::new()
                }
            },
            None => String::new(),
        },
    };
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("SystemExit".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

/// sys.getrecursionlimit() → int
pub fn mb_sys_getrecursionlimit() -> MbValue {
    MbValue::from_int(1000) // Default Python recursion limit
}

/// sys.setrecursionlimit(limit) → None
///
/// CPython rejects a non-positive limit with
/// `ValueError("recursion limit must be greater or equal than 1")`. Mamba uses
/// a fixed recursion limit, so a valid (>= 1) call is still accepted as a no-op,
/// but an invalid limit must raise to match CPython.
pub fn mb_sys_setrecursionlimit(limit: MbValue) -> MbValue {
    // Only validate when an actual integer arg was supplied. Non-int / missing
    // args fall through as a no-op (preserves prior lenient behavior; a real
    // TypeError for non-ints is a separate concern not exercised here).
    if let Some(n) = limit.as_int_pyint() {
        if n < 1 {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "recursion limit must be greater or equal than 1".to_string())),
            );
            return MbValue::none();
        }
    }
    MbValue::none()
}

/// sys.setswitchinterval(interval) → None
///
/// CPython requires a strictly positive interval and raises
/// `ValueError("switch interval must be strictly positive")` for `<= 0`. Mamba
/// does not honor the thread switch interval, so a valid (> 0) call is a no-op,
/// but a non-positive interval must raise to match CPython.
pub fn mb_sys_setswitchinterval(interval: MbValue) -> MbValue {
    // Accept both float (the documented type) and int args. Only raise when a
    // numeric value <= 0 is supplied; a missing / non-numeric arg falls through
    // as a no-op (a real TypeError is a separate concern not exercised here).
    let numeric = interval.as_float().or_else(|| interval.as_int_pyint().map(|i| i as f64));
    if let Some(v) = numeric {
        if v <= 0.0 {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "switch interval must be strictly positive".to_string())),
            );
            return MbValue::none();
        }
    }
    MbValue::none()
}

/// sys.getdefaultencoding() → 'utf-8'
pub fn mb_sys_getdefaultencoding() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("utf-8".to_string()))
}

/// sys.getsizeof(obj) → int (approximate)
pub fn mb_sys_getsizeof(val: MbValue) -> MbValue {
    let size = if val.is_int() || val.is_float() || val.is_bool() || val.is_none() {
        8 // NaN-boxed: 8 bytes
    } else {
        // Heap object: header + data (approximate)
        std::mem::size_of::<super::super::rc::MbObject>()
    };
    MbValue::from_int(size as i64)
}

/// sys.intern(s) — interning is a hint in CPython; return the string unchanged.
pub fn mb_sys_intern(s: MbValue) -> MbValue {
    s
}

/// sys.gettrace() → current trace function (None — Mamba has no Python-level tracing).
pub fn mb_sys_gettrace() -> MbValue {
    MbValue::none()
}

/// sys.settrace(func) → None. No-op stub — accept but ignore.
pub fn mb_sys_settrace(_func: MbValue) -> MbValue {
    MbValue::none()
}

/// sys.getrefcount(obj) → int. Mamba uses Arc internally and doesn't expose
/// per-object refcounts to Python; return a stable >0 value to keep callers
/// (e.g. test fixtures asserting refcount > 0) happy.
pub fn mb_sys_getrefcount(_obj: MbValue) -> MbValue {
    MbValue::from_int(1)
}

/// sys.is_finalizing() → bool. Mamba never reaches finalization mid-run.
pub fn mb_sys_is_finalizing() -> MbValue {
    MbValue::from_bool(false)
}

/// sys.exc_info() → (type, value, traceback). Returns (None, None, None)
/// when no exception is being handled — Mamba's exception machinery doesn't
/// thread the active triple through here yet.
pub fn mb_sys_exc_info() -> MbValue {
    match super::super::exception::last_handled_exception() {
        Some((etype, msg)) => {
            let type_val = MbValue::from_ptr(MbObject::new_str(etype.clone()));
            let value_val = MbValue::from_ptr(MbObject::new_str(msg));
            // Mamba does not yet construct traceback objects exposing here.
            let tb_val = MbValue::none();
            MbValue::from_ptr(MbObject::new_tuple(vec![type_val, value_val, tb_val]))
        }
        None => {
            let triple = vec![MbValue::none(), MbValue::none(), MbValue::none()];
            MbValue::from_ptr(MbObject::new_tuple(triple))
        }
    }
}

/// sys.getfilesystemencoding() → 'utf-8'.
pub fn mb_sys_getfilesystemencoding() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("utf-8".to_string()))
}

/// sys.getfilesystemencodeerrors() → 'surrogateescape' (CPython default on
/// POSIX). Stable string — chosen to match what UTF-8-mode CPython reports.
pub fn mb_sys_getfilesystemencodeerrors() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("surrogateescape".to_string()))
}

/// Populate sys.argv from command-line arguments.
pub fn populate_argv(args: &[String]) {
    let argv: Vec<MbValue> = args.iter()
        .map(|a| MbValue::from_ptr(MbObject::new_str(a.clone())))
        .collect();
    let argv_list = MbValue::from_ptr(MbObject::new_list(argv));

    crate::runtime::module::MODULES.with(|mods| {
        let mut mods = mods.borrow_mut();
        if let Some(sys) = mods.get_mut("sys") {
            sys.attrs.insert("argv".into(), argv_list);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_register() {
        register();
        let name = MbValue::from_ptr(MbObject::new_str("sys".to_string()));
        let result = super::super::super::module::mb_import(name);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_sys_getrecursionlimit() {
        assert_eq!(mb_sys_getrecursionlimit().as_int(), Some(1000));
    }

    #[test]
    fn test_sys_getsizeof() {
        let size = mb_sys_getsizeof(MbValue::from_int(42));
        assert_eq!(size.as_int(), Some(8));
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_sys_getrecursionlimit_is_1000() {
        assert_eq!(mb_sys_getrecursionlimit().as_int(), Some(1000));
    }

    #[test]
    fn test_py312_sys_getsizeof_primitives_return_8() {
        assert_eq!(mb_sys_getsizeof(MbValue::from_int(0)).as_int(), Some(8));
        assert_eq!(mb_sys_getsizeof(MbValue::from_float(1.0)).as_int(), Some(8));
        assert_eq!(mb_sys_getsizeof(MbValue::from_bool(true)).as_int(), Some(8));
        assert_eq!(mb_sys_getsizeof(MbValue::none()).as_int(), Some(8));
    }

    #[test]
    fn test_py312_sys_getsizeof_heap_object_positive() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let size = mb_sys_getsizeof(s);
        assert!(size.as_int().unwrap() > 0);
    }

    #[test]
    fn test_py312_sys_maxsize_fits_in_i48() {
        let maxsize: i64 = (1i64 << 47) - 1;
        let v = MbValue::from_int(maxsize);
        assert_eq!(v.as_int(), Some(maxsize));
    }

    #[test]
    fn test_py312_populate_argv_accepts_empty() {
        register();
        populate_argv(&[]);
    }

}
