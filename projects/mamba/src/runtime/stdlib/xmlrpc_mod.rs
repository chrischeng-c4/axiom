use super::super::rc::MbObject;
use super::super::value::MbValue;
/// xmlrpc.client + xmlrpc.server modules for Mamba (#1261 long-tail).
///
/// Surface-only shim for the two XML-RPC submodules. Mamba doesn't host
/// a real XML-RPC stack — `ServerProxy(url)` returns a dict, `Fault`
/// returns a dict, etc. Goal here is to short-circuit the import-time
/// probe chain (legacy library integrations + setuptools/distutils
/// often import xmlrpc.client) without crashing.
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
const SHELL_POOL_SIZE: usize = 36;
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
    shell_09, shell_10, shell_11, shell_12, shell_13, shell_14, shell_15, shell_16, shell_17,
    shell_18, shell_19, shell_20, shell_21, shell_22, shell_23, shell_24, shell_25, shell_26,
    shell_27, shell_28, shell_29, shell_30, shell_31, shell_32, shell_33, shell_34, shell_35,
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

unsafe extern "C" fn dispatch_dumps(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(
        "<?xml version='1.0'?>\n<methodCall></methodCall>\n".to_string(),
    ))
}

unsafe extern "C" fn dispatch_loads(_a: *const MbValue, _n: usize) -> MbValue {
    let empty_list = || MbValue::from_ptr(MbObject::new_list(Vec::new()));
    MbValue::from_ptr(MbObject::new_list(vec![empty_list(), MbValue::none()]))
}

pub fn register() {
    NEXT_SHELL_SLOT.with(|c| c.set(0));

    register_xmlrpc_root();
    register_xmlrpc_client();
    register_xmlrpc_server();
}

fn register_xmlrpc_root() {
    // CPython's xmlrpc package is just the namespace; submodules carry the surface.
    super::register_module("xmlrpc", HashMap::new());
}

fn register_xmlrpc_client() {
    let mut attrs = HashMap::new();

    // Protocol error constants — CPython exposes a handful of named ints.
    attrs.insert("MAXINT".into(), MbValue::from_int(2_147_483_647));
    attrs.insert("MININT".into(), MbValue::from_int(-2_147_483_648));

    let dumps = dispatch_dumps as *const () as usize;
    let loads = dispatch_loads as *const () as usize;
    let mut shell_addrs: Vec<usize> = Vec::new();

    let class_shells: &[&str] = &[
        "ServerProxy",
        "Server",
        "Transport",
        "SafeTransport",
        "MultiCall",
        "MultiCallIterator",
        "Marshaller",
        "Unmarshaller",
        "ResponseError",
        "Fault",
        "ProtocolError",
        "Binary",
        "Boolean",
        "DateTime",
        "Error",
        "_Method",
        "GZipDecodedResponse",
        "GzipDecodedResponse",
        "ExpatParser",
        "_NullMethod",
    ];
    for name in class_shells {
        let f = shell_addr(next_shell_slot());
        shell_addrs.push(f);
        attrs.insert((*name).into(), MbValue::from_func(f));
    }
    let getparser_addr = shell_addr(next_shell_slot());
    let escape_addr = shell_addr(next_shell_slot());
    shell_addrs.push(getparser_addr);
    shell_addrs.push(escape_addr);
    let dispatchers: &[(&str, usize)] = &[
        ("dumps", dumps),
        ("loads", loads),
        ("getparser", getparser_addr),
        ("escape", escape_addr),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in &shell_addrs {
            set.insert(*a as u64);
        }
        set.insert(dumps as u64);
        set.insert(loads as u64);
    });
    // surface: missing CPython module constants (auto-added)
    attrs.insert("APPLICATION_ERROR".into(), MbValue::from_int(-32500));
    attrs.insert("INTERNAL_ERROR".into(), MbValue::from_int(-32603));
    attrs.insert("INVALID_ENCODING_CHAR".into(), MbValue::from_int(-32702));
    attrs.insert("INVALID_METHOD_PARAMS".into(), MbValue::from_int(-32602));
    attrs.insert("INVALID_XMLRPC".into(), MbValue::from_int(-32600));
    attrs.insert("METHOD_NOT_FOUND".into(), MbValue::from_int(-32601));
    attrs.insert("NOT_WELLFORMED_ERROR".into(), MbValue::from_int(-32700));
    attrs.insert("PARSE_ERROR".into(), MbValue::from_int(-32700));
    attrs.insert("SERVER_ERROR".into(), MbValue::from_int(-32600));
    attrs.insert("SYSTEM_ERROR".into(), MbValue::from_int(-32400));
    attrs.insert("TRANSPORT_ERROR".into(), MbValue::from_int(-32300));
    attrs.insert("UNSUPPORTED_ENCODING".into(), MbValue::from_int(-32701));
    super::register_module("xmlrpc.client", attrs);
}

fn register_xmlrpc_server() {
    let mut attrs = HashMap::new();
    let class_shells: &[&str] = &[
        "SimpleXMLRPCRequestHandler",
        "SimpleXMLRPCDispatcher",
        "SimpleXMLRPCServer",
        "MultiPathXMLRPCServer",
        "DocXMLRPCRequestHandler",
        "DocXMLRPCServer",
        "ServerHTMLDoc",
        "XMLRPCDocGenerator",
        "CGIXMLRPCRequestHandler",
        "list_public_methods",
        "resolve_dotted_attribute",
    ];
    let mut shell_addrs: Vec<usize> = Vec::new();
    for name in class_shells {
        let f = shell_addr(next_shell_slot());
        shell_addrs.push(f);
        attrs.insert((*name).into(), MbValue::from_func(f));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in &shell_addrs {
            set.insert(*a as u64);
        }
    });
    super::register_module("xmlrpc.server", attrs);
}
