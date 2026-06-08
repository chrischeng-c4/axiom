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
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
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

pub fn register() {
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
        for a in addrs { set.insert(*a as u64); }
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
        ("setup_testing_defaults",  dispatch_noop        as *const () as usize),
        ("guess_scheme",            dispatch_empty_str   as *const () as usize),
        ("application_uri",         dispatch_empty_str   as *const () as usize),
        ("request_uri",             dispatch_request_uri as *const () as usize),
        ("shift_path_info",         dispatch_empty_str   as *const () as usize),
        ("is_hop_by_hop",           dispatch_noop        as *const () as usize),
        ("FileWrapper",             dispatch_class_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.util", attrs);
}

fn register_wsgiref_headers() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("Headers", dispatch_class_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.headers", attrs);
}

fn register_wsgiref_simple_server() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("WSGIServer",         dispatch_class_shell as *const () as usize),
        ("WSGIRequestHandler", dispatch_class_shell as *const () as usize),
        ("ServerHandler",      dispatch_class_shell as *const () as usize),
        ("make_server",        dispatch_class_shell as *const () as usize),
        ("demo_app",           dispatch_empty_list  as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
        // surface: missing CPython module constants (auto-added)
    attrs.insert("server_version".into(), MbValue::from_ptr(MbObject::new_str("WSGIServer/0.2".to_string())));
    attrs.insert("software_version".into(), MbValue::from_ptr(MbObject::new_str("WSGIServer/0.2 CPython/3.12.11".to_string())));
    attrs.insert("sys_version".into(), MbValue::from_ptr(MbObject::new_str("CPython/3.12.11".to_string())));
    super::register_module("wsgiref.simple_server", attrs);
}

fn register_wsgiref_validate() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("validator",    dispatch_class_shell as *const () as usize),
        ("WSGIWarning",  dispatch_class_shell as *const () as usize),
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
        ("BaseHandler",      dispatch_class_shell as *const () as usize),
        ("SimpleHandler",    dispatch_class_shell as *const () as usize),
        ("BaseCGIHandler",   dispatch_class_shell as *const () as usize),
        ("CGIHandler",       dispatch_class_shell as *const () as usize),
        ("IISCGIHandler",    dispatch_class_shell as *const () as usize),
        ("read_environ",     dispatch_empty_dict  as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.handlers", attrs);
}
