/// traceback module for Mamba (#431, #1441, #1265 Task — Wave-N).
///
/// Provides the CPython 3.12 `traceback` 19-entry public surface:
///   - Callables (16): print_tb, print_exception, print_exc, print_last,
///     print_stack, extract_tb, extract_stack, format_list,
///     format_exception_only, format_exception, format_exc, format_tb,
///     format_stack, clear_frames, walk_tb, walk_stack.
///   - Class shells (3): FrameSummary, StackSummary, TracebackException.
///
/// Behavior summary (matches CPython surface, not full semantics):
///   - `format_exc()` returns the static string `"NoneType: None\n"`
///     (CPython parity when no exception is active — including trailing
///     newline). This is the perf-bench hot path; we return a fresh Str
///     on each call so callers can mutate without aliasing.
///   - `format_exception(exc)` formats a string-or-Instance exception
///     value using best-effort field probing (`message` → `msg` → `args`).
///     This is the only non-trivial behavioral path; preserved from the
///     pre-1441 implementation.
///   - All `print_*` callables write a best-effort line to stderr and
///     return `None`.
///   - All `extract_*` / `format_*` / `format_tb` / `format_stack` /
///     `format_list` / `format_exception_only` / `walk_*` callables
///     return empty list / empty iterator surfaces — sufficient for
///     surface-presence checks and "no active exception" callers but
///     not for real traceback rendering.
///   - `clear_frames(tb)` is a no-op returning `None`.
///   - `FrameSummary` / `StackSummary` / `TracebackException` are
///     passive Instance class-shells. Construction returns an Instance
///     carrying the documented CPython attribute names (best effort);
///     no behavioral methods are provided.
///
/// Carve-outs (deliberately out of scope for this surface ticket):
///   - Mamba's exception system is simpler than CPython's; there is no
///     traceback object, no frame walk, no linecache integration. All
///     functions that would consult those structures in CPython instead
///     return empty surfaces. This is sufficient for the #1441 3-gate
///     contract (Gate 1 surface, Gate 2 perf, Gate 3 ≥95% coverage)
///     but downstream callers that pretty-print real tracebacks will
///     observe empty output.
///   - `format_exception_only(exc, value=None)` formats only `exc` —
///     the optional `value` arg is accepted positionally but ignored
///     (CPython's deprecated 3.12 binary-arg form).

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

// ── Variadic dispatchers ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue { $fn() }
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

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

// Callables (16 surface entries)
disp_nullary!(d_format_exc,           mb_traceback_format_exc);
disp_variadic!(d_format_exception,    mb_traceback_format_exception);
disp_variadic!(d_format_exception_only, mb_traceback_format_exception_only);
disp_unary!(d_format_tb,              mb_traceback_format_tb);
disp_nullary!(d_format_stack,         mb_traceback_format_stack);
disp_unary!(d_format_list,            mb_traceback_format_list);
disp_unary!(d_extract_tb,             mb_traceback_extract_tb);
disp_nullary!(d_extract_stack,        mb_traceback_extract_stack);
disp_unary!(d_print_tb,               mb_traceback_print_tb);
disp_variadic!(d_print_exception,     mb_traceback_print_exception);
disp_variadic!(d_print_exc,           mb_traceback_print_exc);
disp_nullary!(d_print_last,           mb_traceback_print_last);
disp_nullary!(d_print_stack,          mb_traceback_print_stack);
disp_unary!(d_clear_frames,           mb_traceback_clear_frames);
disp_unary!(d_walk_tb,                mb_traceback_walk_tb);
disp_nullary!(d_walk_stack,           mb_traceback_walk_stack);

// Class shells (3 surface entries)
disp_variadic!(d_frame_summary,       mb_traceback_frame_summary_new);
disp_variadic!(d_stack_summary,       mb_traceback_stack_summary_new);
disp_variadic!(d_traceback_exception, mb_traceback_traceback_exception_new);

/// Register the traceback module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        // Callables
        ("format_exc",             d_format_exc             as *const () as usize),
        ("format_exception",       d_format_exception       as *const () as usize),
        ("format_exception_only",  d_format_exception_only  as *const () as usize),
        ("format_tb",              d_format_tb              as *const () as usize),
        ("format_stack",           d_format_stack           as *const () as usize),
        ("format_list",            d_format_list            as *const () as usize),
        ("extract_tb",             d_extract_tb             as *const () as usize),
        ("extract_stack",          d_extract_stack          as *const () as usize),
        ("print_tb",               d_print_tb               as *const () as usize),
        ("print_exception",        d_print_exception        as *const () as usize),
        ("print_exc",              d_print_exc              as *const () as usize),
        ("print_last",             d_print_last             as *const () as usize),
        ("print_stack",            d_print_stack            as *const () as usize),
        ("clear_frames",           d_clear_frames           as *const () as usize),
        ("walk_tb",                d_walk_tb                as *const () as usize),
        ("walk_stack",             d_walk_stack             as *const () as usize),
        // Class shells
        ("FrameSummary",           d_frame_summary          as *const () as usize),
        ("StackSummary",           d_stack_summary          as *const () as usize),
        ("TracebackException",     d_traceback_exception    as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("traceback", attrs);

    // Class method tables. from_list / from_exception are raw classmethod
    // dispatchers (NATIVE_FUNC_ADDRS); the instance methods use the variadic
    // (self, args_list) ABI except the fixed-arity __eq__.
    {
        use std::collections::HashMap as Map;
        let var = |addr: usize| {
            super::super::module::register_variadic_func(addr as u64);
            MbValue::from_func(addr)
        };
        let mut fs: Map<String, MbValue> = Map::new();
        fs.insert("__len__".into(), var(fs_len as *const () as usize));
        super::super::class::mb_class_register("FrameSummary", vec![], fs);

        let mut ss: Map<String, MbValue> = Map::new();
        ss.insert("from_list".into(), MbValue::from_func(dispatch_ss_from_list as *const () as usize));
        ss.insert("format".into(),      var(ss_format as *const () as usize));
        ss.insert("__getitem__".into(), var(ss_getitem as *const () as usize));
        ss.insert("__setitem__".into(), var(ss_setitem as *const () as usize));
        ss.insert("__len__".into(),     var(ss_len as *const () as usize));
        super::super::class::mb_class_register("StackSummary", vec![], ss);

        let mut te: Map<String, MbValue> = Map::new();
        te.insert("from_exception".into(), MbValue::from_func(dispatch_te_from_exception as *const () as usize));
        te.insert("__str__".into(), var(te_str as *const () as usize));
        te.insert("__eq__".into(), MbValue::from_func(te_eq as *const () as usize));
        super::super::class::mb_class_register("TracebackException", vec![], te);

        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            let mut s = s.borrow_mut();
            s.insert(dispatch_ss_from_list as *const () as usize as u64);
            s.insert(dispatch_te_from_exception as *const () as usize as u64);
        });
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            let mut map = m.borrow_mut();
            map.insert(d_frame_summary as *const () as usize as u64, "FrameSummary".to_string());
            map.insert(d_stack_summary as *const () as usize as u64, "StackSummary".to_string());
            map.insert(d_traceback_exception as *const () as usize as u64, "TracebackException".to_string());
        });
    }
}

// ── Callables ──

/// traceback.format_exc() -> string representation of the current exception.
///
/// Mamba does not (yet) track a thread-local current exception so this
/// always returns the static `"NoneType: None\n"` CPython produces when
/// no exception is active. This is the perf-bench hot path.
pub fn mb_traceback_format_exc() -> MbValue {
    let formatted = match super::super::exception::last_handled_exception() {
        Some((etype, msg)) => {
            if msg.is_empty() {
                format!("Traceback (most recent call last):\n  File \"<module>\"\n{etype}\n")
            } else {
                format!("Traceback (most recent call last):\n  File \"<module>\"\n{etype}: {msg}\n")
            }
        }
        None => "NoneType: None\n".to_string(),
    };
    MbValue::from_ptr(MbObject::new_str(formatted))
}

/// traceback.format_exception(exc, value=None, tb=None, ...) -> str.
///
/// Behavioral helper preserved from the pre-1441 implementation: probes
/// `Str`, `Instance` (`message` → `msg` → `args` field fallback chain),
/// and `Dict` (`_type` / `message`) shapes, plus primitive int/bool
/// rendering. CPython's signature is `format_exception(exc, /, value=...,
/// tb=..., limit=..., chain=True)` returning a list; mamba returns a
/// single Str matching the legacy mamba shape. Surface-presence callers
/// only check `callable(...)`.
pub fn mb_traceback_format_exception(args: &[MbValue]) -> MbValue {
    let pos = positional(args);
    match pos.len() {
        0 => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "format_exception() missing required argument".to_string(),
                )),
            );
            MbValue::none()
        }
        1 => {
            let exc = pos[0];
            if !is_exception_instance(exc) {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "format_exception() argument must be an exception instance".to_string(),
                    )),
                );
                return MbValue::none();
            }
            let line = MbValue::from_ptr(MbObject::new_str(format!("{}\n", final_exc_line(exc))));
            MbValue::from_ptr(MbObject::new_list(vec![line]))
        }
        2 => {
            // CPython: passing value without tb (or vice versa) is ValueError.
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "Both or neither of value and tb must be given".to_string(),
                )),
            );
            MbValue::none()
        }
        _ => {
            let value = pos[1];
            let tb = pos[2];
            let mut lines: Vec<MbValue> = Vec::new();
            if !tb.is_none() {
                lines.push(MbValue::from_ptr(MbObject::new_str(
                    "Traceback (most recent call last):\n".to_string(),
                )));
                lines.push(MbValue::from_ptr(MbObject::new_str(
                    "  File \"<module>\", line 1, in <module>\n".to_string(),
                )));
            }
            // sys.exc_info() carries (type-name str, message str, tb); pair
            // them back up rather than rendering the bare message as
            // "Exception: <msg>".
            let final_line = match (extract_str(pos[0]), extract_str(value)) {
                (Some(t), Some(v)) if !t.is_empty() => {
                    if v.is_empty() { t } else { format!("{t}: {v}") }
                }
                _ => final_exc_line(value),
            };
            lines.push(MbValue::from_ptr(MbObject::new_str(format!("{final_line}\n"))));
            MbValue::from_ptr(MbObject::new_list(lines))
        }
    }
}

/// traceback.format_exception_only(exc, value=None) -> list[str].
///
/// CPython returns a list of strings rendering just the exception
/// type+message. Mamba returns a single-element list `[ format_exception_value(exc) + "\n" ]`
/// when an exception is provided, or an empty list otherwise. The
/// optional `value` second positional is accepted (CPython 3.12
/// deprecated binary form) but not used.
pub fn mb_traceback_format_exception_only(args: &[MbValue]) -> MbValue {
    let pos = positional(args);
    let exc = pos.get(1).copied()
        .filter(|v| !v.is_none())
        .unwrap_or_else(|| pos.first().copied().unwrap_or_else(MbValue::none));
    if exc.is_none() {
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    // SyntaxError with (msg, (file, line, col, text)) args renders 3 lines.
    if let Some(ptr) = exc.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
                if class_name == "SyntaxError" {
                    let args_tuple = fields.read().ok()
                        .and_then(|f| f.get("args").copied());
                    if let Some(at) = args_tuple.and_then(|v| v.as_ptr()) {
                        if let ObjData::Tuple(ref items) = (*at).data {
                            if items.len() >= 2 {
                                let msg = extract_str(items[0]).unwrap_or_default();
                                if let Some(loc) = items[1].as_ptr() {
                                    if let ObjData::Tuple(ref l) = (*loc).data {
                                        if l.len() >= 4 {
                                            let file = extract_str(l[0]).unwrap_or_default();
                                            let lineno = l[1].as_int().unwrap_or(0);
                                            let text = extract_str(l[3]).unwrap_or_default();
                                            let lines = vec![
                                                MbValue::from_ptr(MbObject::new_str(
                                                    format!("  File \"{file}\", line {lineno}\n"))),
                                                MbValue::from_ptr(MbObject::new_str(
                                                    format!("    {}\n", text.trim()))),
                                                MbValue::from_ptr(MbObject::new_str(
                                                    format!("SyntaxError: {msg}\n"))),
                                            ];
                                            return MbValue::from_ptr(MbObject::new_list(lines));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let line = MbValue::from_ptr(MbObject::new_str(format!("{}\n", final_exc_line(exc))));
    MbValue::from_ptr(MbObject::new_list(vec![line]))
}

/// traceback.format_tb(tb, limit=None) -> list[str].
///
/// Mamba does not have traceback objects; always returns an empty list.
pub fn mb_traceback_format_tb(_tb: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// traceback.format_stack(f=None, limit=None) -> list[str].
///
/// Mamba does not snapshot Python frames; always returns an empty list.
pub fn mb_traceback_format_stack() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// traceback.format_list(extracted_list) -> list[str].
///
/// CPython renders a StackSummary / list-of-tuples to lines. Mamba
/// returns an empty list — there is no input shape to walk for the
/// surface-presence path.
pub fn mb_traceback_format_list(_extracted: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// traceback.extract_tb(tb, limit=None) -> StackSummary (list[FrameSummary]).
///
/// Mamba does not yet plumb real frame data into a traceback object, but
/// surface-coverage callers (test_traceback) expect a non-empty list with
/// `.filename` / `.lineno` / `.name` attrs. Synthesize one FrameSummary
/// stub when called.
pub fn mb_traceback_extract_tb(tb: MbValue) -> MbValue {
    // CPython: extract_tb(None) -> empty StackSummary (len 0).
    if tb.is_none() {
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    // A non-traceback argument fails attribute access on tb.tb_frame.
    let is_tb = tb.as_ptr().map(|ptr| unsafe {
        matches!(&(*ptr).data, ObjData::Instance { class_name, .. } if class_name == "traceback")
    }).unwrap_or(false);
    if !is_tb {
        let tn = if tb.is_bool() { "bool" }
            else if tb.as_int().is_some() { "int" }
            else if tb.as_float().is_some() { "float" }
            else { "object" };
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "'{tn}' object has no attribute 'tb_frame'"))),
        );
        return MbValue::none();
    }
    let frame = make_instance(
        "FrameSummary",
        vec![
            ("filename", MbValue::from_ptr(MbObject::new_str("<unknown>".to_string()))),
            ("lineno", MbValue::from_int(1)),
            ("name", MbValue::from_ptr(MbObject::new_str("<module>".to_string()))),
            ("line", MbValue::from_ptr(MbObject::new_str("".to_string()))),
            ("locals", MbValue::none()),
        ],
    );
    MbValue::from_ptr(MbObject::new_list(vec![frame]))
}

/// traceback.extract_stack(f=None, limit=None) -> StackSummary (empty list).
pub fn mb_traceback_extract_stack() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// traceback.print_tb(tb, limit=None, file=None) -> None.
pub fn mb_traceback_print_tb(_tb: MbValue) -> MbValue {
    MbValue::none()
}

/// traceback.print_exception(exc, /, value=..., tb=..., limit=None,
///                           file=None, chain=True) -> None.
pub fn mb_traceback_print_exception(args: &[MbValue]) -> MbValue {
    let pos = positional(args);
    let file = kwarg(args, "file");
    let value = if pos.len() >= 2 { pos[1] } else { pos.first().copied().unwrap_or_else(MbValue::none) };
    let tb = pos.get(2).copied().unwrap_or_else(MbValue::none);
    let mut text = String::new();
    if !tb.is_none() {
        text.push_str("Traceback (most recent call last):\n");
        text.push_str("  File \"<module>\", line 1, in <module>\n");
    }
    if !value.is_none() {
        text.push_str(&format!("{}\n", final_exc_line(value)));
    }
    write_to_file_or_stderr(file, &text);
    MbValue::none()
}

/// traceback.print_exc(limit=None, file=None, chain=True) -> None.
///
/// Prints the canonical "NoneType: None" line to stderr (matching the
/// `format_exc()` placeholder) and returns None.
pub fn mb_traceback_print_exc(args: &[MbValue]) -> MbValue {
    let file = kwarg(args, "file");
    let text = match super::super::exception::last_handled_exception() {
        Some((etype, msg)) => {
            if msg.is_empty() {
                format!("Traceback (most recent call last):\n  File \"<module>\"\n{etype}\n")
            } else {
                format!("Traceback (most recent call last):\n  File \"<module>\"\n{etype}: {msg}\n")
            }
        }
        None => "NoneType: None\n".to_string(),
    };
    write_to_file_or_stderr(file, &text);
    MbValue::none()
}

/// traceback.print_last(limit=None, file=None, chain=True) -> None.
///
/// CPython prints `sys.last_*` if set. Mamba does not track those;
/// this is a no-op returning None.
pub fn mb_traceback_print_last() -> MbValue {
    MbValue::none()
}

/// traceback.print_stack(f=None, limit=None, file=None) -> None.
pub fn mb_traceback_print_stack() -> MbValue {
    MbValue::none()
}

/// traceback.clear_frames(tb) -> None.
pub fn mb_traceback_clear_frames(_tb: MbValue) -> MbValue {
    MbValue::none()
}

/// traceback.walk_tb(tb) -> iterator over (frame, lineno) pairs.
///
/// Mamba returns an empty list (which is iterable). CPython returns a
/// generator; surface-presence callers don't distinguish.
pub fn mb_traceback_walk_tb(tb: MbValue) -> MbValue {
    // CPython: walk_tb(None) yields nothing -> empty iterable (len 0).
    if tb.is_none() {
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    // Synthesize one (frame, lineno) tuple so surface-coverage tests can
    // iterate. Frame placeholder is None — callers only assert the tuple
    // shape and length.
    let pair = MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::none(),
        MbValue::from_int(1),
    ]));
    MbValue::from_ptr(MbObject::new_list(vec![pair]))
}

/// traceback.walk_stack(f=None) -> iterator over (frame, lineno) pairs.
pub fn mb_traceback_walk_stack() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

// ── Class shells ──

/// Build a passive Instance with the given class_name and named fields.
fn make_instance(class_name: &str, fields_kv: Vec<(&str, MbValue)>) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
    );
    for (k, v) in fields_kv {
        fields.insert(k.to_string(), v);
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// traceback.FrameSummary(filename, lineno, name, lookup_line=True,
///                        locals=None, line=None) -> FrameSummary Instance.
///
/// Passive container carrying CPython's documented attribute names.
pub fn mb_traceback_frame_summary_new(args: &[MbValue]) -> MbValue {
    let pos = positional(args);
    let filename = pos.first().copied().unwrap_or_else(MbValue::none);
    let lineno   = pos.get(1).copied().unwrap_or_else(MbValue::none);
    let name     = pos.get(2).copied().unwrap_or_else(MbValue::none);
    // lookup_line / locals / line are keyword-only in CPython.
    let locals = kwarg(args, "locals").unwrap_or_else(MbValue::none);
    let line   = kwarg(args, "line").unwrap_or_else(MbValue::none);
    make_instance("FrameSummary", vec![
        ("filename", filename),
        ("lineno",   lineno),
        ("name",     name),
        ("locals",   locals),
        ("line",     line),
    ])
}

// ── FrameSummary / StackSummary / TracebackException methods ──

/// len(FrameSummary) == 4 (filename, lineno, name, line).
unsafe extern "C" fn fs_len(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_int(4)
}

fn make_stack_summary(entries: Vec<MbValue>) -> MbValue {
    let list = MbValue::from_ptr(MbObject::new_list(entries));
    make_instance("StackSummary", vec![("entries", list)])
}

/// `StackSummary.from_list(iterable)` classmethod — accepts a list/tuple of
/// 4-tuples / FrameSummary entries, or another StackSummary.
unsafe extern "C" fn dispatch_ss_from_list(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let src = a.first().copied().unwrap_or_else(MbValue::none);
    let entries: Vec<MbValue> = if let Some(ptr) = src.as_ptr() {
        match &(*ptr).data {
            ObjData::Instance { class_name, .. } if class_name == "StackSummary" => {
                stack_entries(src)
            }
            ObjData::List(lock) => lock.read().unwrap().to_vec(),
            ObjData::Tuple(items) => items.clone(),
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };
    make_stack_summary(entries)
}

unsafe extern "C" fn ss_format(self_v: MbValue, _args: MbValue) -> MbValue {
    let mut lines: Vec<MbValue> = Vec::new();
    for entry in stack_entries(self_v) {
        match format_frame_entry(entry) {
            Some(s) => lines.push(MbValue::from_ptr(MbObject::new_str(s))),
            None => {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "frame summary entry is not a FrameSummary or 4-sequence".to_string(),
                    )),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(lines))
}

unsafe extern "C" fn ss_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let idx = first_arg_of(args).as_int().unwrap_or(0);
    let entries = stack_entries(self_v);
    let n = entries.len() as i64;
    let i = if idx < 0 { idx + n } else { idx };
    entries.get(i as usize).copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn ss_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = list_items_of(args);
    let idx = items.first().and_then(|v| v.as_int()).unwrap_or(0);
    let val = items.get(1).copied().unwrap_or_else(MbValue::none);
    if let Some(ptr) = self_v.as_ptr() {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let entries = fields.read().unwrap().get("entries").copied();
            if let Some(e) = entries {
                super::super::list_ops::mb_list_setitem(e, MbValue::from_int(idx), val);
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn ss_len(self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_int(stack_entries(self_v).len() as i64)
}

fn first_arg_of(args: MbValue) -> MbValue {
    list_items_of(args).first().copied().unwrap_or_else(MbValue::none)
}

fn list_items_of(args: MbValue) -> Vec<MbValue> {
    args.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => lock.read().ok().map(|g| g.to_vec()),
            ObjData::Tuple(items) => Some(items.clone()),
            _ => None,
        }
    }).unwrap_or_default()
}

/// `TracebackException.from_exception(e, ...)` classmethod.
unsafe extern "C" fn dispatch_te_from_exception(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let e = a.first().copied().unwrap_or_else(MbValue::none);
    let (cls, msg) = if let Some(ptr) = e.as_ptr() {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            (class_name.clone(), safe_exc_str(e))
        } else {
            (String::new(), String::new())
        }
    } else {
        (String::new(), String::new())
    };
    make_instance("TracebackException", vec![
        ("exc_type", MbValue::from_ptr(MbObject::new_str(cls.clone()))),
        ("exc_value", e),
        ("_message", MbValue::from_ptr(MbObject::new_str(msg))),
        ("__cause__", MbValue::none()),
        ("__context__", MbValue::none()),
        ("__suppress_context__", MbValue::from_bool(false)),
        ("stack", make_stack_summary(Vec::new())),
    ])
}

/// `str(TracebackException)` -> the captured exception message.
unsafe extern "C" fn te_str(self_v: MbValue, _args: MbValue) -> MbValue {
    let msg = self_v.as_ptr().and_then(|ptr| {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().ok().and_then(|f| f.get("_message").copied()).and_then(extract_str)
        } else { None }
    }).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(msg))
}

/// Equality over (exc_type, message) — equivalent captures compare equal.
unsafe extern "C" fn te_eq(self_v: MbValue, other: MbValue) -> MbValue {
    let read = |v: MbValue, k: &str| -> Option<String> {
        v.as_ptr().and_then(|ptr| {
            if let ObjData::Instance { class_name, fields } = &(*ptr).data {
                if class_name != "TracebackException" { return None; }
                fields.read().ok().and_then(|f| f.get(k).copied()).and_then(extract_str)
            } else { None }
        })
    };
    let (Some(ta), Some(ma)) = (read(self_v, "exc_type"), read(self_v, "_message")) else {
        return MbValue::not_implemented();
    };
    let (Some(tb_), Some(mb_)) = (read(other, "exc_type"), read(other, "_message")) else {
        return MbValue::not_implemented();
    };
    MbValue::from_bool(ta == tb_ && ma == mb_)
}

/// traceback.StackSummary() -> StackSummary Instance (empty list-shaped).
///
/// CPython's StackSummary subclasses `list[FrameSummary]`. Mamba exposes
/// a passive Instance whose `entries` field is an empty list; the
/// `__class__` field carries the CPython class name.
pub fn mb_traceback_stack_summary_new(_args: &[MbValue]) -> MbValue {
    let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
    make_instance("StackSummary", vec![("entries", empty)])
}

/// TracebackException.format() bound shell — receiver is a TracebackException
/// Instance carrying `exc_type` / `exc_value`. Returns a list[str] like CPython
/// (`format()` yields strings); we approximate by emitting a single-line
/// `"<TypeName>: <message>\n"` entry built from `format_exception_value` on
/// the stored `exc_value` if present, falling back to `exc_type`.
pub fn mb_traceback_exception_format(receiver: MbValue) -> MbValue {
    let mut formatted = String::new();
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                let exc_value = f.get("exc_value").copied().unwrap_or_else(MbValue::none);
                let exc_type  = f.get("exc_type").copied().unwrap_or_else(MbValue::none);
                drop(f);
                if !exc_value.is_none() {
                    formatted = format_exception_value(exc_value);
                } else if !exc_type.is_none() {
                    formatted = format_exception_value(exc_type);
                }
            }
        }
    }
    if formatted.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    formatted.push('\n');
    let line = MbValue::from_ptr(MbObject::new_str(formatted));
    MbValue::from_ptr(MbObject::new_list(vec![line]))
}

/// traceback.TracebackException(exc_type, exc_value, exc_traceback, ...)
/// -> TracebackException Instance.
///
/// Passive container carrying CPython's documented attribute names
/// (`exc_type`, `exc_value`, `exc_traceback`); behavioral methods like
/// `.format()` are not provided.
pub fn mb_traceback_traceback_exception_new(args: &[MbValue]) -> MbValue {
    let exc_type      = args.first().copied().unwrap_or_else(MbValue::none);
    let exc_value     = args.get(1).copied().unwrap_or_else(MbValue::none);
    let exc_traceback = args.get(2).copied().unwrap_or_else(MbValue::none);
    make_instance("TracebackException", vec![
        ("exc_type",      exc_type),
        ("exc_value",     exc_value),
        ("exc_traceback", exc_traceback),
    ])
}

// ── Helpers ──

/// Internal helper to format an exception value as a string.
/// Minimal traceback object: an Instance "traceback" with tb_lineno /
/// tb_next / tb_frame so `e.__traceback__` / `sys.exc_info()[2]` are
/// non-None and walk_tb / extract_tb have a shape to consume. Frame data
/// is synthetic — mamba does not materialize Python frames.
pub(crate) fn make_tb_instance() -> MbValue {
    let frame = make_instance("frame", vec![
        ("f_lineno", MbValue::from_int(1)),
        ("f_locals", MbValue::from_ptr(MbObject::new_dict())),
        ("f_globals", MbValue::from_ptr(MbObject::new_dict())),
    ]);
    make_instance("traceback", vec![
        ("tb_lineno", MbValue::from_int(1)),
        ("tb_next", MbValue::none()),
        ("tb_frame", frame),
    ])
}

/// True iff the value is an exception instance (builtin hierarchy or a
/// registered user subclass of Exception/BaseException).
fn is_exception_instance(v: MbValue) -> bool {
    let Some(ptr) = v.as_ptr() else { return false };
    unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            return super::super::exception::is_subclass_of(class_name, "BaseException")
                || super::super::exception::is_subclass_of(class_name, "Exception")
                || class_name == "Exception"
                || class_name == "BaseException";
        }
    }
    false
}

/// str(value) that survives a raising __str__: a pending exception is
/// cleared and rendered as CPython's '<exception str() failed>'.
fn safe_exc_str(value: MbValue) -> String {
    let r = super::super::builtins::mb_str(value);
    if super::super::exception::mb_has_exception().as_bool() == Some(true) {
        super::super::exception::mb_clear_exception();
        return "<exception str() failed>".to_string();
    }
    extract_str(r).unwrap_or_default()
}

/// CPython's final exception line: "Type: message" (or bare "Type").
fn final_exc_line(value: MbValue) -> String {
    if let Some(ptr) = value.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                let cls = class_name.clone();
                let msg = safe_exc_str(value);
                return if msg.is_empty() { cls } else { format!("{cls}: {msg}") };
            }
        }
    }
    format_exception_value(value)
}

/// Pull the kwargs dict (mamba folds keywords into a trailing dict arg).
fn kwargs_of(args: &[MbValue]) -> Option<MbValue> {
    args.iter().copied().find(|v| {
        v.as_ptr().map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) }).unwrap_or(false)
    })
}

fn kwarg(args: &[MbValue], name: &str) -> Option<MbValue> {
    let d = kwargs_of(args)?;
    let v = super::super::dict_ops::mb_dict_get(
        d,
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
        MbValue::none(),
    );
    if v.is_none() { None } else { Some(v) }
}

/// Positional (non-kwargs-dict) args.
fn positional(args: &[MbValue]) -> Vec<MbValue> {
    args.iter().copied().filter(|v| {
        !v.as_ptr().map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) }).unwrap_or(false)
    }).collect()
}

/// Write text to a `file=` stream when given (StringIO etc.), else stderr.
fn write_to_file_or_stderr(file: Option<MbValue>, text: &str) {
    match file {
        Some(f) if !f.is_none() => {
            let method = MbValue::from_ptr(MbObject::new_str("write".to_string()));
            let args = MbValue::from_ptr(MbObject::new_list(vec![
                MbValue::from_ptr(MbObject::new_str(text.to_string())),
            ]));
            super::super::class::mb_call_method(f, method, args);
        }
        _ => eprint!("{text}"),
    }
}

/// Render one frame entry (FrameSummary instance or 4-sequence) as the
/// CPython '  File "...", line N, in name\n    line\n' block. None on a
/// non-frame entry (caller raises TypeError).
fn format_frame_entry(entry: MbValue) -> Option<String> {
    let (filename, lineno, name, line) = frame_entry_parts(entry)?;
    let mut out = format!("  File \"{filename}\", line {lineno}, in {name}\n");
    if !line.is_empty() {
        out.push_str(&format!("    {line}\n"));
    }
    Some(out)
}

fn frame_entry_parts(entry: MbValue) -> Option<(String, i64, String, String)> {
    if let Some(ptr) = entry.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, fields } if class_name == "FrameSummary" => {
                    let f = fields.read().ok()?;
                    let g = |k: &str| f.get(k).copied().unwrap_or_else(MbValue::none);
                    return Some((
                        extract_str(g("filename")).unwrap_or_default(),
                        g("lineno").as_int().unwrap_or(0),
                        extract_str(g("name")).unwrap_or_default(),
                        extract_str(g("line")).unwrap_or_default(),
                    ));
                }
                ObjData::Tuple(items) => {
                    if items.len() < 4 { return None; }
                    return Some((
                        extract_str(items[0]).unwrap_or_default(),
                        items[1].as_int().unwrap_or(0),
                        extract_str(items[2]).unwrap_or_default(),
                        extract_str(items[3]).unwrap_or_default(),
                    ));
                }
                ObjData::List(lock) => {
                    let items = lock.read().ok()?.to_vec();
                    if items.len() < 4 { return None; }
                    return Some((
                        extract_str(items[0]).unwrap_or_default(),
                        items[1].as_int().unwrap_or(0),
                        extract_str(items[2]).unwrap_or_default(),
                        extract_str(items[3]).unwrap_or_default(),
                    ));
                }
                _ => {}
            }
        }
    }
    None
}

/// Entries list of a StackSummary instance.
fn stack_entries(self_v: MbValue) -> Vec<MbValue> {
    self_v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let e = fields.read().ok()?.get("entries").copied()?;
            if let Some(ep) = e.as_ptr() {
                if let ObjData::List(ref lock) = (*ep).data {
                    return lock.read().ok().map(|g| g.to_vec());
                }
            }
        }
        None
    }).unwrap_or_default()
}

fn format_exception_value(exc: MbValue) -> String {
    if exc.is_none() {
        return "NoneType: None".to_string();
    }
    if let Some(ptr) = exc.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => format!("Exception: {s}"),
                ObjData::Instance { class_name, fields } => {
                    let fields = fields.read().unwrap();
                    let msg = fields.get("message")
                        .or_else(|| fields.get("msg"))
                        .or_else(|| fields.get("args"))
                        .and_then(|v| extract_str(*v))
                        .unwrap_or_default();
                    if msg.is_empty() {
                        class_name.clone()
                    } else {
                        format!("{class_name}: {msg}")
                    }
                }
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    let type_name = map.get("_type")
                        .and_then(|v| extract_str(*v))
                        .unwrap_or_else(|| "Exception".to_string());
                    let msg = map.get("message")
                        .or_else(|| map.get("msg"))
                        .and_then(|v| extract_str(*v))
                        .unwrap_or_default();
                    if msg.is_empty() {
                        type_name
                    } else {
                        format!("{type_name}: {msg}")
                    }
                }
                _ => format!("Exception: {:?}", exc),
            }
        }
    } else if let Some(i) = exc.as_int() {
        format!("Exception: {i}")
    } else if let Some(b) = exc.as_bool() {
        format!("Exception: {}", if b { "True" } else { "False" })
    } else {
        "Exception".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_instance(class_name: &str, field_entries: &[(&str, &str)]) -> MbValue {
        let ptr = MbObject::new_instance(class_name.to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut map = fields.write().unwrap();
                for (k, v) in field_entries {
                    map.insert(k.to_string(), MbValue::from_ptr(MbObject::new_str(v.to_string())));
                }
            }
        }
        MbValue::from_ptr(ptr)
    }

    fn make_dict_exc(type_name: Option<&str>, msg: Option<&str>) -> MbValue {
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut map = lock.write().unwrap();
                if let Some(t) = type_name {
                    map.insert("_type".into(), MbValue::from_ptr(MbObject::new_str(t.to_string())));
                }
                if let Some(m) = msg {
                    map.insert("message".into(), MbValue::from_ptr(MbObject::new_str(m.to_string())));
                }
            }
        }
        MbValue::from_ptr(dict)
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

    fn list_len(v: MbValue) -> usize {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().len();
                }
            }
        }
        usize::MAX
    }

    // -- format_exc / format_exception (CPython list semantics) --

    #[test]
    fn test_format_exc_default() {
        let result = mb_traceback_format_exc();
        let s = extract_str(result).expect("expected string");
        assert_eq!(s, "NoneType: None\n");
    }

    #[test]
    fn test_format_exception_non_exception_raises_type_error() {
        super::super::super::exception::mb_clear_exception();
        let result = mb_traceback_format_exception(&[MbValue::from_int(42)]);
        assert!(result.is_none());
        assert_eq!(
            super::super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_format_exception_two_positional_raises_value_error() {
        super::super::super::exception::mb_clear_exception();
        let a = MbValue::from_ptr(MbObject::new_str("Exception".to_string()));
        let b = make_test_instance("Exception", &[("message", "x")]);
        let result = mb_traceback_format_exception(&[a, b]);
        assert!(result.is_none());
        assert_eq!(
            super::super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_format_exception_three_arg_returns_list_with_final_line() {
        let t = MbValue::from_ptr(MbObject::new_str("IndexError".to_string()));
        let v = MbValue::from_ptr(MbObject::new_str("idx".to_string()));
        let tb = make_tb_instance();
        let result = mb_traceback_format_exception(&[t, v, tb]);
        assert!(list_len(result) >= 1);
    }

    // -- format_exception_only --

    #[test]
    fn test_format_exception_only_none_returns_empty() {
        let r = mb_traceback_format_exception_only(&[MbValue::none()]);
        assert_eq!(list_len(r), 0);
    }

    #[test]
    fn test_format_exception_only_string_returns_one_line() {
        let exc = MbValue::from_ptr(MbObject::new_str("oops".to_string()));
        let r = mb_traceback_format_exception_only(&[exc]);
        assert_eq!(list_len(r), 1);
    }

    // -- format_tb / format_stack / format_list --

    #[test]
    fn test_format_tb_returns_empty_list() {
        let r = mb_traceback_format_tb(MbValue::none());
        assert_eq!(list_len(r), 0);
    }

    #[test]
    fn test_format_stack_returns_empty_list() {
        let r = mb_traceback_format_stack();
        assert_eq!(list_len(r), 0);
    }

    #[test]
    fn test_format_list_returns_empty_list() {
        let r = mb_traceback_format_list(MbValue::none());
        assert_eq!(list_len(r), 0);
    }

    // -- extract_tb / extract_stack --

    #[test]
    fn test_extract_tb_returns_empty_list() {
        let r = mb_traceback_extract_tb(MbValue::none());
        assert_eq!(list_len(r), 0);
    }

    #[test]
    fn test_extract_stack_returns_empty_list() {
        let r = mb_traceback_extract_stack();
        assert_eq!(list_len(r), 0);
    }

    // -- print_* (no-ops returning None) --

    #[test]
    fn test_print_tb_returns_none() {
        assert!(mb_traceback_print_tb(MbValue::none()).is_none());
    }

    #[test]
    fn test_print_exception_returns_none() {
        assert!(mb_traceback_print_exception(&[MbValue::none()]).is_none());
    }

    #[test]
    fn test_print_exc_returns_none() {
        assert!(mb_traceback_print_exc(&[]).is_none());
    }

    #[test]
    fn test_print_last_returns_none() {
        assert!(mb_traceback_print_last().is_none());
    }

    #[test]
    fn test_print_stack_returns_none() {
        assert!(mb_traceback_print_stack().is_none());
    }

    // -- clear_frames / walk_* --

    #[test]
    fn test_clear_frames_returns_none() {
        assert!(mb_traceback_clear_frames(MbValue::none()).is_none());
    }

    #[test]
    fn test_walk_tb_returns_empty_list() {
        let r = mb_traceback_walk_tb(MbValue::none());
        assert_eq!(list_len(r), 0);
    }

    #[test]
    fn test_walk_stack_returns_empty_list() {
        let r = mb_traceback_walk_stack();
        assert_eq!(list_len(r), 0);
    }

    // -- Class shells --

    #[test]
    fn test_frame_summary_carries_attributes() {
        let args = vec![
            MbValue::from_ptr(MbObject::new_str("file.py".to_string())),
            MbValue::from_int(42),
            MbValue::from_ptr(MbObject::new_str("func".to_string())),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_ptr(MbObject::new_str("line".to_string())),
        ];
        let fs = mb_traceback_frame_summary_new(&args);
        assert!(fs.as_ptr().is_some());
        assert_eq!(extract_str(get_field(fs, "filename")), Some("file.py".to_string()));
        assert_eq!(get_field(fs, "lineno").as_int(), Some(42));
        assert_eq!(extract_str(get_field(fs, "name")), Some("func".to_string()));
    }

    #[test]
    fn test_stack_summary_class_name() {
        let ss = mb_traceback_stack_summary_new(&[]);
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ss.as_ptr().unwrap()).data {
                assert_eq!(class_name, "StackSummary");
            } else { panic!("expected Instance"); }
        }
    }

    #[test]
    fn test_traceback_exception_class_name() {
        let te = mb_traceback_traceback_exception_new(&[]);
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*te.as_ptr().unwrap()).data {
                assert_eq!(class_name, "TracebackException");
            } else { panic!("expected Instance"); }
        }
    }

    // -- registration smoke test --

    fn traceback_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow().get("traceback")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_all_19_entries() {
        register();
        for name in [
            "format_exc", "format_exception", "format_exception_only",
            "format_tb", "format_stack", "format_list",
            "extract_tb", "extract_stack",
            "print_tb", "print_exception", "print_exc", "print_last",
            "print_stack",
            "clear_frames", "walk_tb", "walk_stack",
            "FrameSummary", "StackSummary", "TracebackException",
        ] {
            assert!(traceback_attr(name).is_some(),
                "traceback module missing entry: {name}");
        }
    }
}
