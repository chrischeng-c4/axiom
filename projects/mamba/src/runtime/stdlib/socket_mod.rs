use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// socket module for Mamba (#418).
///
/// Provides: socket(), connect, bind, listen, accept, send, recv, close
/// Address family and socket type constants.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            $fn()
        }
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

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_binary!(d_socket_new, mb_socket_new);
disp_nullary!(d_gethostname, mb_socket_gethostname);
disp_unary!(d_gethostbyname, mb_socket_gethostbyname);
disp_binary!(d_getaddrinfo, mb_socket_getaddrinfo);
disp_binary!(d_create_connection, mb_socket_create_connection);
disp_unary!(d_create_server, mb_socket_create_server);
disp_unary!(d_htons, mb_socket_htons);
disp_unary!(d_htonl, mb_socket_htonl);
disp_unary!(d_ntohs, mb_socket_ntohs);
disp_unary!(d_ntohl, mb_socket_ntohl);

/// Register the socket module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Address families
    attrs.insert("AF_INET".into(), MbValue::from_int(2));
    attrs.insert("AF_INET6".into(), MbValue::from_int(10));
    attrs.insert("AF_UNIX".into(), MbValue::from_int(1));

    // Socket types
    attrs.insert("SOCK_STREAM".into(), MbValue::from_int(1));
    attrs.insert("SOCK_DGRAM".into(), MbValue::from_int(2));

    let dispatchers: Vec<(&str, usize)> = vec![
        ("socket", d_socket_new as *const () as usize),
        ("gethostname", d_gethostname as *const () as usize),
        ("gethostbyname", d_gethostbyname as *const () as usize),
        ("getaddrinfo", d_getaddrinfo as *const () as usize),
        (
            "create_connection",
            d_create_connection as *const () as usize,
        ),
        ("create_server", d_create_server as *const () as usize),
        ("htons", d_htons as *const () as usize),
        ("htonl", d_htonl as *const () as usize),
        ("ntohs", d_ntohs as *const () as usize),
        ("ntohl", d_ntohl as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("socket", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn socket_int_arg(value: MbValue, fn_name: &str) -> Option<i64> {
    match value.as_int() {
        Some(n) => Some(n),
        None => {
            raise_type_error(&format!("{fn_name}() argument must be an integer"));
            None
        }
    }
}

/// socket.socket(family, type) -> socket instance dict
pub fn mb_socket_new(family: MbValue, stype: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("socket".to_string())),
            );
            map.insert(
                "family".into(),
                MbValue::from_int(family.as_int().unwrap_or(2)),
            );
            map.insert(
                "type".into(),
                MbValue::from_int(stype.as_int().unwrap_or(1)),
            );
            map.insert("connected".into(), MbValue::from_bool(false));
            map.insert("closed".into(), MbValue::from_bool(false));
        }
    }
    MbValue::from_ptr(dict)
}

/// socket.connect(sock, (host, port)) -> None
pub fn mb_socket_connect(sock: MbValue, addr: MbValue) -> MbValue {
    if let Some(ptr) = sock.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("connected".into(), MbValue::from_bool(true));
                map.insert("addr".into(), addr);
            }
        }
    }
    MbValue::none()
}

/// socket.send(sock, data) -> bytes sent
pub fn mb_socket_send(_sock: MbValue, data: MbValue) -> MbValue {
    let s = extract_str(data).unwrap_or_default();
    MbValue::from_int(s.len() as i64)
}

/// socket.recv(sock, bufsize) -> received data (stub: empty string)
pub fn mb_socket_recv(_sock: MbValue, _bufsize: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

/// socket.close(sock) -> None
pub fn mb_socket_close(sock: MbValue) -> MbValue {
    if let Some(ptr) = sock.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("closed".into(), MbValue::from_bool(true));
                map.insert("connected".into(), MbValue::from_bool(false));
            }
        }
    }
    MbValue::none()
}

/// socket.bind(sock, addr) -> None
pub fn mb_socket_bind(sock: MbValue, addr: MbValue) -> MbValue {
    if let Some(ptr) = sock.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("addr".into(), addr);
                map.insert("bound".into(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::none()
}

/// socket.listen(sock, backlog) -> None
pub fn mb_socket_listen(sock: MbValue, _backlog: MbValue) -> MbValue {
    if let Some(ptr) = sock.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("listening".into(), MbValue::from_bool(true));
            }
        }
    }
    MbValue::none()
}

/// socket.gethostname() -> hostname string
pub fn mb_socket_gethostname() -> MbValue {
    let name = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "localhost".to_string());
    MbValue::from_ptr(MbObject::new_str(name))
}

/// socket.gethostbyname(name) -> IP string (stub: returns "127.0.0.1")
pub fn mb_socket_gethostbyname(_name: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("127.0.0.1".to_string()))
}

/// socket.getaddrinfo(host, port) -> list of (family, type, proto, canonname, sockaddr)
/// Stub: returns a single-entry list with AF_INET / SOCK_STREAM matching the
/// resolved hostname (loopback fallback) and the supplied port.
pub fn mb_socket_getaddrinfo(host: MbValue, port: MbValue) -> MbValue {
    let host_s = extract_str(host).unwrap_or_else(|| "127.0.0.1".to_string());
    let port_i = port.as_int().unwrap_or(0);
    // sockaddr tuple: (host, port)
    let sockaddr = MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(host_s.clone())),
        MbValue::from_int(port_i),
    ]);
    let entry = MbObject::new_tuple(vec![
        MbValue::from_int(2),                                // AF_INET
        MbValue::from_int(1),                                // SOCK_STREAM
        MbValue::from_int(0),                                // proto
        MbValue::from_ptr(MbObject::new_str(String::new())), // canonname
        MbValue::from_ptr(sockaddr),
    ]);
    MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(entry)]))
}

/// socket.create_connection((host, port)) -> connected socket dict
pub fn mb_socket_create_connection(addr: MbValue, _timeout: MbValue) -> MbValue {
    let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
    mb_socket_connect(sock, addr);
    sock
}

/// socket.create_server((host, port)) -> bound + listening socket dict
pub fn mb_socket_create_server(addr: MbValue) -> MbValue {
    let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
    mb_socket_bind(sock, addr);
    mb_socket_listen(sock, MbValue::from_int(128));
    sock
}

/// socket.htons(x) -> integer in network byte order.
pub fn mb_socket_htons(value: MbValue) -> MbValue {
    let Some(n) = socket_int_arg(value, "htons") else {
        return MbValue::none();
    };
    MbValue::from_int((n as u16).to_be() as i64)
}

/// socket.htonl(x) -> integer in network byte order.
pub fn mb_socket_htonl(value: MbValue) -> MbValue {
    let Some(n) = socket_int_arg(value, "htonl") else {
        return MbValue::none();
    };
    MbValue::from_int((n as u32).to_be() as i64)
}

/// socket.ntohs(x) -> integer in host byte order.
pub fn mb_socket_ntohs(value: MbValue) -> MbValue {
    let Some(n) = socket_int_arg(value, "ntohs") else {
        return MbValue::none();
    };
    MbValue::from_int(u16::from_be(n as u16) as i64)
}

/// socket.ntohl(x) -> integer in host byte order.
pub fn mb_socket_ntohl(value: MbValue) -> MbValue {
    let Some(n) = socket_int_arg(value, "ntohl") else {
        return MbValue::none();
    };
    MbValue::from_int(u32::from_be(n as u32) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict_get_bool(val: MbValue, key: &str) -> Option<bool> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).and_then(|v| v.as_bool())
            } else {
                None
            }
        })
    }

    fn dict_get_int(val: MbValue, key: &str) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).and_then(|v| v.as_int())
            } else {
                None
            }
        })
    }

    fn str_val(s: MbValue) -> Option<String> {
        s.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref st) = (*ptr).data {
                Some(st.clone())
            } else {
                None
            }
        })
    }

    // --- extract_str ---
    #[test]
    fn test_extract_str_str_value() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(extract_str(s), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_str_non_str() {
        assert_eq!(extract_str(MbValue::from_int(1)), None);
    }

    // --- mb_socket_new ---
    #[test]
    fn test_socket_new_explicit_family_type() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        assert_eq!(dict_get_int(sock, "family"), Some(2));
        assert_eq!(dict_get_int(sock, "type"), Some(1));
    }

    #[test]
    fn test_socket_new_none_family_defaults_to_2() {
        let sock = mb_socket_new(MbValue::none(), MbValue::none());
        assert_eq!(dict_get_int(sock, "family"), Some(2));
        assert_eq!(dict_get_int(sock, "type"), Some(1));
    }

    // --- mb_socket_connect ---
    #[test]
    fn test_socket_connect_sets_connected() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        let addr = MbValue::from_ptr(MbObject::new_str("127.0.0.1:8080".to_string()));
        mb_socket_connect(sock, addr);
        assert_eq!(dict_get_bool(sock, "connected"), Some(true));
    }

    #[test]
    fn test_socket_connect_null_noop() {
        let addr = MbValue::from_ptr(MbObject::new_str("127.0.0.1:0".to_string()));
        mb_socket_connect(MbValue::none(), addr); // should not panic
    }

    // --- mb_socket_send ---
    #[test]
    fn test_socket_send_str_returns_len() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        let data = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let result = mb_socket_send(sock, data);
        assert_eq!(result.as_int(), Some(5));
    }

    #[test]
    fn test_socket_send_non_str_returns_0() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        let result = mb_socket_send(sock, MbValue::from_int(0));
        assert_eq!(result.as_int(), Some(0));
    }

    // --- mb_socket_recv ---
    #[test]
    fn test_socket_recv_returns_empty_str() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        let result = mb_socket_recv(sock, MbValue::from_int(1024));
        assert_eq!(str_val(result), Some(String::new()));
    }

    // --- mb_socket_close ---
    #[test]
    fn test_socket_create_close() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        mb_socket_close(sock);
        assert_eq!(dict_get_bool(sock, "closed"), Some(true));
        assert_eq!(dict_get_bool(sock, "connected"), Some(false));
    }

    #[test]
    fn test_socket_close_null_noop() {
        mb_socket_close(MbValue::none()); // should not panic
    }

    // --- mb_socket_bind ---
    #[test]
    fn test_socket_bind_sets_bound() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        let addr = MbValue::from_ptr(MbObject::new_str("127.0.0.1:0".to_string()));
        mb_socket_bind(sock, addr);
        assert_eq!(dict_get_bool(sock, "bound"), Some(true));
    }

    #[test]
    fn test_socket_bind_null_noop() {
        mb_socket_bind(MbValue::none(), MbValue::none()); // should not panic
    }

    // --- mb_socket_listen ---
    #[test]
    fn test_socket_listen_sets_listening() {
        let sock = mb_socket_new(MbValue::from_int(2), MbValue::from_int(1));
        mb_socket_listen(sock, MbValue::from_int(5));
        assert_eq!(dict_get_bool(sock, "listening"), Some(true));
    }

    #[test]
    fn test_socket_listen_null_noop() {
        mb_socket_listen(MbValue::none(), MbValue::from_int(5)); // should not panic
    }

    // --- mb_socket_gethostname ---
    #[test]
    fn test_gethostname_hostname_set() {
        std::env::set_var("HOSTNAME", "my-socket-host");
        let result = mb_socket_gethostname();
        std::env::remove_var("HOSTNAME");
        assert_eq!(str_val(result), Some("my-socket-host".to_string()));
    }

    #[test]
    fn test_gethostname_fallback_localhost() {
        let orig_hostname = std::env::var("HOSTNAME").ok();
        let orig_host = std::env::var("HOST").ok();
        std::env::remove_var("HOSTNAME");
        std::env::remove_var("HOST");
        let result = mb_socket_gethostname();
        if let Some(h) = orig_hostname {
            std::env::set_var("HOSTNAME", h);
        }
        if let Some(h) = orig_host {
            std::env::set_var("HOST", h);
        }
        let s = str_val(result).unwrap_or_default();
        assert_eq!(s, "localhost");
    }

    // --- mb_socket_gethostbyname ---
    #[test]
    fn test_gethostbyname_returns_loopback() {
        let name = MbValue::from_ptr(MbObject::new_str("localhost".to_string()));
        let result = mb_socket_gethostbyname(name);
        assert_eq!(str_val(result), Some("127.0.0.1".to_string()));
    }
}
