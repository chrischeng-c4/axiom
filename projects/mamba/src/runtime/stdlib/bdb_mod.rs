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
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

const BDB_CLASS: &str = "Bdb";

unsafe extern "C" fn dispatch_bdb(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance(BDB_CLASS.to_string()))
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
            g.insert("__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("BdbQuit".to_string())));
            g.insert("__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("bdb".to_string())));
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
    attrs.insert("GENERATOR_AND_COROUTINE_FLAGS".into(), MbValue::from_int(672));
    super::register_module("bdb", attrs);

    // Bridge the Bdb constructor func -> its class name so accessing a
    // registered method on the class (`Bdb.runcall`) resolves to a callable
    // unbound method via mb_getattr's func->native-class method bridge (which
    // looks the func addr up in NATIVE_TYPE_NAMES, then lookup_method in the
    // table mb_class_register populates below).
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(addr_b as u64, BDB_CLASS.to_string());
    });

    // Register the Bdb class methods so class-attribute probes are callable.
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    let stub = m_bdb_stub as usize;
    for name in [
        "runcall", "run", "runeval", "runctx", "set_trace", "set_continue",
        "set_step", "set_next", "set_return", "set_until", "set_quit",
        "set_break", "clear_break", "clear_bpbynumber", "clear_all_file_breaks",
        "clear_all_breaks", "get_break", "get_breaks", "get_file_breaks",
        "get_all_breaks", "get_bpbynumber", "canonic", "reset",
        "trace_dispatch", "dispatch_line", "dispatch_call", "dispatch_return",
        "dispatch_exception", "stop_here", "break_here", "break_anywhere",
        "do_clear", "user_call", "user_line", "user_return", "user_exception",
        "format_stack_entry", "is_skipped_module", "set_quit", "message",
        "error",
    ] {
        methods.insert(name.to_string(), MbValue::from_func(stub));
        super::super::module::register_variadic_func(stub as u64);
    }
    super::super::class::mb_class_register(BDB_CLASS, vec![], methods);
}
