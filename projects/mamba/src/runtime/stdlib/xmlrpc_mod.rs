/// xmlrpc.client + xmlrpc.server modules for Mamba (#1261 long-tail).
///
/// Surface-only shim for the two XML-RPC submodules. Mamba doesn't host
/// a real XML-RPC stack — `ServerProxy(url)` returns a dict, `Fault`
/// returns a dict, etc. Goal here is to short-circuit the import-time
/// probe chain (legacy library integrations + setuptools/distutils
/// often import xmlrpc.client) without crashing.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_dumps(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("<?xml version='1.0'?>\n<methodCall></methodCall>\n".to_string()))
}

unsafe extern "C" fn dispatch_loads(_a: *const MbValue, _n: usize) -> MbValue {
    let empty_list = || MbValue::from_ptr(MbObject::new_list(Vec::new()));
    MbValue::from_ptr(MbObject::new_list(vec![empty_list(), MbValue::none()]))
}

pub fn register() {
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
    attrs.insert("MAXINT".into(),  MbValue::from_int(2_147_483_647));
    attrs.insert("MININT".into(),  MbValue::from_int(-2_147_483_648));

    let shell = dispatch_class_shell as *const () as usize;
    let dumps = dispatch_dumps as *const () as usize;
    let loads = dispatch_loads as *const () as usize;

    let class_shells: &[&str] = &[
        "ServerProxy", "Server", "Transport", "SafeTransport",
        "MultiCall", "MultiCallIterator", "Marshaller", "Unmarshaller",
        "ResponseError", "Fault", "ProtocolError",
        "Binary", "Boolean", "DateTime", "Error",
        "_Method", "GZipDecodedResponse", "GzipDecodedResponse",
        "ExpatParser", "_NullMethod",
    ];
    for name in class_shells {
        attrs.insert((*name).into(), MbValue::from_func(shell));
    }
    let dispatchers: &[(&str, usize)] = &[
        ("dumps",                       dumps),
        ("loads",                       loads),
        ("getparser",                   shell),
        ("escape",                      dispatch_class_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(shell as u64);
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
    let shell = dispatch_class_shell as *const () as usize;
    let class_shells: &[&str] = &[
        "SimpleXMLRPCRequestHandler", "SimpleXMLRPCDispatcher",
        "SimpleXMLRPCServer", "MultiPathXMLRPCServer",
        "DocXMLRPCRequestHandler", "DocXMLRPCServer",
        "ServerHTMLDoc", "XMLRPCDocGenerator",
        "CGIXMLRPCRequestHandler",
        "list_public_methods",
        "resolve_dotted_attribute",
    ];
    for name in class_shells {
        attrs.insert((*name).into(), MbValue::from_func(shell));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(shell as u64);
    });
    super::register_module("xmlrpc.server", attrs);
}
