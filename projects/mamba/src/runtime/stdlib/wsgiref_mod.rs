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
        ("FileWrapper", dispatch_class_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.util", attrs);
}

fn register_wsgiref_headers() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[("Headers", dispatch_class_shell as *const () as usize)];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    register_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("wsgiref.headers", attrs);
}

fn register_wsgiref_simple_server() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("WSGIServer", dispatch_class_shell as *const () as usize),
        (
            "WSGIRequestHandler",
            dispatch_class_shell as *const () as usize,
        ),
        ("ServerHandler", dispatch_class_shell as *const () as usize),
        ("make_server", dispatch_class_shell as *const () as usize),
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
        ("validator", dispatch_class_shell as *const () as usize),
        ("WSGIWarning", dispatch_class_shell as *const () as usize),
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
        ("SimpleHandler", dispatch_class_shell as *const () as usize),
        ("BaseCGIHandler", dispatch_class_shell as *const () as usize),
        ("CGIHandler", dispatch_class_shell as *const () as usize),
        ("IISCGIHandler", dispatch_class_shell as *const () as usize),
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
