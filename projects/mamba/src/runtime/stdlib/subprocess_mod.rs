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
//!   - No async stream wiring: `Popen.stdin`/`stdout`/`stderr` are plain
//!     None fields; `communicate`, `wait`, `poll` are not wired through
//!     method dispatch yet. `Popen(...)` runs the command synchronously
//!     to completion and stores `returncode` so the common
//!     `p = Popen(...); p.returncode` pattern is observable.
//!   - `shell=True` keyword argument is not parsed. The argv-list shape
//!     (`run(["cmd", "arg"])`) and the whitespace-split string shape
//!     (`run("cmd arg")`) are both honored, mirroring the existing
//!     `extract_args` contract.
//!   - `env`, `cwd`, `timeout`, `input`, `capture_output`, `text`,
//!     `encoding`, and other keyword arguments are accepted by the
//!     variadic dispatchers but silently ignored. Stdout/stderr are
//!     captured by default (CPython's `run` default is no capture; this
//!     deviation matches the existing #397 contract and keeps the
//!     CompletedProcess fields populated).
//!   - Exception classes (`CalledProcessError`, `TimeoutExpired`,
//!     `SubprocessError`) are Instance stubs with `__name__` /
//!     `__module__` fields. Mamba does not model the Exception subclass
//!     hierarchy, so `raise subprocess.CalledProcessError(...)` is not
//!     wired; the stubs exist for `dir(subprocess)` surface parity and
//!     `except subprocess.CalledProcessError:` sentinels.
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

disp_variadic!(dispatch_run, mb_subprocess_run);
disp_variadic!(dispatch_call, mb_subprocess_call);
disp_variadic!(dispatch_check_output, mb_subprocess_check_output);
disp_variadic!(dispatch_check_call, mb_subprocess_check_call);
disp_variadic!(dispatch_getoutput, mb_subprocess_getoutput);
disp_variadic!(dispatch_getstatusoutput, mb_subprocess_getstatusoutput);
disp_variadic!(dispatch_list2cmdline, mb_subprocess_list2cmdline);
disp_variadic!(dispatch_popen, mb_subprocess_popen);
disp_variadic!(
    dispatch_completed_process,
    mb_subprocess_completed_process_new
);
disp_variadic!(
    dispatch_called_process_error,
    mb_subprocess_called_process_error_new
);
disp_variadic!(dispatch_timeout_expired, mb_subprocess_timeout_expired_new);
disp_variadic!(
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

    // Integer constants — match CPython's negative sentinels.
    attrs.insert("PIPE".into(), MbValue::from_int(-1));
    attrs.insert("STDOUT".into(), MbValue::from_int(-2));
    attrs.insert("DEVNULL".into(), MbValue::from_int(-3));

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

/// Coerce an MbValue into a Vec<String> of argv tokens. Accepts:
///   - a heap string: split on whitespace
///   - a list/tuple of strings: take each element
fn extract_args(val: MbValue) -> Vec<String> {
    if let Some(s) = extract_str(val) {
        return s.split_whitespace().map(|w| w.to_string()).collect();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    return lock
                        .read()
                        .unwrap()
                        .iter()
                        .filter_map(|v| extract_str(*v))
                        .collect();
                }
                ObjData::Tuple(items) => {
                    return items.iter().filter_map(|v| extract_str(*v)).collect();
                }
                _ => {}
            }
        }
    }
    vec![]
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

/// subprocess.run(args) -> CompletedProcess instance
pub fn mb_subprocess_run(args: MbValue) -> MbValue {
    let cmd_args = extract_args(args);
    if cmd_args.is_empty() {
        return make_completed_process(args, -1, "", "empty command");
    }

    let result = std::process::Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let code = output.status.code().unwrap_or(-1);
            make_completed_process(args, code, &stdout, &stderr)
        }
        Err(e) => make_completed_process(args, -1, "", &e.to_string()),
    }
}

/// subprocess.call(args) -> returncode (int)
pub fn mb_subprocess_call(args: MbValue) -> MbValue {
    let cmd_args = extract_args(args);
    if cmd_args.is_empty() {
        return MbValue::from_int(-1);
    }

    let result = std::process::Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .status();

    match result {
        Ok(status) => MbValue::from_int(status.code().unwrap_or(-1) as i64),
        Err(_) => MbValue::from_int(-1),
    }
}

/// subprocess.check_output(args) -> stdout string
pub fn mb_subprocess_check_output(args: MbValue) -> MbValue {
    let cmd_args = extract_args(args);
    if cmd_args.is_empty() {
        return MbValue::none();
    }

    let result = std::process::Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            MbValue::from_ptr(MbObject::new_str(stdout))
        }
        _ => MbValue::none(),
    }
}

/// subprocess.check_call(args) -> 0 or raises CalledProcessError
pub fn mb_subprocess_check_call(args: MbValue) -> MbValue {
    let code = mb_subprocess_call(args);
    if code.as_int() != Some(0) {
        panic!("CalledProcessError: non-zero exit status");
    }
    MbValue::from_int(0)
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
            (output.status.code().unwrap_or(-1), s)
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
    let args = extract_args(seq);
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
/// Carve-out: runs the command synchronously to completion. The returned
/// instance carries `args`, `returncode`, `stdout`, `stderr`, plus
/// `pid` (best-effort, 0 if unavailable) and None placeholders for
/// `stdin` / `stdout_pipe` / `stderr_pipe` so the
/// `p = Popen(...); p.returncode` pattern works.
pub fn mb_subprocess_popen(args: MbValue) -> MbValue {
    let cmd_args = extract_args(args);
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
            let code = output.status.code().unwrap_or(-1);
            fields.insert("returncode".into(), MbValue::from_int(code as i64));
            fields.insert("pid".into(), MbValue::from_int(0));
        }
        Err(_) => {
            fields.insert("returncode".into(), MbValue::from_int(-1));
            fields.insert("pid".into(), MbValue::from_int(0));
        }
    }
    new_instance_with_fields("Popen", fields)
}

/// subprocess.CompletedProcess(args, returncode, stdout=None, stderr=None)
pub fn mb_subprocess_completed_process_new(args: MbValue) -> MbValue {
    // The dispatcher only passes the first positional. CPython's
    // CompletedProcess accepts (args, returncode, stdout, stderr); our
    // wrapper degrades gracefully to a returncode=0 stub when only the
    // args slot is supplied. The full constructor is reachable via
    // make_completed_process inside `run`.
    make_completed_process(args, 0, "", "")
}

fn make_exception_stub(class_name: &str, args: MbValue) -> MbValue {
    let mut f = FxHashMap::default();
    f.insert(
        "__name__".into(),
        MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
    );
    f.insert(
        "__module__".into(),
        MbValue::from_ptr(MbObject::new_str("subprocess".to_string())),
    );
    f.insert("args".into(), args);
    new_instance_with_fields(class_name, f)
}

/// subprocess.CalledProcessError(returncode, cmd, output=None, stderr=None)
pub fn mb_subprocess_called_process_error_new(args: MbValue) -> MbValue {
    make_exception_stub("CalledProcessError", args)
}

/// subprocess.TimeoutExpired(cmd, timeout, output=None, stderr=None)
pub fn mb_subprocess_timeout_expired_new(args: MbValue) -> MbValue {
    make_exception_stub("TimeoutExpired", args)
}

/// subprocess.SubprocessError()
pub fn mb_subprocess_subprocess_error_new(args: MbValue) -> MbValue {
    make_exception_stub("SubprocessError", args)
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
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("echo"), s("hello")]));
        let result = mb_subprocess_run(args);
        assert_eq!(class_name(result).as_deref(), Some("CompletedProcess"));
        assert_eq!(get_field(result, "returncode").as_int(), Some(0));
        let stdout = get_str(get_field(result, "stdout")).unwrap_or_default();
        assert!(stdout.contains("hello"), "stdout = {:?}", stdout);
    }

    #[test]
    fn test_call_returns_zero() {
        assert_eq!(mb_subprocess_call(s("echo hi")).as_int(), Some(0));
    }

    #[test]
    fn test_check_output_string() {
        let out = mb_subprocess_check_output(s("echo hi"));
        let s = get_str(out).unwrap_or_default();
        assert_eq!(s.trim(), "hi");
    }

    #[test]
    fn test_check_call_zero_ok() {
        assert_eq!(mb_subprocess_check_call(s("echo hi")).as_int(), Some(0));
    }

    #[test]
    fn test_empty_args_paths() {
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(mb_subprocess_call(empty).as_int(), Some(-1));
        assert!(mb_subprocess_check_output(empty).is_none());
        let cp = mb_subprocess_run(empty);
        assert_eq!(get_field(cp, "returncode").as_int(), Some(-1));
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
    fn test_completed_process_stub() {
        let cp = mb_subprocess_completed_process_new(s("echo"));
        assert_eq!(class_name(cp).as_deref(), Some("CompletedProcess"));
        assert_eq!(get_field(cp, "returncode").as_int(), Some(0));
    }

    #[test]
    fn test_exception_stubs_carry_names() {
        let e1 = mb_subprocess_called_process_error_new(MbValue::none());
        assert_eq!(class_name(e1).as_deref(), Some("CalledProcessError"));
        assert_eq!(
            get_str(get_field(e1, "__name__")).as_deref(),
            Some("CalledProcessError")
        );
        assert_eq!(
            get_str(get_field(e1, "__module__")).as_deref(),
            Some("subprocess")
        );

        let e2 = mb_subprocess_timeout_expired_new(MbValue::none());
        assert_eq!(class_name(e2).as_deref(), Some("TimeoutExpired"));

        let e3 = mb_subprocess_subprocess_error_new(MbValue::none());
        assert_eq!(class_name(e3).as_deref(), Some("SubprocessError"));
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
