//! subprocess module for Mamba (#397, #1265 Task #82, Wave-9).
//!
//! CPython 3.12 `subprocess` 30-entry surface:
//!   CalledProcessError, CompletedProcess, DEVNULL, PIPE, Popen, STDOUT,
//!   SubprocessError, TimeoutExpired, builtins, call, check_call,
//!   check_output, contextlib, errno, fcntl, getoutput, getstatusoutput,
//!   io, list2cmdline, locale, os, run, select, selectors, signal, sys,
//!   threading, time, types, warnings.
//!
//! Carve-outs:
//!   - `Popen(...)` runs the command synchronously to completion and
//!     stores `returncode` plus the captured streams; `communicate()`,
//!     `wait()`, `poll()`, `kill()`/`terminate()`/`send_signal()` and the
//!     context-manager dunders dispatch through the registered Popen
//!     class. `Popen.stdout`/`stderr` are in-memory pipe stream stubs when
//!     those selectors are `PIPE`; `stdin` remains a deferred live pipe gap.
//!   - `shell=True` keyword argument is not parsed. The argv-list shape
//!     (`run(["cmd", "arg"])`) and the whitespace-split string shape
//!     (`run("cmd arg")`) are both honored, mirroring the existing
//!     `extract_args` contract.
//!   - `run` / `check_output` parse `env`, `cwd`, `input`, `check`,
//!     `capture_output`, `text`/`encoding`/`universal_newlines`,
//!     `stdout`/`stderr` selectors (PIPE/STDOUT/DEVNULL), and `timeout`
//!     (TimeoutExpired) through the shared spawn engine. CPython's
//!     no-capture default is honored: un-captured CompletedProcess
//!     streams are None; `check_output` returns the child stdout as
//!     `bytes`, matching CPython's no-`text=` default.
//!   - Error reporting matches CPython's OSError family: spawning a
//!     missing/unexecutable command raises `FileNotFoundError` /
//!     `PermissionError` / `NotADirectoryError`; a NUL byte in argv raises
//!     `ValueError`; a non-integer `bufsize` raises `TypeError`. A
//!     non-zero exit from `check_call` / `check_output` raises a real
//!     `CalledProcessError` *instance* carrying `returncode`, `cmd`,
//!     `output`/`stdout`, and `stderr`, raised via `mb_raise_instance` so
//!     the value survives into the `except ... as exc:` binding.
//!   - `isinstance(x, subprocess.CompletedProcess)` (and the other class
//!     dispatchers) resolve via `NATIVE_TYPE_NAMES`, mapping the dispatcher
//!     function pointer back to the instance's `class_name`.
//!   - Known gaps (require runtime changes outside this shim, NOT in scope):
//!       * `except subprocess.CalledProcessError as exc:` does not match —
//!         `mb_exception_matches` / `exception.rs::extract_str` only resolve
//!         a *string* except-type, not a native dispatcher func pointer, so
//!         the raised `CalledProcessError` is not caught by name (catching
//!         the builtin family — `ValueError`, `TypeError`,
//!         `FileNotFoundError` — works). Lane-B: needs `exception.rs`.
//!       * `Popen.communicate()` / `.wait()` / `.poll()` are instance
//!         methods on a native class name; dispatching them requires a
//!         `class.rs` branch. Lane-B.
//!       * Fixtures that re-exec the interpreter via
//!         `[sys.executable, "-c", code]` need mamba's CLI to accept `-c`
//!         (main.rs), independent of this module.
//!   - `getoutput` returns stdout only; `getstatusoutput` returns a
//!     2-tuple `(status, output)` where `output` is stdout+stderr joined.
//!     Both shell out via `/bin/sh -c` on Unix to match CPython.
//!   - `list2cmdline` joins args with spaces, quoting any arg containing
//!     whitespace or shell metacharacters. This is a simplified form of
//!     CPython's Windows-specific quoting rules and is sufficient for
//!     round-trip logging on POSIX.
//!   - Module re-exports (`builtins`, `contextlib`, `errno`, `fcntl`,
//!     `io`, `locale`, `os`, `select`, `selectors`, `signal`, `sys`,
//!     `threading`, `time`, `types`, `warnings`): exposed as
//!     `MbValue::none()` placeholders. CPython's `subprocess` imports
//!     these for internal use only; user code that does
//!     `import subprocess; subprocess.os` is rare enough to defer to a
//!     proper module-aliasing pass.

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

// -- Variadic dispatchers --

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

/// Dispatcher variant that forwards the *full* positional slice — used by
/// the exception constructors (`CalledProcessError(returncode, cmd, ...)`)
/// which must populate multiple positional fields.
macro_rules! disp_variadic_all {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

disp_variadic_all!(dispatch_run, mb_subprocess_run_all);
disp_variadic!(dispatch_call, mb_subprocess_call);
disp_variadic_all!(dispatch_check_output, mb_subprocess_check_output_all);
disp_variadic!(dispatch_check_call, mb_subprocess_check_call);
disp_variadic!(dispatch_getoutput, mb_subprocess_getoutput);
disp_variadic!(dispatch_getstatusoutput, mb_subprocess_getstatusoutput);
disp_variadic!(dispatch_list2cmdline, mb_subprocess_list2cmdline);
disp_variadic_all!(dispatch_popen, mb_subprocess_popen_impl);
disp_variadic_all!(
    dispatch_completed_process,
    mb_subprocess_completed_process_new
);
disp_variadic_all!(
    dispatch_called_process_error,
    mb_subprocess_called_process_error_new
);
disp_variadic_all!(dispatch_timeout_expired, mb_subprocess_timeout_expired_new);
disp_variadic_all!(
    dispatch_subprocess_error,
    mb_subprocess_subprocess_error_new
);

/// Register the subprocess module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("run", dispatch_run as *const () as usize),
        ("call", dispatch_call as *const () as usize),
        ("check_output", dispatch_check_output as *const () as usize),
        ("check_call", dispatch_check_call as *const () as usize),
        ("getoutput", dispatch_getoutput as *const () as usize),
        (
            "getstatusoutput",
            dispatch_getstatusoutput as *const () as usize,
        ),
        ("list2cmdline", dispatch_list2cmdline as *const () as usize),
        ("Popen", dispatch_popen as *const () as usize),
        (
            "CompletedProcess",
            dispatch_completed_process as *const () as usize,
        ),
        (
            "CalledProcessError",
            dispatch_called_process_error as *const () as usize,
        ),
        (
            "TimeoutExpired",
            dispatch_timeout_expired as *const () as usize,
        ),
        (
            "SubprocessError",
            dispatch_subprocess_error as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Register the class-like dispatchers as native type names so
    // `isinstance(x, subprocess.CompletedProcess)` and
    // `except subprocess.CalledProcessError` resolve the dispatcher
    // function pointer back to the instance's `class_name`.
    let class_dispatchers: Vec<(&str, usize)> = vec![
        (
            "CompletedProcess",
            dispatch_completed_process as *const () as usize,
        ),
        ("Popen", dispatch_popen as *const () as usize),
        (
            "CalledProcessError",
            dispatch_called_process_error as *const () as usize,
        ),
        (
            "TimeoutExpired",
            dispatch_timeout_expired as *const () as usize,
        ),
        (
            "SubprocessError",
            dispatch_subprocess_error as *const () as usize,
        ),
    ];
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        for (name, addr) in &class_dispatchers {
            map.insert(*addr as u64, name.to_string());
        }
    });

    // Register the "Popen" instance-method class so `p.wait()` dispatches
    // through the normal MRO path (mirrors configparser's
    // `register_method_class`). The constructor func `Popen` (a module attr)
    // and this CLASS_REGISTRY entry of the same name coexist: `Popen(...)`
    // calls the dispatcher, while `p.wait()` resolves `wait` by the
    // instance's `class_name`. `wait` is variadic — its packed positional
    // list (`[timeout]`) is accepted and ignored.
    {
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in [
            ("wait", popen_wait as *const () as usize),
            ("communicate", popen_communicate as *const () as usize),
            ("poll", popen_poll as *const () as usize),
            ("kill", popen_kill as *const () as usize),
            ("terminate", popen_kill as *const () as usize),
            ("send_signal", popen_kill as *const () as usize),
            ("__enter__", popen_enter as *const () as usize),
            ("__exit__", popen_exit as *const () as usize),
        ] {
            super::super::module::register_variadic_func(addr as u64);
            super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
                s.borrow_mut().insert(addr as u64);
            });
            methods.insert(name.to_string(), MbValue::from_func(addr));
        }
        super::super::class::mb_class_register("Popen", Vec::new(), methods);
    }

    // CompletedProcess.check_returncode() — raises CalledProcessError when
    // the recorded returncode is non-zero, mirrors CPython.
    {
        let addr = completed_check_returncode as *const () as usize;
        super::super::module::register_variadic_func(addr as u64);
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        methods.insert("check_returncode".to_string(), MbValue::from_func(addr));
        super::super::class::mb_class_register("CompletedProcess", Vec::new(), methods);
    }

    // Integer constants — match CPython's negative sentinels.
    attrs.insert("PIPE".into(), MbValue::from_int(-1));
    attrs.insert("STDOUT".into(), MbValue::from_int(-2));
    attrs.insert("DEVNULL".into(), MbValue::from_int(-3));

    // `__all__` — CPython's subprocess public API. Deliberately omits the
    // low-level `list2cmdline` helper, matching CPython 3.12 exactly.
    let all_names = [
        "Popen",
        "PIPE",
        "STDOUT",
        "call",
        "check_call",
        "getstatusoutput",
        "getoutput",
        "check_output",
        "run",
        "CalledProcessError",
        "DEVNULL",
        "SubprocessError",
        "TimeoutExpired",
        "CompletedProcess",
    ];
    let all_list: Vec<MbValue> = all_names
        .iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string())))
        .collect();
    attrs.insert(
        "__all__".into(),
        MbValue::from_ptr(MbObject::new_list(all_list)),
    );
    attrs.insert(
        "__name__".into(),
        MbValue::from_ptr(MbObject::new_str("subprocess".to_string())),
    );

    // Module re-exports — placeholders for `dir(subprocess)` parity.
    for sub in [
        "builtins",
        "contextlib",
        "errno",
        "fcntl",
        "io",
        "locale",
        "os",
        "select",
        "selectors",
        "signal",
        "sys",
        "threading",
        "time",
        "types",
        "warnings",
    ] {
        attrs.insert(sub.to_string(), MbValue::none());
    }

    super::register_module("subprocess", attrs);
}

// -- Helpers --

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// CPython type name for an arbitrary `MbValue`, used to build the
/// `expected str, bytes or os.PathLike object, not <type>` TypeError that
/// `extract_args` raises on a non-string argv element.
fn py_type_name(val: MbValue) -> String {
    if val.is_none() {
        return "NoneType".to_string();
    }
    if val.is_bool() {
        return "bool".to_string();
    }
    if val.is_int() {
        return "int".to_string();
    }
    if val.is_float() {
        return "float".to_string();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::BigInt(_) => "int",
                ObjData::Complex(_, _) => "complex",
                ObjData::CodeObject { .. } => "code",
                ObjData::Instance { class_name, .. } => class_name.as_str(),
            }
            .to_string();
        }
    }
    "object".to_string()
}

/// Convert a single argv element to its string token. CPython accepts only
/// `str`, `bytes`, or `os.PathLike`; any other element raises
/// `TypeError("expected str, bytes or os.PathLike object, not <type>")`.
/// On a type error this sets the pending exception and returns `None`; the
/// caller must check `extract_args`'s `Result`.
fn extract_arg_token(val: MbValue) -> Result<String, ()> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => return Ok(s.clone()),
                ObjData::Bytes(b) => {
                    return Ok(String::from_utf8_lossy(b).to_string());
                }
                // os.PathLike: an instance defining __fspath__ — decode by
                // calling it (CPython's os.fspath path).
                ObjData::Instance { ref class_name, .. } => {
                    if !super::super::class::lookup_method(class_name, "__fspath__").is_none() {
                        let mname = MbValue::from_ptr(MbObject::new_str("__fspath__".to_string()));
                        let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
                        let r = super::super::class::mb_call_method(val, mname, empty);
                        if let Some(s) = extract_str(r) {
                            return Ok(s);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    raise(
        "TypeError",
        &format!(
            "expected str, bytes or os.PathLike object, not {}",
            py_type_name(val)
        ),
    );
    Err(())
}

/// Coerce an MbValue into a Vec<String> of argv tokens. Accepts:
///   - a heap string: split on whitespace
///   - a list/tuple of str/bytes elements: take each element
///
/// A non-str/bytes element of a list/tuple raises `TypeError` (matching
/// CPython, which rejects e.g. `[1, 2]` argv) and returns `Err(())`; the
/// pending exception is set, so callers must propagate `None` on `Err`.
fn extract_args(val: MbValue) -> Result<Vec<String>, ()> {
    if let Some(s) = extract_str(val) {
        return Ok(s.split_whitespace().map(|w| w.to_string()).collect());
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let mut out = Vec::new();
                    for v in lock.read().unwrap().iter() {
                        out.push(extract_arg_token(*v)?);
                    }
                    return Ok(out);
                }
                ObjData::Tuple(items) => {
                    let mut out = Vec::new();
                    for v in items.iter() {
                        out.push(extract_arg_token(*v)?);
                    }
                    return Ok(out);
                }
                _ => {}
            }
        }
    }
    Ok(vec![])
}

/// True iff `dict_val` is an env mapping carrying an embedded NUL byte in any
/// variable name (a `DictKey::Str` key) or string value. CPython rejects such
/// an env with `ValueError("embedded null byte")` *before* spawning the child,
/// because the C-level environ encoding forbids NUL. A legitimate env name or
/// value never contains NUL, so this fires only on genuinely invalid input.
fn env_dict_has_nul(dict_val: MbValue) -> bool {
    let Some(ptr) = dict_val.as_ptr() else {
        return false;
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            if let Ok(map) = lock.read() {
                for (k, v) in map.iter() {
                    if let super::super::dict_ops::DictKey::Str(name) = k {
                        if name.contains('\0') {
                            return true;
                        }
                    }
                    if let Some(s) = extract_str(*v) {
                        if s.contains('\0') {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Scan a native-call argument slice for an `env=` keyword whose mapping carries
/// an embedded NUL byte, mirroring CPython's pre-spawn validation. mamba's call
/// lowering folds keyword arguments into a single trailing dict positional
/// (`{'env': {...}}`), so the env mapping arrives nested under the `env` key of
/// a trailing kwargs dict. Returns `true` only when such an env mapping has a
/// NUL in a variable name or value — never on a valid `env=`.
fn args_have_nul_env(a: &[MbValue]) -> bool {
    for extra in a.iter().skip(1) {
        let Some(ptr) = extra.as_ptr() else {
            continue;
        };
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                if let Ok(map) = lock.read() {
                    let env_key = super::super::dict_ops::DictKey::Str("env".to_string());
                    if let Some(env_val) = map.get(&env_key).copied() {
                        if env_dict_has_nul(env_val) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Retain a heap value before storing it in a second slot (e.g. aliasing
/// `output` into both `output` and `stdout`). No-op for inline values.
fn retain(val: MbValue) {
    unsafe {
        super::super::rc::retain_if_ptr(val);
    }
}

fn new_instance_with_fields(class_name: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn pipe_stream_value(data: Vec<u8>, captured: bool, text: bool) -> MbValue {
    if !captured {
        return MbValue::none();
    }
    if text {
        return super::io_mod::mb_stringio_new_with(
            String::from_utf8_lossy(&data).replace("\r\n", "\n"),
        );
    }
    super::io_mod::mb_bytesio_new_with(data)
}

fn close_stream_value(stream: MbValue) {
    let Some(ptr) = stream.as_ptr() else {
        return;
    };
    unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            match class_name.as_str() {
                "BytesIO" => {
                    super::io_mod::mb_bytesio_close(stream);
                }
                "StringIO" => {
                    super::io_mod::mb_stringio_close(stream);
                }
                _ => {}
            }
        }
    }
}

/// Build a CompletedProcess Instance — `returncode`, `stdout`, `stderr`,
/// and `args`. Used as the public return shape of `run`.
fn make_completed_process(
    args_repr: MbValue,
    returncode: i32,
    stdout: &str,
    stderr: &str,
) -> MbValue {
    let mut f = FxHashMap::default();
    f.insert("args".into(), args_repr);
    f.insert("returncode".into(), MbValue::from_int(returncode as i64));
    f.insert(
        "stdout".into(),
        MbValue::from_ptr(MbObject::new_str(stdout.to_string())),
    );
    f.insert(
        "stderr".into(),
        MbValue::from_ptr(MbObject::new_str(stderr.to_string())),
    );
    new_instance_with_fields("CompletedProcess", f)
}

// -- Runtime functions --

/// Raise a catchable exception by string type name, returning None.
fn raise(exc_type: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc_type.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Map a `std::io::Error` from a failed `Command::output()`/`status()` to the
/// CPython exception type name that spawning a missing/unexecutable command
/// raises (the OSError family surfaced by the exec failing in the child).
fn spawn_error_type(e: &std::io::Error) -> &'static str {
    use std::io::ErrorKind;
    match e.kind() {
        ErrorKind::NotFound => "FileNotFoundError",
        ErrorKind::PermissionDenied => "PermissionError",
        // A path component that is not a directory (ENOTDIR).
        _ if e.raw_os_error() == Some(20) => "NotADirectoryError",
        _ => "OSError",
    }
}

/// CPython's `strerror`-style text for the errno carried by a failed exec.
/// CPython formats the OSError as `[Errno N] <strerror>: '<filename>'`, so the
/// human text must match the selected errno rather than always claiming
/// "No such file or directory" (ENOENT). Covers the spawn errno set this
/// module surfaces (ENOENT=2, EACCES=13, ENOTDIR=20).
fn spawn_error_strerror(e: &std::io::Error) -> &'static str {
    use std::io::ErrorKind;
    match e.kind() {
        ErrorKind::NotFound => "No such file or directory",
        ErrorKind::PermissionDenied => "Permission denied",
        _ if e.raw_os_error() == Some(20) => "Not a directory",
        _ => "No such file or directory",
    }
}

/// Build the CPython `[Errno N] <strerror>: '<filename>'` OSError message for a
/// failed exec, picking the errno-appropriate strerror text rather than
/// hardcoding ENOENT's "No such file or directory".
fn spawn_error_message(e: &std::io::Error, filename: &str) -> String {
    format!(
        "[Errno {}] {}: '{}'",
        e.raw_os_error().unwrap_or(2),
        spawn_error_strerror(e),
        filename
    )
}

/// Compute a CPython-style `returncode` from a finished `ExitStatus`.
///
/// CPython reports a child terminated by signal N as a *negative* returncode
/// (`-N`). On Unix, `ExitStatus::code()` is `None` exactly when the child was
/// signal-killed, so fall back to `ExitStatusExt::signal()` and negate it.
fn returncode_of(status: &std::process::ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return -sig;
        }
    }
    -1
}

/// Keyword options the `run`/`check_*` family accepts; parsed from the
/// trailing kwargs dict the call lowering appends.
#[derive(Default)]
struct SpawnOpts {
    env: Option<Vec<(String, String)>>,
    cwd: Option<String>,
    input: Option<Vec<u8>>,
    check: bool,
    capture_output: bool,
    text: bool,
    /// stdout/stderr selectors: 0 inherit, -1 PIPE, -2 STDOUT, -3 DEVNULL.
    stdout_sel: i64,
    stderr_sel: i64,
    timeout: Option<f64>,
}

fn kwargs_dict(a: &[MbValue]) -> Option<MbValue> {
    let last = a.last()?;
    let ptr = last.as_ptr()?;
    unsafe {
        if matches!((*ptr).data, ObjData::Dict(_)) {
            Some(*last)
        } else {
            None
        }
    }
}

fn dict_get(d: MbValue, key: &str) -> Option<MbValue> {
    use super::super::dict_ops::DictKey;
    let ptr = d.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read()
                .unwrap()
                .get(&DictKey::Str(key.to_string()))
                .copied()
        } else {
            None
        }
    }
}

fn parse_spawn_opts(a: &[MbValue]) -> SpawnOpts {
    let mut opts = SpawnOpts::default();
    let Some(kw) = kwargs_dict(a) else {
        return opts;
    };
    if let Some(env) = dict_get(kw, "env") {
        if let Some(ptr) = env.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    use super::super::dict_ops::DictKey;
                    let mut pairs = Vec::new();
                    for (k, v) in lock.read().unwrap().iter() {
                        if let (DictKey::Str(name), Some(val)) = (k, extract_str(*v)) {
                            pairs.push((name.clone(), val));
                        }
                    }
                    opts.env = Some(pairs);
                }
            }
        }
    }
    opts.cwd = dict_get(kw, "cwd").and_then(extract_str);
    opts.input = dict_get(kw, "input").and_then(|v| {
        if let Some(s) = extract_str(v) {
            return Some(s.into_bytes());
        }
        v.as_ptr().and_then(|p| unsafe {
            match (*p).data {
                ObjData::Bytes(ref b) => Some(b.clone()),
                ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().clone()),
                _ => None,
            }
        })
    });
    opts.check = dict_get(kw, "check").and_then(|v| v.as_bool()) == Some(true);
    opts.capture_output = dict_get(kw, "capture_output").and_then(|v| v.as_bool()) == Some(true);
    opts.text = dict_get(kw, "text").and_then(|v| v.as_bool()) == Some(true)
        || dict_get(kw, "encoding").is_some()
        || dict_get(kw, "universal_newlines").and_then(|v| v.as_bool()) == Some(true);
    opts.stdout_sel = dict_get(kw, "stdout").and_then(|v| v.as_int()).unwrap_or(0);
    opts.stderr_sel = dict_get(kw, "stderr").and_then(|v| v.as_int()).unwrap_or(0);
    opts.timeout =
        dict_get(kw, "timeout").and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)));
    opts
}

/// Spawn engine shared by run/call/check_call/check_output: applies
/// env/cwd/stdin wiring, enforces `timeout` (TimeoutExpired), and returns
/// `(returncode, stdout, stderr)`. `Err(())` means an exception was raised.
fn spawn_with_opts(
    cmd_args: &[String],
    opts: &SpawnOpts,
    capture: bool,
) -> Result<(i32, Vec<u8>, Vec<u8>), ()> {
    use std::process::Stdio;

    let mut cmd = std::process::Command::new(&cmd_args[0]);
    cmd.args(&cmd_args[1..]);
    if let Some(ref pairs) = opts.env {
        cmd.env_clear();
        cmd.envs(pairs.iter().map(|(k, v)| (k, v)));
    }
    if let Some(ref cwd) = opts.cwd {
        cmd.current_dir(cwd);
    }
    cmd.stdin(if opts.input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let want_stdout = capture || opts.capture_output || opts.stdout_sel == -1;
    let want_stderr = opts.capture_output || opts.stderr_sel == -1 || opts.stderr_sel == -2;
    cmd.stdout(match opts.stdout_sel {
        -3 => Stdio::null(),
        _ if want_stdout => Stdio::piped(),
        _ => Stdio::inherit(),
    });
    cmd.stderr(match opts.stderr_sel {
        -3 => Stdio::null(),
        _ if want_stderr => Stdio::piped(),
        _ => Stdio::inherit(),
    });

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            raise(spawn_error_type(&e), &spawn_error_message(&e, &cmd_args[0]));
            return Err(());
        }
    };

    if let Some(ref input) = opts.input {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write as _;
            let _ = stdin.write_all(input);
            // Drop closes the pipe so the child sees EOF.
        }
    }

    // Drain pipes on threads so a chatty child can't deadlock against an
    // un-read pipe while we poll for exit.
    let out_handle = child.stdout.take().map(|mut s| {
        std::thread::spawn(move || {
            use std::io::Read as _;
            let mut buf = Vec::new();
            let _ = s.read_to_end(&mut buf);
            buf
        })
    });
    let err_handle = child.stderr.take().map(|mut s| {
        std::thread::spawn(move || {
            use std::io::Read as _;
            let mut buf = Vec::new();
            let _ = s.read_to_end(&mut buf);
            buf
        })
    });

    let status = if let Some(secs) = opts.timeout {
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs_f64(secs.max(0.0));
        loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        raise(
                            "TimeoutExpired",
                            &format!(
                                "Command '{}' timed out after {} seconds",
                                cmd_args.join(" "),
                                secs
                            ),
                        );
                        return Err(());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => {
                    raise("OSError", &format!("wait failed: {e}"));
                    return Err(());
                }
            }
        }
    } else {
        match child.wait() {
            Ok(s) => s,
            Err(e) => {
                raise("OSError", &format!("wait failed: {e}"));
                return Err(());
            }
        }
    };

    let mut stdout = out_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let stderr = err_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    // stderr=subprocess.STDOUT folds the error stream into stdout.
    let stderr = if opts.stderr_sel == -2 {
        stdout.extend_from_slice(&stderr);
        Vec::new()
    } else {
        stderr
    };

    Ok((returncode_of(&status), stdout, stderr))
}

fn stream_value(data: Vec<u8>, captured: bool, text: bool) -> MbValue {
    if !captured {
        return MbValue::none();
    }
    if text {
        MbValue::from_ptr(MbObject::new_str(
            String::from_utf8_lossy(&data).replace("\r\n", "\n"),
        ))
    } else {
        MbValue::from_ptr(MbObject::new_bytes(data))
    }
}

/// subprocess.run(args, **kwargs) -> CompletedProcess instance
pub fn mb_subprocess_run(args: MbValue) -> MbValue {
    mb_subprocess_run_all(&[args])
}

/// Full-signature `subprocess.run` (capture_output / text / env / cwd /
/// input / check / stdout / stderr / timeout).
pub fn mb_subprocess_run_all(a: &[MbValue]) -> MbValue {
    let args = a.first().copied().unwrap_or_else(MbValue::none);
    let Ok(cmd_args) = extract_args(args) else {
        // A non-str/bytes argv element raised TypeError inside extract_args.
        return MbValue::none();
    };
    if cmd_args.is_empty() {
        // CPython raises IndexError on `run([])` ("index out of range").
        return raise("IndexError", "list index out of range");
    }
    if cmd_args.iter().any(|t| t.contains('\0')) {
        return raise("ValueError", "embedded null byte");
    }

    let opts = parse_spawn_opts(a);
    let Ok((code, stdout, stderr)) = spawn_with_opts(&cmd_args, &opts, false) else {
        return MbValue::none();
    };

    let out_captured = opts.capture_output || opts.stdout_sel == -1;
    let err_captured = opts.capture_output || opts.stderr_sel == -1;

    if opts.check && code != 0 {
        // Raise a CalledProcessError *instance* carrying returncode / cmd /
        // output / stderr so `except CalledProcessError as exc:` can read
        // exc.returncode (a bare string exception left it None).
        let out_v = stream_value(stdout, out_captured, opts.text);
        let err_v = stream_value(stderr, err_captured, opts.text);
        return raise_called_process_error(code, &cmd_args, out_v, err_v);
    }

    let mut f = FxHashMap::default();
    f.insert("args".into(), args);
    f.insert("returncode".into(), MbValue::from_int(code as i64));
    f.insert(
        "stdout".into(),
        stream_value(stdout, out_captured, opts.text),
    );
    f.insert(
        "stderr".into(),
        stream_value(stderr, err_captured, opts.text),
    );
    new_instance_with_fields("CompletedProcess", f)
}

/// subprocess.call(args) -> returncode (int)
pub fn mb_subprocess_call(args: MbValue) -> MbValue {
    let Ok(cmd_args) = extract_args(args) else {
        // A non-str/bytes argv element raised TypeError inside extract_args.
        return MbValue::none();
    };
    if cmd_args.is_empty() {
        // CPython's _execute_child does `args[0]`, raising IndexError on `[]`.
        return raise("IndexError", "list index out of range");
    }
    if cmd_args.iter().any(|t| t.contains('\0')) {
        return raise("ValueError", "embedded null byte");
    }

    let result = std::process::Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .status();

    match result {
        Ok(status) => MbValue::from_int(returncode_of(&status) as i64),
        Err(e) => raise(spawn_error_type(&e), &spawn_error_message(&e, &cmd_args[0])),
    }
}

/// subprocess.check_output(args) -> stdout (bytes by default)
///
/// CPython returns the child's stdout as `bytes` (or `str` under
/// `text=True`, which the variadic dispatcher does not yet thread through),
/// and raises `CalledProcessError` on a non-zero exit. A missing command
/// raises the OSError family from the failed exec.
pub fn mb_subprocess_check_output(args: MbValue) -> MbValue {
    mb_subprocess_check_output_all(&[args])
}

/// Full-signature `check_output` (input / stderr=STDOUT / env / cwd / text /
/// timeout) on the shared spawn engine.
pub fn mb_subprocess_check_output_all(a: &[MbValue]) -> MbValue {
    let args = a.first().copied().unwrap_or_else(MbValue::none);
    let Ok(cmd_args) = extract_args(args) else {
        // A non-str/bytes argv element raised TypeError inside extract_args.
        return MbValue::none();
    };
    if cmd_args.is_empty() {
        return raise("IndexError", "list index out of range");
    }
    if cmd_args.iter().any(|t| t.contains('\0')) {
        return raise("ValueError", "embedded null byte");
    }

    let opts = parse_spawn_opts(a);
    let Ok((code, stdout, stderr)) = spawn_with_opts(&cmd_args, &opts, true) else {
        return MbValue::none();
    };
    if code == 0 {
        return stream_value(stdout, true, opts.text);
    }
    let out = MbValue::from_ptr(MbObject::new_bytes(stdout));
    let err = MbValue::from_ptr(MbObject::new_bytes(stderr));
    raise_called_process_error(code, &cmd_args, out, err)
}

/// subprocess.check_call(args) -> 0 or raises CalledProcessError
pub fn mb_subprocess_check_call(args: MbValue) -> MbValue {
    let Ok(cmd_args) = extract_args(args) else {
        // A non-str/bytes argv element raised TypeError inside extract_args.
        return MbValue::none();
    };
    if cmd_args.is_empty() {
        return raise("IndexError", "list index out of range");
    }
    if cmd_args.iter().any(|t| t.contains('\0')) {
        return raise("ValueError", "embedded null byte");
    }

    let result = std::process::Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .status();

    match result {
        Ok(status) if status.success() => MbValue::from_int(0),
        Ok(status) => {
            let code = returncode_of(&status);
            raise_called_process_error(code, &cmd_args, MbValue::none(), MbValue::none())
        }
        Err(e) => raise(spawn_error_type(&e), &spawn_error_message(&e, &cmd_args[0])),
    }
}

/// Raise a catchable `CalledProcessError` *instance* carrying `returncode`,
/// `cmd`, `output`, and `stderr` so user `except subprocess.CalledProcessError
/// as exc:` handlers can read `exc.returncode`. Returns None (the convention
/// for native functions that raise).
fn raise_called_process_error(
    returncode: i32,
    cmd_args: &[String],
    output: MbValue,
    stderr: MbValue,
) -> MbValue {
    let cmd_list: Vec<MbValue> = cmd_args
        .iter()
        .map(|a| MbValue::from_ptr(MbObject::new_str(a.clone())))
        .collect();
    let cmd_val = MbValue::from_ptr(MbObject::new_list(cmd_list));

    let mut f = FxHashMap::default();
    f.insert("returncode".into(), MbValue::from_int(returncode as i64));
    f.insert("cmd".into(), cmd_val);
    // `output` is aliased into both `output` and `stdout`; retain once for
    // the second slot so the shared heap value is not released twice.
    f.insert("output".into(), output);
    retain(output);
    f.insert("stdout".into(), output);
    f.insert("stderr".into(), stderr);
    let msg = format!(
        "Command '{}' returned non-zero exit status {}.",
        cmd_args.join(" "),
        returncode
    );
    f.insert(
        "message".into(),
        MbValue::from_ptr(MbObject::new_str(msg.clone())),
    );
    // `args` is the Exception base's positional tuple; CPython's str(exc)
    // uses the formatted message, so store that as the single arg.
    f.insert(
        "args".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str(msg),
        )])),
    );

    let inst = new_instance_with_fields("CalledProcessError", f);
    super::super::class::mb_raise_instance(inst);
    MbValue::none()
}

/// subprocess.getoutput(cmd) -> str (stdout, status discarded)
///
/// Shells out via `/bin/sh -c cmd` on Unix; returns stdout with trailing
/// newline stripped to match CPython's behaviour.
pub fn mb_subprocess_getoutput(cmd: MbValue) -> MbValue {
    let (_status, out) = run_shell(cmd);
    MbValue::from_ptr(MbObject::new_str(out))
}

/// subprocess.getstatusoutput(cmd) -> tuple(status, output)
pub fn mb_subprocess_getstatusoutput(cmd: MbValue) -> MbValue {
    let (status, out) = run_shell(cmd);
    let tup = vec![
        MbValue::from_int(status as i64),
        MbValue::from_ptr(MbObject::new_str(out)),
    ];
    MbValue::from_ptr(MbObject::new_tuple(tup))
}

fn run_shell(cmd: MbValue) -> (i32, String) {
    let cmd_str = extract_str(cmd).unwrap_or_default();
    if cmd_str.is_empty() {
        return (-1, String::new());
    }
    let result = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(&cmd_str)
        .output();
    match result {
        Ok(output) => {
            let mut s = String::from_utf8_lossy(&output.stdout).to_string();
            s.push_str(&String::from_utf8_lossy(&output.stderr));
            // Strip a single trailing newline to match CPython.
            if s.ends_with('\n') {
                s.pop();
            }
            (returncode_of(&output.status), s)
        }
        Err(e) => (-1, e.to_string()),
    }
}

/// subprocess.list2cmdline(seq) -> str
///
/// Joins argv into a single shell-style string. Args containing
/// whitespace or quote characters are wrapped in double quotes; embedded
/// quotes are backslash-escaped. This is a simplified form sufficient
/// for round-trip logging on POSIX.
pub fn mb_subprocess_list2cmdline(seq: MbValue) -> MbValue {
    let Ok(args) = extract_args(seq) else {
        // A non-str/bytes element raised TypeError inside extract_args
        // (CPython's list2cmdline fsdecodes each arg and rejects e.g. ints).
        return MbValue::none();
    };
    let mut out = String::new();
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let needs_quote = a.is_empty()
            || a.chars()
                .any(|c| c.is_whitespace() || c == '"' || c == '\'' || c == '\\');
        if needs_quote {
            out.push('"');
            for c in a.chars() {
                if c == '"' || c == '\\' {
                    out.push('\\');
                }
                out.push(c);
            }
            out.push('"');
        } else {
            out.push_str(a);
        }
    }
    MbValue::from_ptr(MbObject::new_str(out))
}

/// subprocess.Popen(args) -> Popen instance
///
/// Single-arg entry point retained for the JIT direct-call symbol and unit
/// tests. The full constructor (with `bufsize` positional and validation)
/// is reached through the variadic dispatcher / `mb_subprocess_popen_impl`.
pub fn mb_subprocess_popen(args: MbValue) -> MbValue {
    mb_subprocess_popen_impl(&[args])
}

/// subprocess.Popen(args, bufsize=...) -> Popen instance
///
/// Carve-out: runs the command synchronously to completion. The returned
/// instance carries `args`, `returncode`, `stdout`, `stderr`, plus
/// `pid` (best-effort, 0 if unavailable) and None placeholders for
/// `stdin` / `stdout` / `stderr` so the `p = Popen(...); p.returncode`
/// pattern works.
///
/// Validation performed before any spawn, mirroring CPython:
///   - the second positional (`bufsize`) must be an int, else `TypeError`;
///   - an embedded NUL byte in any argv token raises `ValueError`.
pub fn mb_subprocess_popen_impl(a: &[MbValue]) -> MbValue {
    let args = a.first().copied().unwrap_or_else(MbValue::none);
    let opts = parse_spawn_opts(a);

    // bufsize (2nd positional) must be an integer. CPython raises
    // `TypeError("bufsize must be an integer")` for e.g. `Popen(argv,
    // 'orange')`. The native call convention can fold keyword arguments
    // (`stdout=`, `env=`, ...) into trailing positional slots, so to avoid
    // mis-firing on those we only reject an explicit *string* bufsize, which
    // is never a legitimate value for any Popen keyword.
    if let Some(bufsize) = a.get(1) {
        if extract_str(*bufsize).is_some() {
            return raise("TypeError", "bufsize must be an integer");
        }
    }

    let Ok(cmd_args) = extract_args(args) else {
        // A non-str/bytes argv element raised TypeError inside extract_args.
        return MbValue::none();
    };

    // CPython rejects embedded NUL bytes in argv with ValueError
    // ("embedded null byte") raised before the child is spawned.
    for tok in &cmd_args {
        if tok.contains('\0') {
            return raise("ValueError", "embedded null byte");
        }
    }

    // CPython also rejects an `env=` mapping with a NUL byte in any variable
    // name or value, raising ValueError("embedded null byte") before the spawn.
    if args_have_nul_env(a) {
        return raise("ValueError", "embedded null byte");
    }

    let mut fields = FxHashMap::default();
    fields.insert("args".into(), args);
    fields.insert("stdin".into(), MbValue::none());
    fields.insert("stdout".into(), MbValue::none());
    fields.insert("stderr".into(), MbValue::none());

    if cmd_args.is_empty() {
        fields.insert("returncode".into(), MbValue::from_int(-1));
        fields.insert("pid".into(), MbValue::from_int(0));
        return new_instance_with_fields("Popen", fields);
    }

    let result = std::process::Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .output();

    match result {
        Ok(output) => {
            let code = returncode_of(&output.status);
            fields.insert("returncode".into(), MbValue::from_int(code as i64));
            fields.insert("pid".into(), MbValue::from_int(0));
            fields.insert(
                "stdout".into(),
                pipe_stream_value(output.stdout.clone(), opts.stdout_sel == -1, opts.text),
            );
            fields.insert(
                "stderr".into(),
                pipe_stream_value(output.stderr.clone(), opts.stderr_sel == -1, opts.text),
            );
            // Captured child streams for `communicate()` — the carve-out
            // runs the child synchronously, so they are fully available.
            fields.insert(
                "_captured_stdout".into(),
                MbValue::from_ptr(MbObject::new_bytes(output.stdout)),
            );
            fields.insert(
                "_captured_stderr".into(),
                MbValue::from_ptr(MbObject::new_bytes(output.stderr)),
            );
        }
        // A missing/unexecutable command raises the OSError family, matching
        // CPython's behaviour at construction time.
        Err(e) => {
            return raise(spawn_error_type(&e), &spawn_error_message(&e, &cmd_args[0]));
        }
    }
    new_instance_with_fields("Popen", fields)
}

/// `CompletedProcess.check_returncode()` — no-op on zero, raises
/// `CalledProcessError` otherwise.
unsafe extern "C" fn completed_check_returncode(self_v: MbValue, _args: MbValue) -> MbValue {
    let code = self_v
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .read()
                    .unwrap()
                    .get("returncode")
                    .and_then(|v| v.as_int())
            } else {
                None
            }
        })
        .unwrap_or(0);
    if code != 0 {
        return raise(
            "CalledProcessError",
            &format!("Command returned non-zero exit status {code}."),
        );
    }
    MbValue::none()
}

/// `Popen.communicate(input=None, timeout=None)` -> `(stdout, stderr)`.
/// The child already ran to completion at construction, so this returns
/// the captured streams; a post-hoc `input` cannot be delivered and is
/// ignored.
unsafe extern "C" fn popen_communicate(self_v: MbValue, _args: MbValue) -> MbValue {
    let field = |name: &str| -> MbValue {
        if let Some(ptr) = self_v.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    if let Some(v) = fields.read().unwrap().get(name).copied() {
                        return v;
                    }
                }
            }
        }
        MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
    };
    let out = field("_captured_stdout");
    let err = field("_captured_stderr");
    MbValue::from_ptr(MbObject::new_tuple(vec![out, err]))
}

/// `Popen.poll()` -> returncode (child already exited in the carve-out).
unsafe extern "C" fn popen_poll(self_v: MbValue, args: MbValue) -> MbValue {
    unsafe { popen_wait(self_v, args) }
}

/// `Popen.kill()` / `terminate()` / `send_signal(sig)` — the child already
/// exited; CPython tolerates signaling an exited child, so these no-op.
unsafe extern "C" fn popen_kill(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

/// `with Popen(...) as p:` — enter returns self; exit closes the
/// (already-drained) pipes and never suppresses exceptions.
unsafe extern "C" fn popen_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(self_v) };
    self_v
}

unsafe extern "C" fn popen_exit(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(ptr) = self_v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                for name in ["stdin", "stdout", "stderr"] {
                    if let Some(stream) = fields.get(name).copied() {
                        close_stream_value(stream);
                    }
                }
            }
        }
    }
    MbValue::from_bool(false)
}

/// `Popen.wait(timeout=None)` -> returncode (int).
///
/// Carve-out: the constructor already runs the child synchronously to
/// completion and records `returncode`, so `wait()` simply returns the stored
/// `returncode` field. This matches CPython's contract that `wait()` returns
/// `self.returncode` and is idempotent (a second `wait()` returns the same
/// value). The `timeout` positional/keyword is accepted and ignored — the
/// child has already exited, so no `TimeoutExpired` can occur here.
///
/// Registered as a variadic method on the native "Popen" class so
/// `p.wait()` dispatches through the normal instance-method path. `_args`
/// holds the packed positional list (`[timeout]`), which is unused.
unsafe extern "C" fn popen_wait(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(ptr) = self_v.as_ptr() {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            if let Some(rc) = fields.read().unwrap().get("returncode").copied() {
                return rc;
            }
        }
    }
    MbValue::none()
}

/// subprocess.CompletedProcess(args, returncode, stdout=None, stderr=None)
pub fn mb_subprocess_completed_process_new(a: &[MbValue]) -> MbValue {
    let args = a.first().copied().unwrap_or_else(MbValue::none);
    let returncode = a.get(1).and_then(|v| v.as_int()).unwrap_or(0) as i32;
    let stdout = a.get(2).copied().unwrap_or_else(MbValue::none);
    let stderr = a.get(3).copied().unwrap_or_else(MbValue::none);

    let mut f = FxHashMap::default();
    f.insert("args".into(), args);
    f.insert("returncode".into(), MbValue::from_int(returncode as i64));
    retain(stdout);
    retain(stderr);
    f.insert("stdout".into(), stdout);
    f.insert("stderr".into(), stderr);
    new_instance_with_fields("CompletedProcess", f)
}

/// subprocess.CalledProcessError(returncode, cmd, output=None, stderr=None)
///
/// Constructs an instance with the CPython field layout. Note: `output` and
/// `stdout` are aliases (CPython makes `stdout` a property over `output`).
pub fn mb_subprocess_called_process_error_new(a: &[MbValue]) -> MbValue {
    let returncode = a.first().and_then(|v| v.as_int()).unwrap_or(0);
    let cmd = a.get(1).copied().unwrap_or_else(MbValue::none);
    let output = a.get(2).copied().unwrap_or_else(MbValue::none);
    let stderr = a.get(3).copied().unwrap_or_else(MbValue::none);

    let mut f = FxHashMap::default();
    f.insert("returncode".into(), MbValue::from_int(returncode));
    retain(cmd);
    // `output` aliases into both `output` and `stdout` → retain twice.
    retain(output);
    retain(output);
    retain(stderr);
    f.insert("cmd".into(), cmd);
    f.insert("output".into(), output);
    f.insert("stdout".into(), output);
    f.insert("stderr".into(), stderr);
    // Cosmetic display repr only — never raise from message construction.
    let cmd_repr = extract_args(cmd).unwrap_or_default().join(" ");
    let msg = format!(
        "Command '{}' returned non-zero exit status {}.",
        cmd_repr, returncode
    );
    f.insert(
        "message".into(),
        MbValue::from_ptr(MbObject::new_str(msg.clone())),
    );
    f.insert(
        "args".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str(msg),
        )])),
    );
    new_instance_with_fields("CalledProcessError", f)
}

/// subprocess.TimeoutExpired(cmd, timeout, output=None, stderr=None)
pub fn mb_subprocess_timeout_expired_new(a: &[MbValue]) -> MbValue {
    let cmd = a.first().copied().unwrap_or_else(MbValue::none);
    let timeout = a.get(1).copied().unwrap_or_else(MbValue::none);
    let output = a.get(2).copied().unwrap_or_else(MbValue::none);
    let stderr = a.get(3).copied().unwrap_or_else(MbValue::none);

    let mut f = FxHashMap::default();
    retain(cmd);
    retain(timeout);
    // `output` aliases into both `output` and `stdout` → retain twice.
    retain(output);
    retain(output);
    retain(stderr);
    f.insert("cmd".into(), cmd);
    f.insert("timeout".into(), timeout);
    f.insert("output".into(), output);
    f.insert("stdout".into(), output);
    f.insert("stderr".into(), stderr);
    // Cosmetic display repr only — never raise from message construction.
    let cmd_repr = extract_args(cmd).unwrap_or_default().join(" ");
    let timeout_repr = timeout
        .as_int()
        .map(|i| i.to_string())
        .or_else(|| timeout.as_float().map(|x| x.to_string()))
        .unwrap_or_else(|| "0".to_string());
    let msg = format!(
        "Command '{}' timed out after {} seconds",
        cmd_repr, timeout_repr
    );
    f.insert(
        "message".into(),
        MbValue::from_ptr(MbObject::new_str(msg.clone())),
    );
    f.insert(
        "args".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str(msg),
        )])),
    );
    new_instance_with_fields("TimeoutExpired", f)
}

/// subprocess.SubprocessError(*args)
pub fn mb_subprocess_subprocess_error_new(a: &[MbValue]) -> MbValue {
    let mut f = FxHashMap::default();
    let arg0 = a.first().copied().unwrap_or_else(MbValue::none);
    retain(arg0);
    if let Some(s) = extract_str(arg0) {
        f.insert("message".into(), MbValue::from_ptr(MbObject::new_str(s)));
    }
    let tuple_items: Vec<MbValue> = a.iter().copied().collect();
    for &v in &tuple_items {
        retain(v);
    }
    f.insert(
        "args".into(),
        MbValue::from_ptr(MbObject::new_tuple(tuple_items)),
    );
    new_instance_with_fields("SubprocessError", f)
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
                    if let Some(v) = fields.read().unwrap().get(field) {
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

    fn class_name(instance: MbValue) -> Option<String> {
        instance.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_run_echo_completed_process() {
        // CPython: run() without capture_output leaves stdout as None;
        // request capture_output + text to observe the child's output.
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("echo"), s("hello")]));
        let kwargs = crate::runtime::dict_ops::mb_dict_new();
        crate::runtime::dict_ops::mb_dict_setitem(
            kwargs,
            s("capture_output"),
            MbValue::from_bool(true),
        );
        crate::runtime::dict_ops::mb_dict_setitem(kwargs, s("text"), MbValue::from_bool(true));
        let result = mb_subprocess_run_all(&[args, kwargs]);
        assert_eq!(class_name(result).as_deref(), Some("CompletedProcess"));
        assert_eq!(get_field(result, "returncode").as_int(), Some(0));
        let stdout = get_str(get_field(result, "stdout")).unwrap_or_default();
        assert!(stdout.contains("hello"), "stdout = {:?}", stdout);

        // Default (no capture): stdout stays None, returncode recorded.
        let args2 = MbValue::from_ptr(MbObject::new_list(vec![s("true")]));
        let plain = mb_subprocess_run_all(&[args2]);
        assert!(
            get_field(plain, "stdout").is_none(),
            "uncaptured stdout is None"
        );
    }

    #[test]
    fn test_call_returns_zero() {
        assert_eq!(mb_subprocess_call(s("echo hi")).as_int(), Some(0));
    }

    fn get_bytes(val: MbValue) -> Option<Vec<u8>> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Bytes(ref b) = (*ptr).data {
                Some(b.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_check_output_bytes() {
        // CPython default (no text=) returns bytes.
        let out = mb_subprocess_check_output(s("echo hi"));
        let b = get_bytes(out).unwrap_or_default();
        assert_eq!(String::from_utf8_lossy(&b).trim(), "hi");
    }

    #[test]
    fn test_check_call_zero_ok() {
        assert_eq!(mb_subprocess_check_call(s("echo hi")).as_int(), Some(0));
    }

    #[test]
    fn test_check_call_nonzero_raises_called_process_error() {
        super::super::super::exception::mb_clear_exception();
        let r = mb_subprocess_check_call(s("false"));
        assert!(r.is_none());
        // The catchable instance carries returncode.
        let inst = super::super::super::class::mb_catch_exception_instance();
        assert_eq!(class_name(inst).as_deref(), Some("CalledProcessError"));
        assert_ne!(get_field(inst, "returncode").as_int(), Some(0));
    }

    #[test]
    fn test_empty_args_raises() {
        super::super::super::exception::mb_clear_exception();
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        // CPython 3.12: every dispatcher reaches `_execute_child`, which does
        // `args[0]` and raises `IndexError: list index out of range` on `[]`.
        // call/run/check_output/check_call all raise (return None).
        assert!(mb_subprocess_call(empty).is_none());
        super::super::super::exception::mb_clear_exception();
        assert!(mb_subprocess_check_output(empty).is_none());
        super::super::super::exception::mb_clear_exception();
        assert!(mb_subprocess_run(empty).is_none());
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_getoutput_strips_trailing_newline() {
        let out = mb_subprocess_getoutput(s("echo hi"));
        assert_eq!(get_str(out).as_deref(), Some("hi"));
    }

    #[test]
    fn test_getstatusoutput_tuple() {
        let r = mb_subprocess_getstatusoutput(s("echo hi"));
        unsafe {
            if let ObjData::Tuple(ref items) = (*r.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_int(), Some(0));
                assert_eq!(get_str(items[1]).as_deref(), Some("hi"));
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_getstatusoutput_nonzero_status() {
        let r = mb_subprocess_getstatusoutput(s("false"));
        unsafe {
            if let ObjData::Tuple(ref items) = (*r.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 2);
                assert_ne!(items[0].as_int(), Some(0));
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_list2cmdline_simple() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![s("echo"), s("hello")]));
        let out = mb_subprocess_list2cmdline(list);
        assert_eq!(get_str(out).as_deref(), Some("echo hello"));
    }

    #[test]
    fn test_list2cmdline_quotes_whitespace() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![s("echo"), s("hello world")]));
        let out = mb_subprocess_list2cmdline(list);
        assert_eq!(get_str(out).as_deref(), Some("echo \"hello world\""));
    }

    #[test]
    fn test_popen_returncode_populated() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("echo"), s("hi")]));
        let p = mb_subprocess_popen(args);
        assert_eq!(class_name(p).as_deref(), Some("Popen"));
        assert_eq!(get_field(p, "returncode").as_int(), Some(0));
        assert!(get_field(p, "stdin").is_none());
    }

    #[test]
    fn test_popen_empty_args() {
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        let p = mb_subprocess_popen(empty);
        assert_eq!(class_name(p).as_deref(), Some("Popen"));
        assert_eq!(get_field(p, "returncode").as_int(), Some(-1));
    }

    #[test]
    fn test_completed_process_constructor() {
        // CompletedProcess(args, returncode, stdout, stderr)
        let cp = mb_subprocess_completed_process_new(&[
            s("echo"),
            MbValue::from_int(0),
            s("out"),
            s("err"),
        ]);
        assert_eq!(class_name(cp).as_deref(), Some("CompletedProcess"));
        assert_eq!(get_field(cp, "returncode").as_int(), Some(0));
        assert_eq!(get_str(get_field(cp, "stdout")).as_deref(), Some("out"));
        assert_eq!(get_str(get_field(cp, "stderr")).as_deref(), Some("err"));
    }

    #[test]
    fn test_called_process_error_constructor_fields() {
        // CalledProcessError(returncode, cmd, output, stderr)
        let cmd = MbValue::from_ptr(MbObject::new_list(vec![s("false")]));
        let e1 =
            mb_subprocess_called_process_error_new(&[MbValue::from_int(7), cmd, s("o"), s("e")]);
        assert_eq!(class_name(e1).as_deref(), Some("CalledProcessError"));
        assert_eq!(get_field(e1, "returncode").as_int(), Some(7));
        assert_eq!(get_str(get_field(e1, "output")).as_deref(), Some("o"));
        assert_eq!(get_str(get_field(e1, "stderr")).as_deref(), Some("e"));

        let e2 = mb_subprocess_timeout_expired_new(&[s("cmd"), MbValue::from_int(5)]);
        assert_eq!(class_name(e2).as_deref(), Some("TimeoutExpired"));
        assert_eq!(get_field(e2, "timeout").as_int(), Some(5));

        let e3 = mb_subprocess_subprocess_error_new(&[s("boom")]);
        assert_eq!(class_name(e3).as_deref(), Some("SubprocessError"));
        assert_eq!(get_str(get_field(e3, "message")).as_deref(), Some("boom"));
    }

    #[test]
    fn test_register_wires_full_surface() {
        register();
        let snap = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        // 12 dispatchers should each be registered; the set is monotonic
        // across the test process so we only assert presence is non-zero.
        assert!(
            snap >= 12,
            "expected at least 12 native func addrs registered"
        );
    }
}
