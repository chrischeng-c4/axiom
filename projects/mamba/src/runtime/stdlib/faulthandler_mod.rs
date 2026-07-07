use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// faulthandler module for Mamba (#879).
///
/// Provides the core CPython 3.12 `faulthandler` diagnostics surface:
///   - `enable(file=sys.stderr, all_threads=True)` / `disable()` /
///     `is_enabled()` — install/remove real Unix fatal-signal handlers for
///     `SIGSEGV` / `SIGFPE` / `SIGABRT` / `SIGBUS` via `libc::signal`.
///   - `dump_traceback(file=sys.stderr, all_threads=True)` — prints the
///     current Python-level call stack, reusing the same
///     `traceback_mod::trace_stack_snapshot()` frame data that backs
///     `sys._getframe()` / `inspect.currentframe()` (#889).
///
/// Carve-outs (deliberately out of scope, per the issue):
///   - `dump_traceback_later()` timers and Windows support are not
///     implemented.
///   - Only the *current* thread's frames are ever dumped; `all_threads`
///     only changes the printed header ("Current thread ..." vs "Stack
///     ..."), matching CPython's single-thread-visible-here shape without
///     modeling other OS threads' Python frames.
///   - The fatal-signal handler formats its dump using ordinary Rust
///     `String`/`format!` machinery before calling `libc::write`. This is a
///     pragmatic best-effort dump (R3), not textbook POSIX
///     async-signal-safety (real CPython's C implementation preallocates
///     fixed buffers); acceptable because the handler only runs once,
///     immediately before the process terminates via the signal's default
///     disposition.
use std::collections::HashMap;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static ALL_THREADS: AtomicBool = AtomicBool::new(true);
static DUMP_FD: AtomicI32 = AtomicI32::new(2);

/// The fatal signals faulthandler installs handlers for (R2). CPython also
/// covers SIGILL on request but the issue scopes this to SEGV/FPE/ABRT/BUS.
const FATAL_SIGNALS: [c_int; 4] = [libc::SIGSEGV, libc::SIGFPE, libc::SIGABRT, libc::SIGBUS];

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
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

fn is_dict_value(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
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

// ── file → fd resolution (best-effort, R3) ──

/// `sys.stdout` / `sys.stderr` / `sys.stdin` are lightweight instance
/// sentinels (`sys._Stream`, see sys_mod::build_stream_stub) with no real
/// `fileno()` method wired up. Recognize them directly by name so the
/// documented default (`file=sys.stderr` → fd 2) resolves correctly instead
/// of falling through to the generic fallback.
fn stream_stub_fd(file: MbValue) -> Option<i32> {
    let ptr = file.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "sys._Stream" {
                let name = fields.read().unwrap().get("name").copied()?;
                let name = extract_str(name)?;
                return Some(match name.as_str() {
                    "<stdout>" => 1,
                    "<stdin>" => 0,
                    _ => 2, // "<stderr>" and anything else stream-shaped
                });
            }
        }
    }
    None
}

fn resolve_fd(file: MbValue) -> i32 {
    if file.is_none() {
        return 2; // default: sys.stderr
    }
    if let Some(fd) = file.as_int() {
        return fd as i32;
    }
    if let Some(fd) = stream_stub_fd(file) {
        return fd;
    }
    // Best-effort: try file.fileno() (real file objects / tempfiles).
    let empty_args = MbValue::from_ptr(MbObject::new_list(Vec::new()));
    let result = super::super::class::mb_call_method(file, new_str("fileno"), empty_args);
    let raised =
        super::super::builtins::mb_is_truthy(super::super::exception::mb_has_exception()) != 0;
    if raised {
        super::super::exception::mb_clear_exception();
        return 2;
    }
    result.as_int().map(|v| v as i32).unwrap_or(2)
}

fn write_raw(fd: i32, buf: &[u8]) {
    let mut rest = buf;
    unsafe {
        while !rest.is_empty() {
            let n = libc::write(fd, rest.as_ptr() as *const c_void, rest.len());
            if n <= 0 {
                break;
            }
            rest = &rest[n as usize..];
        }
    }
}

fn current_thread_hex_id() -> u64 {
    #[cfg(unix)]
    unsafe {
        libc::pthread_self() as u64
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// Build and write a CPython-shaped stack dump:
///   [Fatal Python error: <name>\n\n]
///   <Current thread 0x.. | Stack> (most recent call first):
///     File "<file>", line N in <func>
///     ...
fn write_stack_dump(fd: i32, all_threads: bool, fatal_error: Option<&str>) {
    let mut out = String::new();
    if let Some(name) = fatal_error {
        out.push_str(&format!("Fatal Python error: {name}\n\n"));
    }
    if all_threads {
        out.push_str(&format!(
            "Current thread 0x{:016x} (most recent call first):\n",
            current_thread_hex_id()
        ));
    } else {
        out.push_str("Stack (most recent call first):\n");
    }
    // trace_stack_snapshot() is module-frame-first / innermost-last; the
    // dump wants innermost (most recent call) first.
    let frames = super::traceback_mod::trace_stack_snapshot();
    if frames.is_empty() {
        out.push_str("  <no Python frame>\n");
    } else {
        for (filename, lineno, name) in frames.iter().rev() {
            out.push_str(&format!("  File \"{filename}\", line {lineno} in {name}\n"));
        }
    }
    write_raw(fd, out.as_bytes());
}

fn fatal_signal_name(signum: c_int) -> &'static str {
    match signum {
        libc::SIGSEGV => "Segmentation fault",
        libc::SIGFPE => "Floating-point exception",
        libc::SIGABRT => "Aborted",
        libc::SIGBUS => "Bus error",
        _ => "Fatal error",
    }
}

extern "C" fn fault_signal_handler(signum: c_int) {
    let fd = DUMP_FD.load(Ordering::Relaxed);
    let all_threads = ALL_THREADS.load(Ordering::Relaxed);
    write_stack_dump(fd, all_threads, Some(fatal_signal_name(signum)));
    // Reset to the default disposition and re-raise so the process still
    // terminates the same way it would without faulthandler installed
    // (core dump / signal exit code), matching CPython's own fatal handler.
    unsafe {
        libc::signal(signum, libc::SIG_DFL);
        libc::raise(signum);
    }
}

unsafe fn install_fatal_handlers() {
    for &sig in FATAL_SIGNALS.iter() {
        unsafe {
            libc::signal(sig, fault_signal_handler as usize as libc::sighandler_t);
        }
    }
}

unsafe fn restore_default_handlers() {
    for &sig in FATAL_SIGNALS.iter() {
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
        }
    }
}

// ── public entry points ──

fn mb_enable(file: MbValue, all_threads: bool) -> MbValue {
    let fd = resolve_fd(file);
    DUMP_FD.store(fd, Ordering::Relaxed);
    ALL_THREADS.store(all_threads, Ordering::Relaxed);
    if !ENABLED.swap(true, Ordering::SeqCst) {
        unsafe {
            install_fatal_handlers();
        }
    }
    MbValue::none()
}

fn mb_disable() -> MbValue {
    if ENABLED.swap(false, Ordering::SeqCst) {
        unsafe {
            restore_default_handlers();
        }
    }
    MbValue::none()
}

fn mb_is_enabled() -> MbValue {
    MbValue::from_bool(ENABLED.load(Ordering::SeqCst))
}

fn mb_dump_traceback(file: MbValue, all_threads: bool) -> MbValue {
    let fd = resolve_fd(file);
    write_stack_dump(fd, all_threads, None);
    MbValue::none()
}

// ── dispatchers: (args_ptr, nargs) native-func convention ──

unsafe extern "C" fn dispatch_enable(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let pos: Vec<MbValue> = a.iter().copied().filter(|v| !is_dict_value(*v)).collect();
    let file = kwarg(a, "file")
        .or_else(|| pos.first().copied())
        .filter(|v| !v.is_none())
        .unwrap_or_else(MbValue::none);
    let all_threads = kwarg(a, "all_threads")
        .or_else(|| pos.get(1).copied())
        .map(|v| super::super::builtins::mb_is_truthy(v) != 0)
        .unwrap_or(true);
    mb_enable(file, all_threads)
}

unsafe extern "C" fn dispatch_disable(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_disable()
}

unsafe extern "C" fn dispatch_is_enabled(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_is_enabled()
}

unsafe extern "C" fn dispatch_dump_traceback(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let pos: Vec<MbValue> = a.iter().copied().filter(|v| !is_dict_value(*v)).collect();
    let file = kwarg(a, "file")
        .or_else(|| pos.first().copied())
        .filter(|v| !v.is_none())
        .unwrap_or_else(MbValue::none);
    let all_threads = kwarg(a, "all_threads")
        .or_else(|| pos.get(1).copied())
        .map(|v| super::super::builtins::mb_is_truthy(v) != 0)
        .unwrap_or(true);
    mb_dump_traceback(file, all_threads)
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("enable", dispatch_enable as usize),
        ("disable", dispatch_disable as usize),
        ("is_enabled", dispatch_is_enabled as usize),
        ("dump_traceback", dispatch_dump_traceback as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("faulthandler", attrs);
}
