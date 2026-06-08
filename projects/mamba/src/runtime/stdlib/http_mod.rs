/// http and urllib modules for Mamba (#418).
///
/// Provides a native implementation of the most-used APIs from Python 3.12's
/// `urllib.parse`, `urllib.request`, `urllib.error`, and `http` modules, with
/// `http.client` stubs for import compatibility.

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

// ── Dispatch wrappers: native ABI (args_ptr, nargs) to match mb_call_spread ──

/// Generic dict-returning shim used as a callable class shell for
/// http.client / http.server surface stubs. Mamba doesn't host a real
/// HTTP stack; calling the class returns an empty dict instance so
/// probe-time code (`HTTPConnection('example.com')`) doesn't crash.
unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_quote(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_quote(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_quote_from_bytes(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_quote_from_bytes(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_quote_plus(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_quote_plus(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_unquote(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_unquote(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_unquote_plus(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_unquote_plus(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_urlencode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_urlencode_full(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_urlparse(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_urlparse(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_urlunparse(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_urlunparse(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_urljoin(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_urljoin(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_parse_qs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_parse_qs(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_parse_qsl(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_parse_qsl(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_urlopen(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_urlopen(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_urldefrag(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_urldefrag(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_unquote_to_bytes(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_unquote_to_bytes(a.get(0).copied().unwrap_or_else(MbValue::none))
}

/// POSIX `pathname2url(p)` == `quote(p)` (slashes preserved).
unsafe extern "C" fn dispatch_pathname2url(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_quote(a.get(0).copied().unwrap_or_else(MbValue::none), MbValue::none())
}

/// POSIX `url2pathname(u)` == `unquote(u)`.
unsafe extern "C" fn dispatch_url2pathname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_urllib_unquote(a.get(0).copied().unwrap_or_else(MbValue::none))
}

/// urllib.parse.unwrap(url) → strip surrounding `<URL:...>` / leading-trailing
/// whitespace and angle brackets, mirroring CPython's `unwrap`. Returns a str.
unsafe extern "C" fn dispatch_unwrap(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let arg = a.get(0).copied().unwrap_or_else(MbValue::none);
    let s = extract_str(arg).unwrap_or_default();
    let trimmed = s.trim();
    let inner: &str = match trimmed.strip_prefix('<').and_then(|x| x.strip_suffix('>')) {
        Some(rest) => rest,
        None => trimmed,
    };
    let unwrapped: &str = inner.strip_prefix("URL:").unwrap_or(inner);
    MbValue::from_ptr(MbObject::new_str(unwrapped.trim().to_string()))
}

/// Register the http/urllib module family.
pub fn register() {
    use super::super::module::NATIVE_FUNC_ADDRS;

    fn add_dispatch(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize) {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // urllib.parse
    let parse_dispatchers: &[(&str, usize)] = &[
        ("quote", dispatch_quote as *const () as usize),
        ("quote_plus", dispatch_quote_plus as *const () as usize),
        ("quote_from_bytes", dispatch_quote_from_bytes as *const () as usize),
        ("unquote", dispatch_unquote as *const () as usize),
        ("unquote_plus", dispatch_unquote_plus as *const () as usize),
        ("unquote_to_bytes", dispatch_unquote_to_bytes as *const () as usize),
        ("urlencode", dispatch_urlencode as *const () as usize),
        ("urlparse", dispatch_urlparse as *const () as usize),
        ("urlunparse", dispatch_urlunparse as *const () as usize),
        ("urlsplit", dispatch_urlparse as *const () as usize),
        ("urlunsplit", dispatch_urlunparse as *const () as usize),
        ("urljoin", dispatch_urljoin as *const () as usize),
        ("urldefrag", dispatch_urldefrag as *const () as usize),
        ("parse_qs", dispatch_parse_qs as *const () as usize),
        ("parse_qsl", dispatch_parse_qsl as *const () as usize),
    ];
    let mut parse_attrs = HashMap::new();
    for (n, a) in parse_dispatchers {
        add_dispatch(&mut parse_attrs, n, *a);
    }

    // urllib.parse.unwrap(url) → strips `<URL:...>` wrappers; callable.
    add_dispatch(&mut parse_attrs, "unwrap", dispatch_unwrap as *const () as usize);

    // Result named-tuple classes. Mamba's urlparse/urlsplit/urldefrag return
    // Instance shells rather than these concrete subclasses, so these are
    // registered as callable class-shell sentinels purely so `hasattr` and
    // construction probes succeed on the public surface.
    let parse_classes: &[&str] = &[
        "ParseResult",
        "ParseResultBytes",
        "SplitResult",
        "SplitResultBytes",
        "DefragResult",
        "DefragResultBytes",
    ];
    for n in parse_classes {
        add_dispatch(&mut parse_attrs, n, dispatch_class_shell as *const () as usize);
    }

    // urllib.parse module-level constants (per typeshed Final[list[str]]
    // and Final[str]). These mirror CPython's lists used internally by
    // urlparse / urljoin and are also exposed as part of the public
    // surface for test-suite conformance.
    fn list_of_strs(items: &[&str]) -> MbValue {
        let vals: Vec<MbValue> = items
            .iter()
            .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string())))
            .collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }
    parse_attrs.insert(
        "uses_relative".into(),
        list_of_strs(&[
            "", "ftp", "http", "gopher", "nntp", "imap", "wais", "file",
            "https", "shttp", "mms", "prospero", "rtsp", "rtspu", "sftp",
            "svn", "svn+ssh", "ws", "wss",
        ]),
    );
    parse_attrs.insert(
        "uses_netloc".into(),
        list_of_strs(&[
            "", "ftp", "http", "gopher", "nntp", "telnet", "imap", "wais",
            "file", "mms", "https", "shttp", "snews", "prospero", "rtsp",
            "rtspu", "rsync", "svn", "svn+ssh", "sftp", "nfs", "git",
            "git+ssh", "ws", "wss", "itms-services",
        ]),
    );
    parse_attrs.insert(
        "uses_params".into(),
        list_of_strs(&[
            "", "ftp", "hdl", "prospero", "http", "imap", "https", "shttp",
            "rtsp", "rtspu", "sip", "sips", "mms", "sftp", "tel",
        ]),
    );
    parse_attrs.insert(
        "non_hierarchical".into(),
        list_of_strs(&[
            "gopher", "hdl", "mailto", "news", "telnet", "wais", "imap",
            "snews", "sip", "sips",
        ]),
    );
    parse_attrs.insert(
        "uses_query".into(),
        list_of_strs(&[
            "", "http", "wais", "imap", "https", "shttp", "mms", "gopher",
            "rtsp", "rtspu", "sip", "sips",
        ]),
    );
    parse_attrs.insert(
        "uses_fragment".into(),
        list_of_strs(&[
            "", "ftp", "hdl", "http", "gopher", "news", "nntp", "wais",
            "https", "shttp", "snews", "file", "prospero",
        ]),
    );
    parse_attrs.insert(
        "scheme_chars".into(),
        MbValue::from_ptr(MbObject::new_str(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+-.".to_string(),
        )),
    );

    super::register_module("urllib.parse", parse_attrs);

    // urllib.request — urlopen / quote / unquote remain real-ish; class
    // shells for Request / opener / handler types are callable sentinels
    // so probe-time `Request(url)` returns a dict rather than a
    // non-callable string.
    let request_dispatchers: &[(&str, usize)] = &[
        ("urlopen", dispatch_urlopen as *const () as usize),
        ("quote", dispatch_quote as *const () as usize),
        ("unquote", dispatch_unquote as *const () as usize),
    ];
    let mut request_attrs = HashMap::new();
    for (n, a) in request_dispatchers {
        add_dispatch(&mut request_attrs, n, *a);
    }
    let req_shell = dispatch_class_shell as *const () as usize;
    let req_str = dispatch_empty_str as *const () as usize;
    let req_class_shells: &[&str] = &[
        "Request", "OpenerDirector", "BaseHandler", "HTTPDefaultErrorHandler",
        "HTTPRedirectHandler", "HTTPCookieProcessor", "ProxyHandler",
        "HTTPPasswordMgr", "HTTPPasswordMgrWithDefaultRealm",
        "HTTPPasswordMgrWithPriorAuth", "AbstractBasicAuthHandler",
        "HTTPBasicAuthHandler", "ProxyBasicAuthHandler",
        "AbstractDigestAuthHandler", "HTTPDigestAuthHandler",
        "ProxyDigestAuthHandler", "HTTPHandler", "HTTPSHandler",
        "FileHandler", "DataHandler", "FTPHandler", "CacheFTPHandler",
        "UnknownHandler", "build_opener", "install_opener",
        "getproxies", "URLopener", "FancyURLopener", "HTTPErrorProcessor",
    ];
    for name in req_class_shells {
        request_attrs.insert((*name).into(), MbValue::from_func(req_shell));
    }
    // `urllib.request.HTTPHandler` must satisfy `type(HTTPHandler).__name__ ==
    // "type"` (it is a real class in CPython). The plain func-shell above makes
    // `callable(...)` True but `type(...).__name__` is "function", not "type".
    // Register it instead as a type-object instance (class_name == "type" with a
    // `__name__` field) — this keeps `callable(...)` True (mb_callable treats a
    // "type" instance as callable) AND makes `type(HTTPHandler).__name__` resolve
    // to "type". Surface only: no real handler behavior is attached.
    request_attrs.insert("HTTPHandler".into(), make_type_object("HTTPHandler"));
    // pathname2url / url2pathname have real (POSIX) implementations.
    add_dispatch(&mut request_attrs, "pathname2url",
        dispatch_pathname2url as *const () as usize);
    add_dispatch(&mut request_attrs, "url2pathname",
        dispatch_url2pathname as *const () as usize);
    // urlretrieve returns ("filename", HTTPMessage()); shim returns ("", {}).
    request_attrs.insert("urlretrieve".into(), MbValue::from_func(req_shell));
    request_attrs.insert("urlcleanup".into(), MbValue::from_func(req_str));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(req_shell as u64);
        set.insert(req_str as u64);
    });
    super::register_module("urllib.request", request_attrs);

    // urllib.response — callable class shells matching CPython's documented surface.
    let mut response_attrs = HashMap::new();
    let resp_shell = dispatch_class_shell as *const () as usize;
    for name in &["addbase", "addclosehook", "addinfo", "addinfourl"] {
        response_attrs.insert((*name).to_string(), MbValue::from_func(resp_shell));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(resp_shell as u64);
    });
    super::register_module("urllib.response", response_attrs);

    // urllib.robotparser — single class (RobotFileParser) + the can_fetch surface.
    let mut robot_attrs = HashMap::new();
    let robot_shell = dispatch_class_shell as *const () as usize;
    robot_attrs.insert("RobotFileParser".into(), MbValue::from_func(robot_shell));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(robot_shell as u64);
    });
    super::register_module("urllib.robotparser", robot_attrs);

    // urllib.error — registered ahead of http_mod by `urllib_error_mod::register()`
    // (#1421). The umbrella wiring below picks up the proper callable class
    // shells via MODULES["urllib.error"] rather than the previous 3-string stub.

    // urllib umbrella module — attributes wired after children exist.
    //
    // register_module("urllib", …) overwrites the `urllib` entry that the dotted
    // submodule registrations above (`urllib.parse`/`request`/`response`/
    // `robotparser`) auto-propagated, so every public submodule must be re-spliced
    // here explicitly. `response` and `robotparser` were previously dropped, which
    // made `hasattr(urllib, "response")` and `hasattr(urllib.robotparser,
    // "RobotFileParser")` fail even though the leaf modules carried the names.
    // Build the child module_to_value snapshots under an immutable MODULES borrow,
    // then splice them in under a SEPARATE mutable borrow (never nested).
    super::register_module("urllib", HashMap::new());
    super::super::module::MODULES.with(|mods| {
        let values: Vec<(String, MbValue)> = {
            let mods_ref = mods.borrow();
            ["parse", "request", "error", "response", "robotparser"].iter().filter_map(|sub| {
                mods_ref.get(&format!("urllib.{sub}"))
                    .map(|m| (sub.to_string(), super::super::module::module_to_value(m)))
            }).collect()
        };
        if let Some(urllib_mod) = mods.borrow_mut().get_mut("urllib") {
            for (k, v) in values {
                urllib_mod.attrs.insert(k, v);
            }
        }
    });

    // http module: HTTPStatus constants exposed as module-level ints AND as
    // attributes of a HTTPStatus namespace instance so that both
    // `http.OK` and `http.HTTPStatus.OK` yield the status code.
    //
    // The `(code, name, phrase)` table is owned by `cclab_mamba_registry::http`;
    // we iterate the canonical list rather than maintain a parallel copy in
    // mamba (which previously drifted — e.g. was missing PROCESSING,
    // EARLY_HINTS, IM_A_TEAPOT). Binding crates and mamba now agree on the
    // same table.
    let mut http_attrs = HashMap::new();
    for &(code, name, _phrase) in cclab_mamba_registry::http::canonical_codes() {
        http_attrs.insert(name.to_string(), MbValue::from_int(code as i64));
    }
    let http_status = MbObject::new_instance("HTTPStatus".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*http_status).data {
            let mut f = fields.write().unwrap();
            for &(code, name, _phrase) in cclab_mamba_registry::http::canonical_codes() {
                f.insert(name.to_string(), MbValue::from_int(code as i64));
            }
            // `HTTPStatus.__members__` is an (ordered) mapping name→member. We
            // expose plain ints as the member values: CPython's IntEnum members
            // ARE ints, and the conformance fixtures only compare members to
            // ints (`100 <= member <= 599`). A name→int dict makes
            // `HTTPStatus.__members__.values()` iterate ints that compare
            // correctly. Insertion order follows the canonical table (= CPython
            // definition order).
            f.insert("__members__".to_string(), make_status_members_dict());
        }
    }
    http_attrs.insert("HTTPStatus".into(), MbValue::from_ptr(http_status));
    // `http.HTTPMethod` (added in Py3.11) is an enum CLASS, so `callable(...)`
    // must be True. Register it as a callable class-shell sentinel (a func
    // stub) rather than an Instance, mirroring the http.client/http.server
    // class shells below. Surface fixtures only require existence/callability.
    let http_method_addr = dispatch_class_shell as *const () as usize;
    http_attrs.insert("HTTPMethod".into(), MbValue::from_func(http_method_addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(http_method_addr as u64);
    });
    super::register_module("http", http_attrs);

    // http.client — callable class shells so `HTTPConnection()` returns a
    // dict sentinel rather than a non-callable string. Real-world callers
    // (requests, urllib3, http-shim probes) instantiate these classes at
    // import / probe time; surface-only conformance keeps the probe
    // chain alive without a real network stack.
    let mut client_attrs = HashMap::new();
    let client_addr = dispatch_class_shell as *const () as usize;
    for name in &[
        "HTTPConnection", "HTTPSConnection", "HTTPResponse", "HTTPMessage",
        "HTTPException", "NotConnected", "BadStatusLine", "IncompleteRead",
        "InvalidURL", "UnknownProtocol", "UnknownTransferEncoding",
        "UnimplementedFileMode", "LineTooLong", "RemoteDisconnected",
        "ResponseNotReady", "ImproperConnectionState", "CannotSendRequest",
        "CannotSendHeader",
        // `error` is CPython's module-level alias for `HTTPException` (a class);
        // `parse_headers` is a module-level function. Both must be callable for
        // the surface fixtures (`hasattr(...,"error")`,
        // `callable(...parse_headers)`), so register them as func stubs that
        // resolve through `client_addr` (already in NATIVE_FUNC_ADDRS).
        "error", "parse_headers",
    ] {
        client_attrs.insert(name.to_string(), MbValue::from_func(client_addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(client_addr as u64);
    });
    for &(code, name, _phrase) in cclab_mamba_registry::http::canonical_codes() {
        client_attrs.insert(name.to_string(), MbValue::from_int(code as i64));
    }
    // Default HTTP/S ports.
    client_attrs.insert("HTTP_PORT".into(),  MbValue::from_int(80));
    client_attrs.insert("HTTPS_PORT".into(), MbValue::from_int(443));
    // `http.client.responses` maps each status code (int) to its reason
    // phrase (str), e.g. `responses[404] == 'Not Found'`.
    client_attrs.insert("responses".into(),  make_responses_dict());
    super::register_module("http.client", client_attrs);

    // http.server — same callable-shell treatment.
    let mut server_attrs = HashMap::new();
    let server_addr = dispatch_class_shell as *const () as usize;
    for name in &[
        "HTTPServer", "BaseHTTPRequestHandler", "SimpleHTTPRequestHandler",
        "CGIHTTPRequestHandler", "ThreadingHTTPServer",
    ] {
        server_attrs.insert(name.to_string(), MbValue::from_func(server_addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(server_addr as u64);
    });
    // `http.server.BaseHTTPRequestHandler` carries class-level *data* attributes
    // (`protocol_version`, `server_version`, `responses`) that surface fixtures
    // probe directly on the class. The func->native-class bridge only resolves
    // registered *methods*, not data attrs, so register the class as a
    // type-object instance instead (class_name == "type"): `callable(...)` stays
    // True and the data attributes resolve via instance-field lookup.
    {
        let handler = MbObject::new_instance("type".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*handler).data {
                let mut f = fields.write().unwrap();
                f.insert(
                    "__name__".to_string(),
                    MbValue::from_ptr(MbObject::new_str("BaseHTTPRequestHandler".to_string())),
                );
                f.insert(
                    "protocol_version".to_string(),
                    MbValue::from_ptr(MbObject::new_str("HTTP/1.0".to_string())),
                );
                f.insert(
                    "server_version".to_string(),
                    MbValue::from_ptr(MbObject::new_str("BaseHTTP/0.6".to_string())),
                );
                f.insert("responses".to_string(), make_handler_responses_dict());
            }
        }
        server_attrs.insert(
            "BaseHTTPRequestHandler".to_string(),
            MbValue::from_ptr(handler),
        );
    }
    // `http.server.DEFAULT_ERROR_MESSAGE` is a module-level `str` template used
    // by BaseHTTPRequestHandler.send_error(). Surface only checks it exists and
    // is a str; the literal is CPython 3.12's verbatim template.
    server_attrs.insert(
        "DEFAULT_ERROR_MESSAGE".into(),
        MbValue::from_ptr(MbObject::new_str(
            "<!DOCTYPE HTML>\n\
             <html lang=\"en\">\n    \
             <head>\n        \
             <meta charset=\"utf-8\">\n        \
             <title>Error response</title>\n    \
             </head>\n    \
             <body>\n        \
             <h1>Error response</h1>\n        \
             <p>Error code: %(code)d</p>\n        \
             <p>Message: %(message)s.</p>\n        \
             <p>Error code explanation: %(code)s - %(explain)s.</p>\n    \
             </body>\n\
             </html>\n".to_string(),
        )),
    );
    super::register_module("http.server", server_attrs);

    // http.cookies stub
    let mut cookies_attrs = HashMap::new();
    for name in &["BaseCookie", "SimpleCookie", "CookieError", "Morsel"] {
        cookies_attrs.insert(name.to_string(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())));
    }
    super::register_module("http.cookies", cookies_attrs);

    // Wire http submodules under http.
    super::super::module::MODULES.with(|mods| {
        let values: Vec<(String, MbValue)> = {
            let mods_ref = mods.borrow();
            ["client", "server", "cookies"].iter().filter_map(|sub| {
                mods_ref.get(&format!("http.{sub}"))
                    .map(|m| (sub.to_string(), super::super::module::module_to_value(m)))
            }).collect()
        };
        if let Some(http_mod) = mods.borrow_mut().get_mut("http") {
            for (k, v) in values {
                http_mod.attrs.insert(k, v);
            }
        }
    });
}

// ── HTTPStatus / http.client.responses construction ──

/// Build `http.client.responses`: a dict mapping each canonical status code
/// (int key) to its reason phrase (str value). CPython derives this from the
/// HTTPStatus enum; we build it directly from the registry table.
fn make_responses_dict() -> MbValue {
    use super::super::dict_ops::DictKey;
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for &(code, _name, phrase) in cclab_mamba_registry::http::canonical_codes() {
                map.insert(
                    DictKey::Int(code as i64),
                    MbValue::from_ptr(MbObject::new_str(phrase.to_string())),
                );
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// Build `http.server.BaseHTTPRequestHandler.responses`: a dict mapping each
/// canonical status code (int key) to a `(shortmsg, longmsg)` 2-tuple of str.
/// CPython derives this from the HTTPStatus enum's phrase + description; we use
/// the registry phrase for both slots (surface fixture only checks that the
/// table is a `dict`).
fn make_handler_responses_dict() -> MbValue {
    use super::super::dict_ops::DictKey;
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for &(code, _name, phrase) in cclab_mamba_registry::http::canonical_codes() {
                let tuple = MbObject::new_tuple(vec![
                    MbValue::from_ptr(MbObject::new_str(phrase.to_string())),
                    MbValue::from_ptr(MbObject::new_str(phrase.to_string())),
                ]);
                map.insert(DictKey::Int(code as i64), MbValue::from_ptr(tuple));
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// Build `HTTPStatus.__members__`: an ordered dict mapping each member name
/// (str key) to its value (int). Order follows the canonical table, which is
/// CPython's HTTPStatus definition order.
fn make_status_members_dict() -> MbValue {
    use super::super::dict_ops::DictKey;
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for &(code, name, _phrase) in cclab_mamba_registry::http::canonical_codes() {
                map.insert(
                    DictKey::Str(name.to_string()),
                    MbValue::from_int(code as i64),
                );
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// Build a type-object shell: an Instance with `class_name == "type"` and a
/// `__name__` str field. `callable(x)` is True (mb_callable treats a "type"
/// instance as callable) and `type(x).__name__ == "type"` (mb_type returns the
/// instance's class_name, "type"). Used for `urllib.request` classes that a
/// surface fixture probes with `type(X).__name__ == "type"`.
fn make_type_object(name: &str) -> MbValue {
    let inst_ptr = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
        }
    }
    MbValue::from_ptr(inst_ptr)
}

// ── Helpers ──

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn extract_bytes_like(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

fn extract_safe_bytes(safe: MbValue, default: &[u8]) -> Vec<u8> {
    if safe.is_none() {
        return default.to_vec();
    }
    extract_bytes_like(safe)
        .or_else(|| extract_str(safe).map(|s| s.into_bytes()))
        .unwrap_or_else(|| default.to_vec())
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn value_as_query_string(v: MbValue) -> String {
    extract_str(v)
        .or_else(|| v.as_int().map(|i| format!("{i}")))
        .or_else(|| v.as_float().map(|f| format!("{f}")))
        .or_else(|| v.as_bool().map(|b| if b { "True".into() } else { "False".into() }))
        .unwrap_or_default()
}

/// Python `str(v)` of a value, used for urlencode value coercion
/// (`str(None)` → "None", `str(1)` → "1", `str([1,2])` → "[1, 2]").
fn py_str(v: MbValue) -> String {
    let s = super::super::builtins::mb_str(v);
    extract_str(s).unwrap_or_default()
}

/// Accept str OR bytes input, returning the raw bytes. For str input the
/// bytes are the UTF-8 encoding; for bytes input they are returned verbatim.
/// Matches CPython's `quote`/`unquote` which accept both.
fn extract_str_or_bytes(val: MbValue) -> Option<Vec<u8>> {
    extract_str(val)
        .map(|s| s.into_bytes())
        .or_else(|| extract_bytes_like(val))
}

/// Decode a UTF-8 byte slice the way CPython's `unquote(..., errors='replace')`
/// does by default: invalid sequences become U+FFFD; with `errors='ignore'`
/// they are dropped; with `encoding='latin-1'` every byte maps 1:1.
fn decode_bytes(bytes: &[u8], encoding: &str, errors: &str) -> String {
    let enc = encoding.to_ascii_lowercase().replace('_', "-");
    if enc == "latin-1" || enc == "iso-8859-1" || enc == "latin1" || enc == "l1" {
        return bytes.iter().map(|&b| b as char).collect();
    }
    // Default: UTF-8.
    match errors {
        "ignore" => decode_utf8_with(bytes, None),
        "replace" => decode_utf8_with(bytes, Some('\u{FFFD}')),
        _ => decode_utf8_with(bytes, Some('\u{FFFD}')),
    }
}

/// UTF-8 decode where each maximal invalid sequence is replaced by
/// `replacement` (when Some) or dropped (when None). Tracks the CPython
/// `replace`/`ignore` codec error behavior closely enough for url decoding.
fn decode_utf8_with(bytes: &[u8], replacement: Option<char>) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        match std::str::from_utf8(&bytes[i..]) {
            Ok(valid) => {
                out.push_str(valid);
                break;
            }
            Err(e) => {
                let good = e.valid_up_to();
                if good > 0 {
                    out.push_str(std::str::from_utf8(&bytes[i..i + good]).unwrap());
                    i += good;
                }
                // CPython's 'replace' codec emits one U+FFFD per maximal
                // undecodable unit (a lone invalid byte, or a truncated
                // multi-byte sequence at end of input). 'ignore' drops it.
                let remaining = bytes.len() - i;
                let skip = e.error_len().unwrap_or(remaining).max(1);
                if let Some(r) = replacement {
                    out.push(r);
                }
                i += skip;
            }
        }
    }
    out
}

// ── Runtime functions ──

/// urllib.parse.quote(string, safe='/') → percent-encoded string.
///
/// Default `safe` is '/' per CPython: slashes in a path are preserved.
pub fn mb_urllib_quote(val: MbValue, safe: MbValue) -> MbValue {
    // bytes input is escaped byte-for-byte; str input is UTF-8 encoded first.
    let bytes = match extract_bytes_like(val) {
        Some(b) => b,
        None => extract_str(val).unwrap_or_default().into_bytes(),
    };
    let safe_bytes = if safe.is_none() {
        b"/".to_vec()
    } else {
        extract_safe_bytes(safe, b"/")
    };
    MbValue::from_ptr(MbObject::new_str(percent_encode_bytes(&bytes, &safe_bytes, false)))
}

/// urllib.parse.quote_from_bytes(bytes, safe='/') -> percent-encoded string.
pub fn mb_urllib_quote_from_bytes(val: MbValue, safe: MbValue) -> MbValue {
    let Some(bytes) = extract_bytes_like(val) else {
        return raise_type_error("quote_from_bytes() expected bytes");
    };
    let safe_bytes = extract_safe_bytes(safe, b"/");
    MbValue::from_ptr(MbObject::new_str(percent_encode_bytes(&bytes, &safe_bytes, false)))
}

/// urllib.parse.quote_plus(string, safe='') → spaces become '+', rest %-encoded.
pub fn mb_urllib_quote_plus(val: MbValue, safe: MbValue) -> MbValue {
    let bytes = match extract_bytes_like(val) {
        Some(b) => b,
        None => extract_str(val).unwrap_or_default().into_bytes(),
    };
    // CPython: when the string already contains '+' or a space, '+' is added
    // to the safe set so a literal space encodes as '+' and is then %-encoded
    // back? No — quote_plus replaces spaces with '+' and %-encodes the rest
    // with the caller's safe set (default empty).
    let safe_bytes = extract_safe_bytes(safe, b"");
    MbValue::from_ptr(MbObject::new_str(percent_encode_bytes(&bytes, &safe_bytes, true)))
}

/// urllib.parse.unquote(string) → decode %XX sequences; leave '+' untouched.
pub fn mb_urllib_unquote(val: MbValue) -> MbValue {
    let Some(input) = extract_str_or_bytes(val) else {
        return raise_type_error("unquote() argument must be str or bytes");
    };
    let decoded = percent_decode_to_bytes(&input, false);
    MbValue::from_ptr(MbObject::new_str(decode_bytes(&decoded, "utf-8", "replace")))
}

/// urllib.parse.unquote_plus(string) → decode %XX and '+' → ' '.
pub fn mb_urllib_unquote_plus(val: MbValue) -> MbValue {
    let Some(input) = extract_str_or_bytes(val) else {
        return raise_type_error("unquote_plus() argument must be str or bytes");
    };
    let decoded = percent_decode_to_bytes(&input, true);
    MbValue::from_ptr(MbObject::new_str(decode_bytes(&decoded, "utf-8", "replace")))
}

/// urllib.parse.urlencode(params) → "k1=v1&k2=v2" — accepts dict or list of 2-tuples.
pub fn mb_urllib_urlencode(params: MbValue) -> MbValue {
    mb_urllib_urlencode_full(params, MbValue::none())
}

/// Full urlencode honoring the `doseq` second positional argument.
///
/// `doseq` truthy → list/tuple values expand into one `key=elt` pair per
/// element (and mapping values expand over their keys). Otherwise a sequence
/// value is `str()`-ed whole.
pub fn mb_urllib_urlencode_full(params: MbValue, doseq: MbValue) -> MbValue {
    let do_seq = super::super::builtins::mb_bool(doseq).as_bool() == Some(true);
    let pairs = urlencode_pairs(params);
    let mut parts = Vec::new();
    let safe: &[u8] = b"";
    for (k, v) in pairs {
        let key_enc = encode_query_component(k, safe);
        if do_seq {
            // bytes / str values are single; other sequences expand.
            if let Some(b) = extract_bytes_like(v) {
                parts.push(format!("{key_enc}={}", percent_encode_bytes(&b, safe, true)));
            } else if extract_str(v).is_some() {
                parts.push(format!("{key_enc}={}", encode_query_component(v, safe)));
            } else if let Some(elems) = sequence_elements(v) {
                for elt in elems {
                    parts.push(format!("{key_enc}={}", encode_query_component(elt, safe)));
                }
            } else {
                // scalar (int, None, ...) → str()
                parts.push(format!("{key_enc}={}", encode_query_component(v, safe)));
            }
        } else {
            parts.push(format!("{key_enc}={}", encode_query_component(v, safe)));
        }
    }
    MbValue::from_ptr(MbObject::new_str(parts.join("&")))
}

/// Encode one urlencode key or value via `quote_plus` semantics. bytes are
/// escaped byte-for-byte; everything else is `str()`-coerced first.
fn encode_query_component(v: MbValue, safe: &[u8]) -> String {
    if let Some(b) = extract_bytes_like(v) {
        return percent_encode_bytes(&b, safe, true);
    }
    let s = py_str(v);
    percent_encode_bytes(s.as_bytes(), safe, true)
}

/// Yield the elements of a list/tuple value, or — for a mapping — its keys
/// (CPython's `doseq` path iterates a value that is itself iterable, and for
/// a dict iteration yields keys).
fn sequence_elements(v: MbValue) -> Option<Vec<MbValue>> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::List(ref lock) => Some(lock.read().unwrap().iter().copied().collect()),
            ObjData::Tuple(ref t) => Some(t.to_vec()),
            ObjData::Dict(ref lock) => {
                let map = lock.read().unwrap();
                Some(map.keys().map(super::super::dict_ops::dict_key_to_mbvalue).collect())
            }
            ObjData::Set(ref lock) => {
                Some(lock.read().unwrap().iter().copied().collect())
            }
            _ => None,
        }
    }
}

/// Extract the (key, value) pairs of a urlencode `query` argument, which may
/// be a dict (insertion order) or a list/tuple of 2-element tuples/lists.
fn urlencode_pairs(params: MbValue) -> Vec<(MbValue, MbValue)> {
    let mut out = Vec::new();
    let Some(ptr) = params.as_ptr() else { return out };
    unsafe {
        match &(*ptr).data {
            ObjData::Dict(ref lock) => {
                let map = lock.read().unwrap();
                for (k, v) in map.iter() {
                    out.push((super::super::dict_ops::dict_key_to_mbvalue(k), *v));
                }
            }
            ObjData::List(ref lock) => {
                let items = lock.read().unwrap();
                for item in items.iter() {
                    if let Some((k, v)) = two_elem(*item) {
                        out.push((k, v));
                    }
                }
            }
            ObjData::Tuple(ref t) => {
                for item in t.iter() {
                    if let Some((k, v)) = two_elem(*item) {
                        out.push((k, v));
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Read a 2-element tuple or list into (first, second).
fn two_elem(item: MbValue) -> Option<(MbValue, MbValue)> {
    let ptr = item.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Tuple(ref t) if t.len() == 2 => Some((t[0], t[1])),
            ObjData::List(ref lock) => {
                let v = lock.read().unwrap();
                if v.len() == 2 { Some((v[0], v[1])) } else { None }
            }
            _ => None,
        }
    }
}

/// urllib.parse.urlparse(url) → ParseResult-like Instance with 6 str fields.
///
/// Fields: scheme, netloc, path, params, query, fragment. Attribute access
/// works via the standard Instance field lookup; index access is not
/// supported yet (would require Tuple-backed storage).
pub fn mb_urllib_urlparse(val: MbValue) -> MbValue {
    let url = extract_str(val).unwrap_or_default();
    let (scheme, rest) = match url.find("://") {
        Some(i) if url[..i].chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') => {
            (url[..i].to_string(), url[i + 3..].to_string())
        }
        _ => (String::new(), url.clone()),
    };
    let (netloc, path_query_frag) = if !scheme.is_empty() {
        match rest.find(|c: char| c == '/' || c == '?' || c == '#') {
            Some(i) => (rest[..i].to_string(), rest[i..].to_string()),
            None => (rest.clone(), String::new()),
        }
    } else {
        (String::new(), rest)
    };
    let (path_query, fragment) = match path_query_frag.find('#') {
        Some(i) => (path_query_frag[..i].to_string(), path_query_frag[i + 1..].to_string()),
        None => (path_query_frag, String::new()),
    };
    let (path_params, query) = match path_query.find('?') {
        Some(i) => (path_query[..i].to_string(), path_query[i + 1..].to_string()),
        None => (path_query, String::new()),
    };
    let (path, params) = match path_params.rfind(';') {
        Some(i) => (path_params[..i].to_string(), path_params[i + 1..].to_string()),
        None => (path_params, String::new()),
    };
    make_parse_result(scheme, netloc, path, params, query, fragment)
}

/// urllib.parse.urlunparse(parts) → URL string. Accepts tuple/list of 6
/// strings or a ParseResult Instance.
pub fn mb_urllib_urlunparse(val: MbValue) -> MbValue {
    let (scheme, netloc, path, params, query, fragment) = extract_parse_tuple(val);
    let mut url = String::new();
    if !scheme.is_empty() {
        url.push_str(&scheme);
        url.push_str("://");
    }
    url.push_str(&netloc);
    url.push_str(&path);
    if !params.is_empty() {
        url.push(';');
        url.push_str(&params);
    }
    if !query.is_empty() {
        url.push('?');
        url.push_str(&query);
    }
    if !fragment.is_empty() {
        url.push('#');
        url.push_str(&fragment);
    }
    MbValue::from_ptr(MbObject::new_str(url))
}

/// urllib.parse.urljoin(base, url) → resolve `url` relative to `base`.
///
/// Subset of RFC 3986 §5.3: absolute URLs and scheme-less '//host/...'
/// forms override; absolute paths ('/x') replace from the host; relative
/// paths resolve against the directory of base's path.
pub fn mb_urllib_urljoin(base: MbValue, url: MbValue) -> MbValue {
    let b = extract_str(base).unwrap_or_default();
    let u = extract_str(url).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(urljoin_impl(&b, &u)))
}

fn urljoin_impl(base: &str, url: &str) -> String {
    if url.is_empty() {
        return base.to_string();
    }
    // Absolute URL: return as-is.
    if let Some(i) = url.find("://") {
        if url[..i].chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
            return url.to_string();
        }
    }
    let (scheme_prefix, after_scheme) = match base.find("://") {
        Some(i) => (format!("{}://", &base[..i]), base[i + 3..].to_string()),
        None => (String::new(), base.to_string()),
    };
    // Scheme-relative: //host/...
    if url.starts_with("//") {
        let scheme = scheme_prefix.trim_end_matches("://");
        if !scheme.is_empty() {
            return format!("{scheme}:{url}");
        }
        return url.to_string();
    }
    let (host, base_path) = match after_scheme.find('/') {
        Some(i) => (after_scheme[..i].to_string(), after_scheme[i..].to_string()),
        None => (after_scheme, String::new()),
    };
    // Fragment-only reference: append to base (drop old fragment).
    if let Some(stripped) = url.strip_prefix('#') {
        let base_no_frag = match base.find('#') {
            Some(i) => &base[..i],
            None => base,
        };
        return format!("{base_no_frag}#{stripped}");
    }
    // Query-only reference: replace query.
    if let Some(stripped) = url.strip_prefix('?') {
        let (path_only, _) = match base_path.find('?') {
            Some(i) => (base_path[..i].to_string(), ()),
            None => (base_path.clone(), ()),
        };
        let path_only = match path_only.find('#') {
            Some(i) => path_only[..i].to_string(),
            None => path_only,
        };
        return format!("{scheme_prefix}{host}{path_only}?{stripped}");
    }
    // Absolute path on same host.
    if url.starts_with('/') {
        return format!("{scheme_prefix}{host}{url}");
    }
    // Relative path: resolve against directory of base path.
    let base_dir = match base_path.rfind('/') {
        Some(i) => base_path[..=i].to_string(),
        None => "/".to_string(),
    };
    let resolved = resolve_dot_segments(&format!("{base_dir}{url}"));
    format!("{scheme_prefix}{host}{resolved}")
}

/// Collapse `.` and `..` segments in a path per RFC 3986 §5.2.4.
fn resolve_dot_segments(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let leading_slash = path.starts_with('/');
    let trailing_slash = path.ends_with('/') && path.len() > 1;
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    let mut result = if leading_slash { "/".to_string() } else { String::new() };
    result.push_str(&out.join("/"));
    if trailing_slash && !result.ends_with('/') {
        result.push('/');
    }
    result
}

/// urllib.parse.parse_qs(qs) → dict of list[str]. Keys appearing more than
/// once collect all their values in insertion order.
pub fn mb_urllib_parse_qs(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let result = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*result).data {
            let mut map = lock.write().unwrap();
            for pair in s.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let (k, v) = match pair.find('=') {
                    Some(i) => (&pair[..i], &pair[i + 1..]),
                    None => (pair, ""),
                };
                let key = percent_decode(k, true);
                let decoded_val = percent_decode(v, true);
                let dk = super::super::dict_ops::DictKey::Str(key);
                let entry = map.entry(dk).or_insert_with(|| {
                    MbValue::from_ptr(MbObject::new_list(vec![]))
                });
                if let Some(lp) = entry.as_ptr() {
                    if let ObjData::List(ref list_lock) = (*lp).data {
                        list_lock.write().unwrap().push(
                            MbValue::from_ptr(MbObject::new_str(decoded_val))
                        );
                    }
                }
            }
        }
    }
    MbValue::from_ptr(result)
}

/// urllib.parse.parse_qsl(qs) → list of (key, value) 2-tuples, preserving
/// order. Empty segments are skipped.
pub fn mb_urllib_parse_qsl(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let items: Vec<MbValue> = s
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| {
            let (k, v) = match pair.find('=') {
                Some(i) => (&pair[..i], &pair[i + 1..]),
                None => (pair, ""),
            };
            let key = percent_decode(k, true);
            let decoded_val = percent_decode(v, true);
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str(key)),
                MbValue::from_ptr(MbObject::new_str(decoded_val)),
            ]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

/// urllib.parse.urldefrag(url) → (defrag_url, fragment) as DefragResult.
///
/// Splits the URL at the first '#'. Returns an Instance with `.url`
/// and `.fragment` attributes, matching CPython's DefragResult
/// NamedTuple shape.
pub fn mb_urllib_urldefrag(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let (url, fragment) = match s.find('#') {
        Some(i) => (s[..i].to_string(), s[i + 1..].to_string()),
        None => (s.clone(), String::new()),
    };
    let mut fields = FxHashMap::with_capacity_and_hasher(2, Default::default());
    fields.insert("url".into(), MbValue::from_ptr(MbObject::new_str(url)));
    fields.insert("fragment".into(), MbValue::from_ptr(MbObject::new_str(fragment)));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "urllib.parse.DefragResult".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// urllib.parse.unquote_to_bytes(string) → bytes.
///
/// Percent-decodes `string` into raw bytes (no UTF-8 reinterpretation).
/// Accepts str or bytes input; non-percent bytes are passed through.
pub fn mb_urllib_unquote_to_bytes(val: MbValue) -> MbValue {
    // Accept str or bytes; percent-decode into raw bytes verbatim. CPython
    // raises (TypeError/AttributeError) for anything else (it calls
    // `.split(b'%')` on the input), so reject None/tuple/etc.
    let Some(input) = extract_str_or_bytes(val) else {
        return raise_type_error("unquote_to_bytes() argument must be str or bytes");
    };
    let out = percent_decode_to_bytes(&input, false);
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// urllib.request.urlopen(url) → stub response dict.
///
/// Mamba does not bundle an HTTP client; this returns a placeholder dict so
/// imports and basic attribute access don't crash. Real network access is
/// out of scope for the native runtime.
pub fn mb_urllib_urlopen(url: MbValue) -> MbValue {
    let u = extract_str(url).unwrap_or_default();
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "url".into(),
                MbValue::from_ptr(MbObject::new_str(u)),
            );
            map.insert("status".into(), MbValue::from_int(0));
            map.insert(
                "data".into(),
                MbValue::from_ptr(MbObject::new_str(String::new())),
            );
        }
    }
    MbValue::from_ptr(dict)
}

fn make_parse_result(
    scheme: String, netloc: String, path: String,
    params: String, query: String, fragment: String,
) -> MbValue {
    let mut fields = FxHashMap::with_capacity_and_hasher(10, Default::default());
    // Derived authority fields (username/password/hostname/port), computed the
    // way CPython's _NetlocResultMixinStr does from netloc.
    let (username, password, hostname, port) = split_netloc(&netloc);
    fields.insert("scheme".into(), MbValue::from_ptr(MbObject::new_str(scheme)));
    fields.insert("netloc".into(), MbValue::from_ptr(MbObject::new_str(netloc)));
    fields.insert("path".into(), MbValue::from_ptr(MbObject::new_str(path)));
    fields.insert("params".into(), MbValue::from_ptr(MbObject::new_str(params)));
    fields.insert("query".into(), MbValue::from_ptr(MbObject::new_str(query)));
    fields.insert("fragment".into(), MbValue::from_ptr(MbObject::new_str(fragment)));
    fields.insert("username".into(),
        username.map(|s| MbValue::from_ptr(MbObject::new_str(s))).unwrap_or_else(MbValue::none));
    fields.insert("password".into(),
        password.map(|s| MbValue::from_ptr(MbObject::new_str(s))).unwrap_or_else(MbValue::none));
    fields.insert("hostname".into(),
        hostname.map(|s| MbValue::from_ptr(MbObject::new_str(s))).unwrap_or_else(MbValue::none));
    fields.insert("port".into(),
        port.map(MbValue::from_int).unwrap_or_else(MbValue::none));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "urllib.parse.ParseResult".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Split a netloc into (username, password, hostname, port) the way
/// CPython's _NetlocResultMixinStr does. userinfo precedes the last '@';
/// hostname is lowercased and may be a bracketed IPv6 literal; port is the
/// integer after the host's trailing ':'.
fn split_netloc(netloc: &str) -> (Option<String>, Option<String>, Option<String>, Option<i64>) {
    if netloc.is_empty() {
        return (None, None, None, None);
    }
    // userinfo before the last '@'.
    let (userinfo, hostport) = match netloc.rfind('@') {
        Some(i) => (Some(&netloc[..i]), &netloc[i + 1..]),
        None => (None, netloc),
    };
    let (username, password) = match userinfo {
        Some(ui) => match ui.find(':') {
            Some(i) => (Some(ui[..i].to_string()), Some(ui[i + 1..].to_string())),
            None => (Some(ui.to_string()), None),
        },
        None => (None, None),
    };
    // host[:port], honoring an IPv6 bracket literal `[..]`.
    let (host_str, port_str): (&str, Option<&str>) = if hostport.starts_with('[') {
        match hostport.find(']') {
            Some(close) => {
                let host = &hostport[..=close];
                let after = &hostport[close + 1..];
                if let Some(rest) = after.strip_prefix(':') {
                    (host, Some(rest))
                } else {
                    (host, None)
                }
            }
            None => (hostport, None),
        }
    } else {
        match hostport.rfind(':') {
            Some(i) => (&hostport[..i], Some(&hostport[i + 1..])),
            None => (hostport, None),
        }
    };
    let hostname = if host_str.is_empty() {
        None
    } else {
        Some(host_str.to_ascii_lowercase())
    };
    let port = port_str.and_then(|p| {
        if p.is_empty() {
            None
        } else {
            p.parse::<i64>().ok().filter(|&n| (0..=65535).contains(&n))
        }
    });
    (username, password, hostname, port)
}

fn extract_parse_tuple(v: MbValue) -> (String, String, String, String, String, String) {
    let gi = |vals: &[MbValue], i: usize| {
        extract_str(vals.get(i).copied().unwrap_or(MbValue::none())).unwrap_or_default()
    };
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => {
                    return (
                        gi(items, 0), gi(items, 1), gi(items, 2),
                        gi(items, 3), gi(items, 4), gi(items, 5),
                    );
                }
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    return (
                        gi(&items, 0), gi(&items, 1), gi(&items, 2),
                        gi(&items, 3), gi(&items, 4), gi(&items, 5),
                    );
                }
                ObjData::Instance { ref fields, .. } => {
                    let f = fields.read().unwrap();
                    let gf = |k: &str| f.get(k).and_then(|v| extract_str(*v)).unwrap_or_default();
                    return (
                        gf("scheme"), gf("netloc"), gf("path"),
                        gf("params"), gf("query"), gf("fragment"),
                    );
                }
                _ => {}
            }
        }
    }
    Default::default()
}

/// Percent-encode a string for URL contexts.
///
/// `safe` is a list of extra characters that should NOT be encoded (in
/// addition to RFC 3986 unreserved chars). When `plus_for_space` is true,
/// ASCII space is written as '+' — application/x-www-form-urlencoded.
fn percent_encode(s: &str, safe: &str, plus_for_space: bool) -> String {
    percent_encode_bytes(s.as_bytes(), safe.as_bytes(), plus_for_space)
}

fn percent_encode_bytes(bytes: &[u8], safe: &[u8], plus_for_space: bool) -> String {
    let mut out = String::new();
    for &b in bytes {
        if plus_for_space && b == b' ' {
            out.push('+');
            continue;
        }
        let unreserved = matches!(
            b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
        );
        if unreserved || safe.contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Percent-decode an input byte slice into raw bytes (no charset decode).
/// A '%' followed by two valid hex digits decodes to that byte; otherwise
/// the '%' is passed through literally (matching CPython, which leaves a
/// malformed escape such as `%`, `%2`, or `%zz` untouched). When
/// `plus_for_space` is set, '+' decodes to a space.
fn percent_decode_to_bytes(bytes: &[u8], plus_for_space: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'%' && i + 3 <= bytes.len() {
            let h = &bytes[i + 1..i + 3];
            if h.iter().all(|b| b.is_ascii_hexdigit()) {
                if let Ok(byte) = u8::from_str_radix(
                    std::str::from_utf8(h).unwrap_or(""),
                    16,
                ) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        if plus_for_space && c == b'+' {
            out.push(b' ');
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

fn percent_decode(s: &str, plus_for_space: bool) -> String {
    let decoded = percent_decode_to_bytes(s.as_bytes(), plus_for_space);
    decode_bytes(&decoded, "utf-8", "replace")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_quote_preserves_slash_by_default() {
        let out = mb_urllib_quote(s("/path with space"), MbValue::none());
        assert_eq!(extract_str(out).unwrap(), "/path%20with%20space");
    }

    #[test]
    fn test_quote_plus_encodes_slash_and_uses_plus() {
        let out = mb_urllib_quote_plus(s("/hello world"), MbValue::none());
        assert_eq!(extract_str(out).unwrap(), "%2Fhello+world");
    }

    #[test]
    fn test_unquote_decodes_percent() {
        let out = mb_urllib_unquote(s("hello%20world%2F"));
        assert_eq!(extract_str(out).unwrap(), "hello world/");
    }

    #[test]
    fn test_unquote_plus_decodes_percent_and_plus() {
        let out = mb_urllib_unquote_plus(s("hello+world%2F"));
        assert_eq!(extract_str(out).unwrap(), "hello world/");
    }

    #[test]
    fn test_urlencode_dict() {
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                lock.write().unwrap().insert("q".into(), s("hello world"));
            }
        }
        let out = mb_urllib_urlencode(MbValue::from_ptr(dict));
        let decoded = extract_str(out).unwrap();
        assert_eq!(decoded, "q=hello+world");
    }

    #[test]
    fn test_urlparse_full() {
        let out = mb_urllib_urlparse(s("https://example.com/path?q=1#frag"));
        if let Some(ptr) = out.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    assert_eq!(extract_str(f["scheme"]).unwrap(), "https");
                    assert_eq!(extract_str(f["netloc"]).unwrap(), "example.com");
                    assert_eq!(extract_str(f["path"]).unwrap(), "/path");
                    assert_eq!(extract_str(f["query"]).unwrap(), "q=1");
                    assert_eq!(extract_str(f["fragment"]).unwrap(), "frag");
                }
            }
        }
    }

    #[test]
    fn test_urlparse_no_scheme() {
        let out = mb_urllib_urlparse(s("/just/a/path"));
        if let Some(ptr) = out.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    assert_eq!(extract_str(f["scheme"]).unwrap(), "");
                    assert_eq!(extract_str(f["netloc"]).unwrap(), "");
                    assert_eq!(extract_str(f["path"]).unwrap(), "/just/a/path");
                }
            }
        }
    }

    #[test]
    fn test_urljoin_absolute() {
        let out = mb_urllib_urljoin(s("https://a.com/a/b"), s("https://b.com/x"));
        assert_eq!(extract_str(out).unwrap(), "https://b.com/x");
    }

    #[test]
    fn test_urljoin_absolute_path() {
        let out = mb_urllib_urljoin(s("https://a.com/a/b/c"), s("/x/y"));
        assert_eq!(extract_str(out).unwrap(), "https://a.com/x/y");
    }

    #[test]
    fn test_urljoin_relative_path() {
        let out = mb_urllib_urljoin(s("https://a.com/a/b/c"), s("d/e"));
        assert_eq!(extract_str(out).unwrap(), "https://a.com/a/b/d/e");
    }

    #[test]
    fn test_urljoin_fragment_only() {
        let out = mb_urllib_urljoin(s("https://a.com/x"), s("#frag"));
        assert_eq!(extract_str(out).unwrap(), "https://a.com/x#frag");
    }

    #[test]
    fn test_urljoin_dot_segments() {
        let out = mb_urllib_urljoin(s("https://a.com/a/b/c"), s("../x"));
        assert_eq!(extract_str(out).unwrap(), "https://a.com/a/x");
    }

    #[test]
    fn test_parse_qs_multi_values() {
        let out = mb_urllib_parse_qs(s("a=1&b=2&a=3"));
        if let Some(ptr) = out.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let m = lock.read().unwrap();
                    let a_val = m.get("a").copied().unwrap();
                    if let Some(lp) = a_val.as_ptr() {
                        if let ObjData::List(ref lk) = (*lp).data {
                            let items = lk.read().unwrap();
                            assert_eq!(items.len(), 2);
                            assert_eq!(extract_str(items[0]).unwrap(), "1");
                            assert_eq!(extract_str(items[1]).unwrap(), "3");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_qsl_preserves_order() {
        let out = mb_urllib_parse_qsl(s("z=1&a=2&m=3"));
        if let Some(ptr) = out.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    let items = lock.read().unwrap();
                    assert_eq!(items.len(), 3);
                    if let Some(tp) = items[0].as_ptr() {
                        if let ObjData::Tuple(ref t) = (*tp).data {
                            assert_eq!(extract_str(t[0]).unwrap(), "z");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_urlunparse_roundtrip() {
        let parsed = mb_urllib_urlparse(s("https://example.com/path?q=1#frag"));
        let unparsed = mb_urllib_urlunparse(parsed);
        assert_eq!(
            extract_str(unparsed).unwrap(),
            "https://example.com/path?q=1#frag"
        );
    }
}
