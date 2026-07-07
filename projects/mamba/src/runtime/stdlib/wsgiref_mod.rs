use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// wsgiref module + submodules for Mamba (#1261 long-tail).
///
/// Surface-only shim covering the WSGI utilities Flask / Werkzeug /
/// gunicorn import at probe time. Mamba doesn't host a real WSGI loop —
/// each dispatcher returns an identity-stable sentinel. The five
/// submodules registered are `wsgiref`, `wsgiref.util`,
/// `wsgiref.headers`, `wsgiref.simple_server`, `wsgiref.validate`,
/// `wsgiref.handlers`. All callable dispatchers use the native
/// extern "C" ABI with NATIVE_FUNC_ADDRS registration.
use std::collections::HashMap;

// #1040 follow-up: this file's `dispatch_class_shell` used to be handed out
// as the SAME function address to every class-shell name registered here,
// across every `register_*` call in this file. Because FUNC_NAMES/
// NATIVE_FUNC_ADDRS are address-keyed, whichever name registered last (in
// HashMap iteration order, which is nondeterministic per process) won
// `X.__name__` for every other class sharing that address -- the same
// #962/#954 symptom. The fix: give every class-shell name a genuinely
// distinct function pointer, drawn from a pool of `SHELL_POOL_SIZE`
// individually fold-immune trivial stub functions, indexed via a
// thread-local "next free slot" counter (`next_shell_slot`) so every call
// site simply draws a fresh slot per name -- no manual per-call `pool_start`
// bookkeeping required, since `register()` runs registration sequentially
// on a single thread at module-init time.
//
// IMPORTANT: this pool does NOT use `icf_guard!()` directly. That macro
// derives its fingerprint from `module_path!()`/`line!()`/`column!()`, which
// are resolved at the span of the *macro definition's* literal tokens -- for
// a single `macro_rules!` invocation that expands a `$(...)* ` repetition
// into N functions, every repetition shares that ONE span, so
// `line!()`/`column!()` come back IDENTICAL for all N and `icf_guard!()`
// silently fails to discriminate them. LLVM then folds all "distinct"
// shells back onto a single address, reproducing the exact bug one level
// down. The fix here instead fingerprints on `stringify!($name)`, which DOES
// vary per repetition (driven by the captured `$name` token's text, not by
// span), giving every pool slot a genuinely distinct compiled body.
const SHELL_POOL_SIZE: usize = 14;
type ShellFn = unsafe extern "C" fn(*const MbValue, usize) -> MbValue;

macro_rules! def_shell_pool {
    ($($name:ident),* $(,)?) => {
        $(
            unsafe extern "C" fn $name(_a: *const MbValue, _n: usize) -> MbValue {
                ::std::hint::black_box(crate::runtime::module::icf_fingerprint(concat!(
                    module_path!(),
                    "::",
                    stringify!($name)
                )));
                MbValue::from_ptr(MbObject::new_dict())
            }
        )*
        const SHELL_POOL: [ShellFn; SHELL_POOL_SIZE] = [$($name),*];
    };
}
def_shell_pool!(
    shell_00, shell_01, shell_02, shell_03, shell_04, shell_05, shell_06, shell_07, shell_08,
    shell_09, shell_10, shell_11, shell_12, shell_13,
);

/// Pool slot at `idx` as a raw function-pointer address.
fn shell_addr(idx: usize) -> usize {
    SHELL_POOL[idx] as usize
}

thread_local! {
    static NEXT_SHELL_SLOT: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

/// Draw the next unused pool slot index. `register()` runs sequentially on
/// a single thread at module-init time, so a simple monotonic counter gives
/// every class-shell name a fresh, non-overlapping slot with no manual
/// per-call range bookkeeping.
fn next_shell_slot() -> usize {
    NEXT_SHELL_SLOT.with(|c| {
        let v = c.get();
        assert!(
            v < SHELL_POOL_SIZE,
            "shell pool exhausted (SHELL_POOL_SIZE={}); bump it",
            SHELL_POOL_SIZE
        );
        c.set(v + 1);
        v
    })
}

unsafe extern "C" fn dispatch_noop(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_empty_list(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

unsafe extern "C" fn dispatch_empty_dict(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_request_uri(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("/".to_string()))
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn make_type_obj(name: &str, module: &str) -> MbValue {
    let obj = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*obj).data {
            let mut map = fields.write().unwrap();
            map.insert("__name__".to_string(), new_str(name));
            map.insert("__qualname__".to_string(), new_str(name));
            map.insert("__module__".to_string(), new_str(module));
        }
    }
    MbValue::from_ptr(obj)
}

fn extract_args(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn is_bytes_like(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false)
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str("TypeError"), new_str(msg));
    MbValue::none()
}

unsafe extern "C" fn base_handler_write(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_args(args);
    let data = items.first().copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(data) {
        return raise_type_error("BaseHandler.write() argument must be bytes-like");
    }
    MbValue::none()
}

fn register_variadic_method_class(class_name: &str, method_name: &str, addr: usize) {
    super::super::module::register_variadic_func(addr as u64);
    let mut methods = HashMap::new();
    methods.insert(method_name.to_string(), MbValue::from_func(addr));
    super::super::class::mb_class_register(class_name, vec!["object".to_string()], methods);
}

pub fn register() {
    NEXT_SHELL_SLOT.with(|c| c.set(0));

    register_wsgiref_root();
    register_wsgiref_util();
    register_wsgiref_headers();
    register_wsgiref_simple_server();
    register_wsgiref_validate();
    register_wsgiref_handlers();
}

fn register_addrs(addrs: &[usize]) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in addrs {
            set.insert(*a as u64);
        }
    });
}

fn register_wsgiref_root() {
    // The wsgiref root is just a marker; CPython exposes the submodules
    // by name only. The umbrella mirrors that.
    let attrs = HashMap::new();
    super::register_module("wsgiref", attrs);
}

fn register_wsgiref_util() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        (
            "setup_testing_defaults",
            dispatch_noop as *const () as usize,
        ),
        ("guess_scheme", dispatch_empty_str as *const () as usize),
        ("application_uri", dispatch_empty_str as *const () as usize),
        ("request_uri", dispatch_request_uri as *const () as usize),
        ("shift_path_info", dispatch_empty_str as *const () as usize),
        ("is_hop_by_hop", dispatch_noop as *const () as usize),
        ("FileWrapper", shell_addr(next_shell_slot())),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.util", attrs);
}

fn register_wsgiref_headers() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[("Headers", shell_addr(next_shell_slot()))];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.headers", attrs);
}

fn register_wsgiref_simple_server() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("WSGIServer", shell_addr(next_shell_slot())),
        ("WSGIRequestHandler", shell_addr(next_shell_slot())),
        ("ServerHandler", shell_addr(next_shell_slot())),
        ("make_server", shell_addr(next_shell_slot())),
        ("demo_app", dispatch_empty_list as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    // surface: missing CPython module constants (auto-added)
    attrs.insert(
        "server_version".into(),
        MbValue::from_ptr(MbObject::new_str("WSGIServer/0.2".to_string())),
    );
    attrs.insert(
        "software_version".into(),
        MbValue::from_ptr(MbObject::new_str(
            "WSGIServer/0.2 CPython/3.12.11".to_string(),
        )),
    );
    attrs.insert(
        "sys_version".into(),
        MbValue::from_ptr(MbObject::new_str("CPython/3.12.11".to_string())),
    );
    super::register_module("wsgiref.simple_server", attrs);
}

fn register_wsgiref_validate() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("validator", shell_addr(next_shell_slot())),
        ("WSGIWarning", shell_addr(next_shell_slot())),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.validate", attrs);
}

fn register_wsgiref_handlers() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("SimpleHandler", shell_addr(next_shell_slot())),
        ("BaseCGIHandler", shell_addr(next_shell_slot())),
        ("CGIHandler", shell_addr(next_shell_slot())),
        ("IISCGIHandler", shell_addr(next_shell_slot())),
        ("read_environ", dispatch_empty_dict as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    attrs.insert(
        "BaseHandler".into(),
        make_type_obj("BaseHandler", "wsgiref.handlers"),
    );
    register_variadic_method_class(
        "BaseHandler",
        "write",
        base_handler_write as *const () as usize,
    );
    super::register_module("wsgiref.handlers", attrs);
}
