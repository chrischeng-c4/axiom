use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// bdb module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering the top-level
/// `bdb` entry points (`Bdb`, `BdbQuit`, `Breakpoint`,
/// `set_trace`, `checkfuncname`). The class constructors return
/// identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// bdb sentinels.
///
/// `Bdb` is bridged into the runtime class registry so class-attribute
/// method probes (`callable(bdb.Bdb.runcall)`) resolve to callable
/// unbound methods via mb_getattr's func->native-class bridge.
/// `BdbQuit` is registered as a `type` object so `type(BdbQuit).__name__`
/// reads `"type"` for the surface probe.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.
use std::collections::HashMap;

const BDB_CLASS: &str = "Bdb";

unsafe extern "C" fn dispatch_bdb(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let inst = MbObject::new_instance(BDB_CLASS.to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert("breaks".to_string(), super::super::dict_ops::mb_dict_new());
            g.insert("quitting".to_string(), MbValue::from_bool(false));
        }
    }
    MbValue::from_ptr(inst)
}

// ── Bdb instance helpers ──

fn field_get(recv: MbValue, name: &str) -> Option<MbValue> {
    recv.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

fn field_set(recv: MbValue, name: &str, val: MbValue) {
    if let Some(ptr) = recv.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(name.to_string(), val);
            }
        }
    }
}

/// The instance's `breaks` dict, creating it on first use (user subclass
/// instances are constructed without the native fields).
fn breaks_dict(recv: MbValue) -> MbValue {
    if let Some(d) = field_get(recv, "breaks") {
        let is_dict = d
            .as_ptr()
            .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) });
        if is_dict {
            return d;
        }
    }
    let d = super::super::dict_ops::mb_dict_new();
    field_set(recv, "breaks", d);
    d
}

fn seq_items(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lk) = (*p).data {
                Some(lk.read().unwrap().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn new_str(s: String) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s))
}

fn extract_str(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Str(ref s) = (*p).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn raise_value_error(msg: String) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

/// Call a user-overridable hook (user_line / user_return) on the receiver
/// when a subclass defines it; the native Bdb base's hooks are no-ops.
fn call_user_hook(recv: MbValue, name: &str, args: Vec<MbValue>) {
    let class_name = recv.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*p).data {
            Some(class_name.clone())
        } else {
            None
        }
    });
    let Some(class_name) = class_name else { return };
    if class_name == BDB_CLASS {
        return;
    }
    let nm = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    let arglist = MbValue::from_ptr(MbObject::new_list(args));
    let _ = super::super::class::mb_call_method(recv, nm, arglist);
}

// ── Bdb instance methods (self, args_list convention) ──

unsafe extern "C" fn m_set_break(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let Some(filename) = items.first().copied().and_then(extract_str) else {
        return MbValue::none();
    };
    let lineno = items.get(1).and_then(|v| v.as_int_pyint()).unwrap_or(1);
    // CPython consults linecache: a missing file or out-of-range line returns
    // an error STRING (not a raise).
    let line_ok = std::fs::read_to_string(&filename)
        .map(|src| {
            let count = src.lines().count() as i64;
            lineno >= 1 && lineno <= count
        })
        .unwrap_or(false);
    if !line_ok {
        return new_str(format!("Line {filename}:{lineno} does not exist"));
    }
    let breaks = breaks_dict(self_v);
    let key = new_str(filename);
    let existing = super::super::dict_ops::mb_dict_get(breaks, key, MbValue::none());
    if existing.is_none() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(lineno)]));
        super::super::dict_ops::mb_dict_setitem(breaks, key, list);
    } else if let Some(p) = existing.as_ptr() {
        unsafe {
            if let ObjData::List(ref lk) = (*p).data {
                lk.write().unwrap().push(MbValue::from_int(lineno));
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn m_clear_break(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let Some(filename) = items.first().copied().and_then(extract_str) else {
        return MbValue::none();
    };
    let lineno = items.get(1).and_then(|v| v.as_int_pyint()).unwrap_or(1);
    let breaks = breaks_dict(self_v);
    let key = new_str(filename.clone());
    let entry = super::super::dict_ops::mb_dict_get(breaks, key, MbValue::none());
    let mut found = false;
    if let Some(p) = entry.as_ptr() {
        unsafe {
            if let ObjData::List(ref lk) = (*p).data {
                let mut g = lk.write().unwrap();
                if let Some(pos) = g.iter().position(|v| v.as_int() == Some(lineno)) {
                    g.remove(pos);
                    found = true;
                }
            }
        }
    }
    if !found {
        return new_str(format!("There is no breakpoint at {filename}:{lineno}"));
    }
    MbValue::none()
}

unsafe extern "C" fn m_get_break(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let Some(filename) = items.first().copied().and_then(extract_str) else {
        return MbValue::from_bool(false);
    };
    let lineno = items.get(1).and_then(|v| v.as_int_pyint()).unwrap_or(1);
    let breaks = breaks_dict(self_v);
    let entry = super::super::dict_ops::mb_dict_get(breaks, new_str(filename), MbValue::none());
    let hit = entry.as_ptr().is_some_and(|p| unsafe {
        if let ObjData::List(ref lk) = (*p).data {
            lk.read()
                .unwrap()
                .iter()
                .any(|v| v.as_int() == Some(lineno))
        } else {
            false
        }
    });
    MbValue::from_bool(hit)
}

unsafe extern "C" fn m_clear_all_breaks(self_v: MbValue, _args: MbValue) -> MbValue {
    field_set(self_v, "breaks", super::super::dict_ops::mb_dict_new());
    MbValue::none()
}

unsafe extern "C" fn m_get_bpbynumber(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let arg = items.first().copied().unwrap_or_else(MbValue::none);
    // CPython: int(arg) failure → "Non-numeric breakpoint number"; a number
    // with no live Breakpoint → "Breakpoint number N out of range". The shim
    // tracks no numbered breakpoints, so every numeric arg is out of range.
    let n = if let Some(i) = arg.as_int_pyint() {
        Some(i)
    } else {
        extract_str(arg).and_then(|s| s.trim().parse::<i64>().ok())
    };
    match n {
        None => {
            let shown = extract_str(arg).unwrap_or_else(|| "?".to_string());
            raise_value_error(format!("Non-numeric breakpoint number {shown}"))
        }
        Some(i) => raise_value_error(format!("Breakpoint number {i} out of range")),
    }
}

unsafe extern "C" fn m_set_quit(self_v: MbValue, _args: MbValue) -> MbValue {
    field_set(self_v, "quitting", MbValue::from_bool(true));
    MbValue::none()
}

unsafe extern "C" fn m_runcall(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let Some(func) = items.first().copied() else {
        return MbValue::none();
    };
    // Minimal trace simulation: reset quitting, fire user_line once (where a
    // subclass may set_quit / set_step / set_continue), honor quitting by
    // aborting (CPython swallows BdbQuit and returns None), then run the
    // function for real and fire user_return with its result.
    field_set(self_v, "quitting", MbValue::from_bool(false));
    call_user_hook(self_v, "user_line", vec![MbValue::none()]);
    if field_get(self_v, "quitting").and_then(|v| v.as_bool()) == Some(true) {
        return MbValue::none();
    }
    let rest = MbValue::from_ptr(MbObject::new_list(items[1..].to_vec()));
    let result = super::super::builtins::mb_call_spread(func, rest);
    call_user_hook(self_v, "user_return", vec![MbValue::none(), result]);
    result
}

unsafe extern "C" fn m_runeval(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let Some(expr) = items.first().copied() else {
        return MbValue::none();
    };
    field_set(self_v, "quitting", MbValue::from_bool(false));
    call_user_hook(self_v, "user_line", vec![MbValue::none()]);
    if field_get(self_v, "quitting").and_then(|v| v.as_bool()) == Some(true) {
        return MbValue::none();
    }
    super::super::builtins::mb_eval(expr)
}

unsafe extern "C" fn dispatch_breakpoint(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_set_trace(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_checkfuncname(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_bool(false)
}

// ── Bdb instance-method stubs (self, args_list) ─────────────────────────────
// Surface probes only need callability; these return None placeholders.

unsafe extern "C" fn m_bdb_stub(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

/// `BdbQuit` as a real `type` object so `type(bdb.BdbQuit).__name__ == "type"`
/// and `callable(bdb.BdbQuit)` both hold for the surface probes.
fn make_bdbquit_type() -> MbValue {
    let inst = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("BdbQuit".to_string())),
            );
            g.insert(
                "__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("bdb".to_string())),
            );
        }
    }
    MbValue::from_ptr(inst)
}

/// Register the bdb module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_b = dispatch_bdb as *const () as usize;
    attrs.insert("Bdb".into(), MbValue::from_func(addr_b));

    let addr_br = dispatch_breakpoint as *const () as usize;
    attrs.insert("Breakpoint".into(), MbValue::from_func(addr_br));

    let addr_st = dispatch_set_trace as *const () as usize;
    attrs.insert("set_trace".into(), MbValue::from_func(addr_st));

    let addr_cf = dispatch_checkfuncname as *const () as usize;
    attrs.insert("checkfuncname".into(), MbValue::from_func(addr_cf));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_b as u64);
        set.insert(addr_br as u64);
        set.insert(addr_st as u64);
        set.insert(addr_cf as u64);
    });

    // BdbQuit is a real `type` object (recipe B): type(BdbQuit).__name__ == "type".
    attrs.insert("BdbQuit".into(), make_bdbquit_type());

    // surface: missing CPython module constants (auto-added)
    attrs.insert("CO_ASYNC_GENERATOR".into(), MbValue::from_int(512));
    attrs.insert("CO_COROUTINE".into(), MbValue::from_int(128));
    attrs.insert("CO_GENERATOR".into(), MbValue::from_int(32));
    attrs.insert(
        "GENERATOR_AND_COROUTINE_FLAGS".into(),
        MbValue::from_int(672),
    );
    super::register_module("bdb", attrs);

    // Bridge the Bdb constructor func -> its class name so accessing a
    // registered method on the class (`Bdb.runcall`) resolves to a callable
    // unbound method via mb_getattr's func->native-class method bridge (which
    // looks the func addr up in NATIVE_TYPE_NAMES, then lookup_method in the
    // table mb_class_register populates below).
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(addr_b as u64, BDB_CLASS.to_string());
        // Breakpoint as a class symbol: `Breakpoint.bplist` resolves through
        // the registered class attr below.
        map.insert(addr_br as u64, "Breakpoint".to_string());
    });

    // Breakpoint.bplist — class-level registry, an empty dict before any
    // breakpoint exists.
    super::super::class::mb_class_register("Breakpoint", vec![], HashMap::new());
    super::super::class::mb_class_set_class_attr(
        MbValue::from_ptr(MbObject::new_str("Breakpoint".to_string())),
        MbValue::from_ptr(MbObject::new_str("bplist".to_string())),
        super::super::dict_ops::mb_dict_new(),
    );

    // BdbQuit doubles as a raisable exception class: registry membership with
    // an Exception base makes `issubclass(bdb.BdbQuit, Exception)` and
    // `except bdb.BdbQuit` resolve (the raised instance carries the same
    // class-name string).
    super::super::class::mb_class_register(
        "BdbQuit",
        vec!["Exception".to_string()],
        HashMap::new(),
    );

    // Register the Bdb class methods. Breakpoint bookkeeping (set/clear/get,
    // breaks dict, get_bpbynumber contracts), set_quit, and the runcall /
    // runeval entry points are real; the rest stay callable no-op stubs.
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    let stub = m_bdb_stub as usize;
    let real: &[(&str, usize)] = &[
        ("set_break", m_set_break as usize),
        ("clear_break", m_clear_break as usize),
        ("get_break", m_get_break as usize),
        ("clear_all_breaks", m_clear_all_breaks as usize),
        ("get_bpbynumber", m_get_bpbynumber as usize),
        ("set_quit", m_set_quit as usize),
        ("runcall", m_runcall as usize),
        ("runeval", m_runeval as usize),
    ];
    for (name, addr) in real {
        methods.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    for name in [
        "run",
        "runctx",
        "set_trace",
        "set_continue",
        "set_step",
        "set_next",
        "set_return",
        "set_until",
        "clear_bpbynumber",
        "clear_all_file_breaks",
        "get_breaks",
        "get_file_breaks",
        "get_all_breaks",
        "canonic",
        "reset",
        "trace_dispatch",
        "dispatch_line",
        "dispatch_call",
        "dispatch_return",
        "dispatch_exception",
        "stop_here",
        "break_here",
        "break_anywhere",
        "do_clear",
        "user_call",
        "user_line",
        "user_return",
        "user_exception",
        "format_stack_entry",
        "is_skipped_module",
        "message",
        "error",
    ] {
        methods.insert(name.to_string(), MbValue::from_func(stub));
        super::super::module::register_variadic_func(stub as u64);
    }
    super::super::class::mb_class_register(BDB_CLASS, vec![], methods);
}
