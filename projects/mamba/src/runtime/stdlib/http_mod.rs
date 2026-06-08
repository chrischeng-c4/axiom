use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// http and urllib modules for Mamba (#418).
///
/// Provides a native implementation of the most-used APIs from Python 3.12's
/// `urllib.parse`, `urllib.request`, `urllib.error`, and `http` modules, with
/// `http.client` stubs for import compatibility.
use std::collections::HashMap;

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
    mb_urllib_urlencode(a.get(0).copied().unwrap_or_else(MbValue::none))
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
        (
            "quote_from_bytes",
            dispatch_quote_from_bytes as *const () as usize,
        ),
        ("unquote", dispatch_unquote as *const () as usize),
        ("unquote_plus", dispatch_unquote_plus as *const () as usize),
        (
            "unquote_to_bytes",
            dispatch_unquote_to_bytes as *const () as usize,
        ),
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
            "", "ftp", "http", "gopher", "nntp", "imap", "wais", "file", "https", "shttp", "mms",
            "prospero", "rtsp", "rtspu", "sftp", "svn", "svn+ssh", "ws", "wss",
        ]),
    );
    parse_attrs.insert(
        "uses_netloc".into(),
        list_of_strs(&[
            "",
            "ftp",
            "http",
            "gopher",
            "nntp",
            "telnet",
            "imap",
            "wais",
            "file",
            "mms",
            "https",
            "shttp",
            "snews",
            "prospero",
            "rtsp",
            "rtspu",
            "rsync",
            "svn",
            "svn+ssh",
            "sftp",
            "nfs",
            "git",
            "git+ssh",
            "ws",
            "wss",
            "itms-services",
        ]),
    );
    parse_attrs.insert(
        "uses_params".into(),
        list_of_strs(&[
            "", "ftp", "hdl", "prospero", "http", "imap", "https", "shttp", "rtsp", "rtspu", "sip",
            "sips", "mms", "sftp", "tel",
        ]),
    );
    parse_attrs.insert(
        "non_hierarchical".into(),
        list_of_strs(&[
            "gopher", "hdl", "mailto", "news", "telnet", "wais", "imap", "snews", "sip", "sips",
        ]),
    );
    parse_attrs.insert(
        "uses_query".into(),
        list_of_strs(&[
            "", "http", "wais", "imap", "https", "shttp", "mms", "gopher", "rtsp", "rtspu", "sip",
            "sips",
        ]),
    );
    parse_attrs.insert(
        "uses_fragment".into(),
        list_of_strs(&[
            "", "ftp", "hdl", "http", "gopher", "news", "nntp", "wais", "https", "shttp", "snews",
            "file", "prospero",
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
        "Request",
        "OpenerDirector",
        "BaseHandler",
        "HTTPDefaultErrorHandler",
        "HTTPRedirectHandler",
        "HTTPCookieProcessor",
        "ProxyHandler",
        "HTTPPasswordMgr",
        "HTTPPasswordMgrWithDefaultRealm",
        "HTTPPasswordMgrWithPriorAuth",
        "AbstractBasicAuthHandler",
        "HTTPBasicAuthHandler",
        "ProxyBasicAuthHandler",
        "AbstractDigestAuthHandler",
        "HTTPDigestAuthHandler",
        "ProxyDigestAuthHandler",
        "HTTPHandler",
        "HTTPSHandler",
        "FileHandler",
        "DataHandler",
        "FTPHandler",
        "CacheFTPHandler",
        "UnknownHandler",
        "build_opener",
        "install_opener",
        "getproxies",
        "url2pathname",
        "pathname2url",
    ];
    for name in req_class_shells {
        request_attrs.insert((*name).into(), MbValue::from_func(req_shell));
    }
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
    super::register_module("urllib", HashMap::new());
    super::super::module::MODULES.with(|mods| {
        let values: Vec<(String, MbValue)> = {
            let mods_ref = mods.borrow();
            ["parse", "request", "error"]
                .iter()
                .filter_map(|sub| {
                    mods_ref
                        .get(&format!("urllib.{sub}"))
                        .map(|m| (sub.to_string(), super::super::module::module_to_value(m)))
                })
                .collect()
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
        }
    }
    http_attrs.insert("HTTPStatus".into(), MbValue::from_ptr(http_status));
    super::register_module("http", http_attrs);

    // http.client — callable class shells so `HTTPConnection()` returns a
    // dict sentinel rather than a non-callable string. Real-world callers
    // (requests, urllib3, http-shim probes) instantiate these classes at
    // import / probe time; surface-only conformance keeps the probe
    // chain alive without a real network stack.
    let mut client_attrs = HashMap::new();
    let client_addr = dispatch_class_shell as *const () as usize;
    for name in &[
        "HTTPConnection",
        "HTTPSConnection",
        "HTTPResponse",
        "HTTPMessage",
        "HTTPException",
        "NotConnected",
        "BadStatusLine",
        "IncompleteRead",
        "InvalidURL",
        "UnknownProtocol",
        "UnknownTransferEncoding",
        "UnimplementedFileMode",
        "LineTooLong",
        "RemoteDisconnected",
        "ResponseNotReady",
        "ImproperConnectionState",
        "CannotSendRequest",
        "CannotSendHeader",
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
    client_attrs.insert("HTTP_PORT".into(), MbValue::from_int(80));
    client_attrs.insert("HTTPS_PORT".into(), MbValue::from_int(443));
    client_attrs.insert("responses".into(), MbValue::from_ptr(MbObject::new_dict()));
    super::register_module("http.client", client_attrs);

    // http.server — same callable-shell treatment.
    let mut server_attrs = HashMap::new();
    let server_addr = dispatch_class_shell as *const () as usize;
    for name in &[
        "HTTPServer",
        "BaseHTTPRequestHandler",
        "SimpleHTTPRequestHandler",
        "CGIHTTPRequestHandler",
        "ThreadingHTTPServer",
    ] {
        server_attrs.insert(name.to_string(), MbValue::from_func(server_addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(server_addr as u64);
    });
    super::register_module("http.server", server_attrs);

    // http.cookies stub
    let mut cookies_attrs = HashMap::new();
    for name in &["BaseCookie", "SimpleCookie", "CookieError", "Morsel"] {
        cookies_attrs.insert(
            name.to_string(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
        );
    }
    super::register_module("http.cookies", cookies_attrs);

    // Wire http submodules under http.
    super::super::module::MODULES.with(|mods| {
        let values: Vec<(String, MbValue)> = {
            let mods_ref = mods.borrow();
            ["client", "server", "cookies"]
                .iter()
                .filter_map(|sub| {
                    mods_ref
                        .get(&format!("http.{sub}"))
                        .map(|m| (sub.to_string(), super::super::module::module_to_value(m)))
                })
                .collect()
        };
        if let Some(http_mod) = mods.borrow_mut().get_mut("http") {
            for (k, v) in values {
                http_mod.attrs.insert(k, v);
            }
        }
    });
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
        .or_else(|| {
            v.as_bool()
                .map(|b| if b { "True".into() } else { "False".into() })
        })
        .unwrap_or_default()
}

// ── Runtime functions ──

/// urllib.parse.quote(string, safe='/') → percent-encoded string.
///
/// Default `safe` is '/' per CPython: slashes in a path are preserved.
pub fn mb_urllib_quote(val: MbValue, safe: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let safe_chars = if safe.is_none() {
        "/".to_string()
    } else {
        extract_str(safe).unwrap_or_else(|| "/".to_string())
    };
    MbValue::from_ptr(MbObject::new_str(percent_encode(&s, &safe_chars, false)))
}

/// urllib.parse.quote_from_bytes(bytes, safe='/') -> percent-encoded string.
pub fn mb_urllib_quote_from_bytes(val: MbValue, safe: MbValue) -> MbValue {
    let Some(bytes) = extract_bytes_like(val) else {
        return raise_type_error("quote_from_bytes() expected bytes");
    };
    let safe_bytes = extract_safe_bytes(safe, b"/");
    MbValue::from_ptr(MbObject::new_str(percent_encode_bytes(
        &bytes,
        &safe_bytes,
        false,
    )))
}

/// urllib.parse.quote_plus(string, safe='') → spaces become '+', rest %-encoded.
pub fn mb_urllib_quote_plus(val: MbValue, safe: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let safe_chars = extract_str(safe).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(percent_encode(&s, &safe_chars, true)))
}

/// urllib.parse.unquote(string) → decode %XX sequences; leave '+' untouched.
pub fn mb_urllib_unquote(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(percent_decode(&s, false)))
}

/// urllib.parse.unquote_plus(string) → decode %XX and '+' → ' '.
pub fn mb_urllib_unquote_plus(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(percent_decode(&s, true)))
}

/// urllib.parse.urlencode(params) → "k1=v1&k2=v2" — accepts dict or list of 2-tuples.
pub fn mb_urllib_urlencode(params: MbValue) -> MbValue {
    let mut parts = Vec::new();
    if let Some(ptr) = params.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    for (k, v) in map.iter() {
                        let key_str = k.to_string();
                        let val_str = value_as_query_string(*v);
                        parts.push(format!(
                            "{}={}",
                            percent_encode(&key_str, "", true),
                            percent_encode(&val_str, "", true),
                        ));
                    }
                }
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    for item in items.iter() {
                        if let Some(iptr) = item.as_ptr() {
                            if let ObjData::Tuple(ref tup) = (*iptr).data {
                                if tup.len() == 2 {
                                    let k = extract_str(tup[0]).unwrap_or_default();
                                    let v = value_as_query_string(tup[1]);
                                    parts.push(format!(
                                        "{}={}",
                                        percent_encode(&k, "", true),
                                        percent_encode(&v, "", true),
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    MbValue::from_ptr(MbObject::new_str(parts.join("&")))
}

/// urllib.parse.urlparse(url) → ParseResult-like Instance with 6 str fields.
///
/// Fields: scheme, netloc, path, params, query, fragment. Attribute access
/// works via the standard Instance field lookup; index access is not
/// supported yet (would require Tuple-backed storage).
pub fn mb_urllib_urlparse(val: MbValue) -> MbValue {
    let url = extract_str(val).unwrap_or_default();
    let (scheme, rest) = match url.find("://") {
        Some(i)
            if url[..i]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') =>
        {
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
        Some(i) => (
            path_query_frag[..i].to_string(),
            path_query_frag[i + 1..].to_string(),
        ),
        None => (path_query_frag, String::new()),
    };
    let (path_params, query) = match path_query.find('?') {
        Some(i) => (path_query[..i].to_string(), path_query[i + 1..].to_string()),
        None => (path_query, String::new()),
    };
    let (path, params) = match path_params.rfind(';') {
        Some(i) => (
            path_params[..i].to_string(),
            path_params[i + 1..].to_string(),
        ),
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
        if url[..i]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
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
    let mut result = if leading_slash {
        "/".to_string()
    } else {
        String::new()
    };
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
                let entry = map
                    .entry(dk)
                    .or_insert_with(|| MbValue::from_ptr(MbObject::new_list(vec![])));
                if let Some(lp) = entry.as_ptr() {
                    if let ObjData::List(ref list_lock) = (*lp).data {
                        list_lock
                            .write()
                            .unwrap()
                            .push(MbValue::from_ptr(MbObject::new_str(decoded_val)));
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
    fields.insert(
        "fragment".into(),
        MbValue::from_ptr(MbObject::new_str(fragment)),
    );
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
    let s = extract_str(val).unwrap_or_default();
    let bytes_in = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes_in.len());
    let mut i = 0;
    while i < bytes_in.len() {
        if bytes_in[i] == b'%' && i + 2 < bytes_in.len() {
            let hex = std::str::from_utf8(&bytes_in[i + 1..i + 3]).unwrap_or("");
            if let Ok(b) = u8::from_str_radix(hex, 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes_in[i]);
        i += 1;
    }
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
            map.insert("url".into(), MbValue::from_ptr(MbObject::new_str(u)));
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
    scheme: String,
    netloc: String,
    path: String,
    params: String,
    query: String,
    fragment: String,
) -> MbValue {
    let mut fields = FxHashMap::with_capacity_and_hasher(6, Default::default());
    fields.insert(
        "scheme".into(),
        MbValue::from_ptr(MbObject::new_str(scheme)),
    );
    fields.insert(
        "netloc".into(),
        MbValue::from_ptr(MbObject::new_str(netloc)),
    );
    fields.insert("path".into(), MbValue::from_ptr(MbObject::new_str(path)));
    fields.insert(
        "params".into(),
        MbValue::from_ptr(MbObject::new_str(params)),
    );
    fields.insert("query".into(), MbValue::from_ptr(MbObject::new_str(query)));
    fields.insert(
        "fragment".into(),
        MbValue::from_ptr(MbObject::new_str(fragment)),
    );
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

fn extract_parse_tuple(v: MbValue) -> (String, String, String, String, String, String) {
    let gi = |vals: &[MbValue], i: usize| {
        extract_str(vals.get(i).copied().unwrap_or(MbValue::none())).unwrap_or_default()
    };
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => {
                    return (
                        gi(items, 0),
                        gi(items, 1),
                        gi(items, 2),
                        gi(items, 3),
                        gi(items, 4),
                        gi(items, 5),
                    );
                }
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    return (
                        gi(&items, 0),
                        gi(&items, 1),
                        gi(&items, 2),
                        gi(&items, 3),
                        gi(&items, 4),
                        gi(&items, 5),
                    );
                }
                ObjData::Instance { ref fields, .. } => {
                    let f = fields.read().unwrap();
                    let gf = |k: &str| f.get(k).and_then(|v| extract_str(*v)).unwrap_or_default();
                    return (
                        gf("scheme"),
                        gf("netloc"),
                        gf("path"),
                        gf("params"),
                        gf("query"),
                        gf("fragment"),
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

fn percent_decode(s: &str, plus_for_space: bool) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        if plus_for_space && c == b'+' {
            out.push(b' ');
        } else {
            out.push(c);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
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
