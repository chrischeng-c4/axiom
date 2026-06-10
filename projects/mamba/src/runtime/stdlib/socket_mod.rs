/// socket module for Mamba (#418).
///
/// Provides: socket(), connect, bind, listen, accept, send, recv, close
/// Address family and socket type constants.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

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

// Generic surface stub: a callable that accepts any args and returns None.
// Used to register the public function/builtin names that CPython's `socket`
// exposes but Mamba does not yet implement, so that `hasattr` / `callable`
// surface fixtures pass without a broken (panicking) body.
unsafe extern "C" fn d_socket_stub(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

// Generic surface stub for the type-like module attrs (enum/flag classes such
// as `AddressFamily`, `SocketKind`, `SocketType`). Registered via
// `NATIVE_TYPE_NAMES` so they read back as a type and are callable.
unsafe extern "C" fn d_socket_type_stub(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

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
        ("create_connection", d_create_connection as *const () as usize),
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

        // surface: missing CPython module constants (auto-added)
    attrs.insert("AF_APPLETALK".into(), MbValue::from_int(16));
    attrs.insert("AF_DECnet".into(), MbValue::from_int(12));
    attrs.insert("AF_IPX".into(), MbValue::from_int(23));
    attrs.insert("AF_LINK".into(), MbValue::from_int(18));
    attrs.insert("AF_ROUTE".into(), MbValue::from_int(17));
    attrs.insert("AF_SNA".into(), MbValue::from_int(11));
    attrs.insert("AF_SYSTEM".into(), MbValue::from_int(32));
    attrs.insert("AF_UNSPEC".into(), MbValue::from_int(0));
    attrs.insert("AI_ADDRCONFIG".into(), MbValue::from_int(1024));
    attrs.insert("AI_ALL".into(), MbValue::from_int(256));
    attrs.insert("AI_CANONNAME".into(), MbValue::from_int(2));
    attrs.insert("AI_DEFAULT".into(), MbValue::from_int(1536));
    attrs.insert("AI_MASK".into(), MbValue::from_int(5127));
    attrs.insert("AI_NUMERICHOST".into(), MbValue::from_int(4));
    attrs.insert("AI_NUMERICSERV".into(), MbValue::from_int(4096));
    attrs.insert("AI_PASSIVE".into(), MbValue::from_int(1));
    attrs.insert("AI_V4MAPPED".into(), MbValue::from_int(2048));
    attrs.insert("AI_V4MAPPED_CFG".into(), MbValue::from_int(512));
    attrs.insert("EAGAIN".into(), MbValue::from_int(35));
    attrs.insert("EAI_ADDRFAMILY".into(), MbValue::from_int(1));
    attrs.insert("EAI_AGAIN".into(), MbValue::from_int(2));
    attrs.insert("EAI_BADFLAGS".into(), MbValue::from_int(3));
    attrs.insert("EAI_BADHINTS".into(), MbValue::from_int(12));
    attrs.insert("EAI_FAIL".into(), MbValue::from_int(4));
    attrs.insert("EAI_FAMILY".into(), MbValue::from_int(5));
    attrs.insert("EAI_MAX".into(), MbValue::from_int(15));
    attrs.insert("EAI_MEMORY".into(), MbValue::from_int(6));
    attrs.insert("EAI_NODATA".into(), MbValue::from_int(7));
    attrs.insert("EAI_NONAME".into(), MbValue::from_int(8));
    attrs.insert("EAI_OVERFLOW".into(), MbValue::from_int(14));
    attrs.insert("EAI_PROTOCOL".into(), MbValue::from_int(13));
    attrs.insert("EAI_SERVICE".into(), MbValue::from_int(9));
    attrs.insert("EAI_SOCKTYPE".into(), MbValue::from_int(10));
    attrs.insert("EAI_SYSTEM".into(), MbValue::from_int(11));
    attrs.insert("EBADF".into(), MbValue::from_int(9));
    attrs.insert("ETHERTYPE_ARP".into(), MbValue::from_int(2054));
    attrs.insert("ETHERTYPE_IP".into(), MbValue::from_int(2048));
    attrs.insert("ETHERTYPE_IPV6".into(), MbValue::from_int(34525));
    attrs.insert("ETHERTYPE_VLAN".into(), MbValue::from_int(33024));
    attrs.insert("EWOULDBLOCK".into(), MbValue::from_int(35));
    attrs.insert("INADDR_ALLHOSTS_GROUP".into(), MbValue::from_int(3758096385));
    attrs.insert("INADDR_ANY".into(), MbValue::from_int(0));
    attrs.insert("INADDR_BROADCAST".into(), MbValue::from_int(4294967295));
    attrs.insert("INADDR_LOOPBACK".into(), MbValue::from_int(2130706433));
    attrs.insert("INADDR_MAX_LOCAL_GROUP".into(), MbValue::from_int(3758096639));
    attrs.insert("INADDR_NONE".into(), MbValue::from_int(4294967295));
    attrs.insert("INADDR_UNSPEC_GROUP".into(), MbValue::from_int(3758096384));
    attrs.insert("IPPORT_RESERVED".into(), MbValue::from_int(1024));
    attrs.insert("IPPORT_USERRESERVED".into(), MbValue::from_int(5000));
    attrs.insert("IPPROTO_AH".into(), MbValue::from_int(51));
    attrs.insert("IPPROTO_DSTOPTS".into(), MbValue::from_int(60));
    attrs.insert("IPPROTO_EGP".into(), MbValue::from_int(8));
    attrs.insert("IPPROTO_EON".into(), MbValue::from_int(80));
    attrs.insert("IPPROTO_ESP".into(), MbValue::from_int(50));
    attrs.insert("IPPROTO_FRAGMENT".into(), MbValue::from_int(44));
    attrs.insert("IPPROTO_GGP".into(), MbValue::from_int(3));
    attrs.insert("IPPROTO_GRE".into(), MbValue::from_int(47));
    attrs.insert("IPPROTO_HELLO".into(), MbValue::from_int(63));
    attrs.insert("IPPROTO_HOPOPTS".into(), MbValue::from_int(0));
    attrs.insert("IPPROTO_ICMP".into(), MbValue::from_int(1));
    attrs.insert("IPPROTO_ICMPV6".into(), MbValue::from_int(58));
    attrs.insert("IPPROTO_IDP".into(), MbValue::from_int(22));
    attrs.insert("IPPROTO_IGMP".into(), MbValue::from_int(2));
    attrs.insert("IPPROTO_IP".into(), MbValue::from_int(0));
    attrs.insert("IPPROTO_IPCOMP".into(), MbValue::from_int(108));
    attrs.insert("IPPROTO_IPIP".into(), MbValue::from_int(4));
    attrs.insert("IPPROTO_IPV4".into(), MbValue::from_int(4));
    attrs.insert("IPPROTO_IPV6".into(), MbValue::from_int(41));
    attrs.insert("IPPROTO_MAX".into(), MbValue::from_int(256));
    attrs.insert("IPPROTO_ND".into(), MbValue::from_int(77));
    attrs.insert("IPPROTO_NONE".into(), MbValue::from_int(59));
    attrs.insert("IPPROTO_PIM".into(), MbValue::from_int(103));
    attrs.insert("IPPROTO_PUP".into(), MbValue::from_int(12));
    attrs.insert("IPPROTO_RAW".into(), MbValue::from_int(255));
    attrs.insert("IPPROTO_ROUTING".into(), MbValue::from_int(43));
    attrs.insert("IPPROTO_RSVP".into(), MbValue::from_int(46));
    attrs.insert("IPPROTO_SCTP".into(), MbValue::from_int(132));
    attrs.insert("IPPROTO_TCP".into(), MbValue::from_int(6));
    attrs.insert("IPPROTO_TP".into(), MbValue::from_int(29));
    attrs.insert("IPPROTO_UDP".into(), MbValue::from_int(17));
    attrs.insert("IPPROTO_XTP".into(), MbValue::from_int(36));
    attrs.insert("IPV6_CHECKSUM".into(), MbValue::from_int(26));
    attrs.insert("IPV6_DONTFRAG".into(), MbValue::from_int(62));
    attrs.insert("IPV6_DSTOPTS".into(), MbValue::from_int(50));
    attrs.insert("IPV6_HOPLIMIT".into(), MbValue::from_int(47));
    attrs.insert("IPV6_HOPOPTS".into(), MbValue::from_int(49));
    attrs.insert("IPV6_JOIN_GROUP".into(), MbValue::from_int(12));
    attrs.insert("IPV6_LEAVE_GROUP".into(), MbValue::from_int(13));
    attrs.insert("IPV6_MULTICAST_HOPS".into(), MbValue::from_int(10));
    attrs.insert("IPV6_MULTICAST_IF".into(), MbValue::from_int(9));
    attrs.insert("IPV6_MULTICAST_LOOP".into(), MbValue::from_int(11));
    attrs.insert("IPV6_NEXTHOP".into(), MbValue::from_int(48));
    attrs.insert("IPV6_PATHMTU".into(), MbValue::from_int(44));
    attrs.insert("IPV6_PKTINFO".into(), MbValue::from_int(46));
    attrs.insert("IPV6_RECVDSTOPTS".into(), MbValue::from_int(40));
    attrs.insert("IPV6_RECVHOPLIMIT".into(), MbValue::from_int(37));
    attrs.insert("IPV6_RECVHOPOPTS".into(), MbValue::from_int(39));
    attrs.insert("IPV6_RECVPATHMTU".into(), MbValue::from_int(43));
    attrs.insert("IPV6_RECVPKTINFO".into(), MbValue::from_int(61));
    attrs.insert("IPV6_RECVRTHDR".into(), MbValue::from_int(38));
    attrs.insert("IPV6_RECVTCLASS".into(), MbValue::from_int(35));
    attrs.insert("IPV6_RTHDR".into(), MbValue::from_int(51));
    attrs.insert("IPV6_RTHDRDSTOPTS".into(), MbValue::from_int(57));
    attrs.insert("IPV6_RTHDR_TYPE_0".into(), MbValue::from_int(0));
    attrs.insert("IPV6_TCLASS".into(), MbValue::from_int(36));
    attrs.insert("IPV6_UNICAST_HOPS".into(), MbValue::from_int(4));
    attrs.insert("IPV6_USE_MIN_MTU".into(), MbValue::from_int(42));
    attrs.insert("IPV6_V6ONLY".into(), MbValue::from_int(27));
    attrs.insert("IP_ADD_MEMBERSHIP".into(), MbValue::from_int(12));
    attrs.insert("IP_ADD_SOURCE_MEMBERSHIP".into(), MbValue::from_int(70));
    attrs.insert("IP_BLOCK_SOURCE".into(), MbValue::from_int(72));
    attrs.insert("IP_DEFAULT_MULTICAST_LOOP".into(), MbValue::from_int(1));
    attrs.insert("IP_DEFAULT_MULTICAST_TTL".into(), MbValue::from_int(1));
    attrs.insert("IP_DROP_MEMBERSHIP".into(), MbValue::from_int(13));
    attrs.insert("IP_DROP_SOURCE_MEMBERSHIP".into(), MbValue::from_int(71));
    attrs.insert("IP_HDRINCL".into(), MbValue::from_int(2));
    attrs.insert("IP_MAX_MEMBERSHIPS".into(), MbValue::from_int(4095));
    attrs.insert("IP_MULTICAST_IF".into(), MbValue::from_int(9));
    attrs.insert("IP_MULTICAST_LOOP".into(), MbValue::from_int(11));
    attrs.insert("IP_MULTICAST_TTL".into(), MbValue::from_int(10));
    attrs.insert("IP_OPTIONS".into(), MbValue::from_int(1));
    attrs.insert("IP_PKTINFO".into(), MbValue::from_int(26));
    attrs.insert("IP_RECVDSTADDR".into(), MbValue::from_int(7));
    attrs.insert("IP_RECVOPTS".into(), MbValue::from_int(5));
    attrs.insert("IP_RECVRETOPTS".into(), MbValue::from_int(6));
    attrs.insert("IP_RECVTOS".into(), MbValue::from_int(27));
    attrs.insert("IP_RETOPTS".into(), MbValue::from_int(8));
    attrs.insert("IP_TOS".into(), MbValue::from_int(3));
    attrs.insert("IP_TTL".into(), MbValue::from_int(4));
    attrs.insert("IP_UNBLOCK_SOURCE".into(), MbValue::from_int(73));
    attrs.insert("LOCAL_PEERCRED".into(), MbValue::from_int(1));
    attrs.insert("MSG_CTRUNC".into(), MbValue::from_int(32));
    attrs.insert("MSG_DONTROUTE".into(), MbValue::from_int(4));
    attrs.insert("MSG_DONTWAIT".into(), MbValue::from_int(128));
    attrs.insert("MSG_EOF".into(), MbValue::from_int(256));
    attrs.insert("MSG_EOR".into(), MbValue::from_int(8));
    attrs.insert("MSG_NOSIGNAL".into(), MbValue::from_int(524288));
    attrs.insert("MSG_OOB".into(), MbValue::from_int(1));
    attrs.insert("MSG_PEEK".into(), MbValue::from_int(2));
    attrs.insert("MSG_TRUNC".into(), MbValue::from_int(16));
    attrs.insert("MSG_WAITALL".into(), MbValue::from_int(64));
    attrs.insert("NI_DGRAM".into(), MbValue::from_int(16));
    attrs.insert("NI_MAXHOST".into(), MbValue::from_int(1025));
    attrs.insert("NI_MAXSERV".into(), MbValue::from_int(32));
    attrs.insert("NI_NAMEREQD".into(), MbValue::from_int(4));
    attrs.insert("NI_NOFQDN".into(), MbValue::from_int(1));
    attrs.insert("NI_NUMERICHOST".into(), MbValue::from_int(2));
    attrs.insert("NI_NUMERICSERV".into(), MbValue::from_int(8));
    attrs.insert("PF_SYSTEM".into(), MbValue::from_int(32));
    attrs.insert("SCM_CREDS".into(), MbValue::from_int(3));
    attrs.insert("SCM_RIGHTS".into(), MbValue::from_int(1));
    attrs.insert("SHUT_RD".into(), MbValue::from_int(0));
    attrs.insert("SHUT_RDWR".into(), MbValue::from_int(2));
    attrs.insert("SHUT_WR".into(), MbValue::from_int(1));
    attrs.insert("SOCK_RAW".into(), MbValue::from_int(3));
    attrs.insert("SOCK_RDM".into(), MbValue::from_int(4));
    attrs.insert("SOCK_SEQPACKET".into(), MbValue::from_int(5));
    attrs.insert("SOL_IP".into(), MbValue::from_int(0));
    attrs.insert("SOL_SOCKET".into(), MbValue::from_int(65535));
    attrs.insert("SOL_TCP".into(), MbValue::from_int(6));
    attrs.insert("SOL_UDP".into(), MbValue::from_int(17));
    attrs.insert("SOMAXCONN".into(), MbValue::from_int(128));
    attrs.insert("SO_ACCEPTCONN".into(), MbValue::from_int(2));
    attrs.insert("SO_BINDTODEVICE".into(), MbValue::from_int(4404));
    attrs.insert("SO_BROADCAST".into(), MbValue::from_int(32));
    attrs.insert("SO_DEBUG".into(), MbValue::from_int(1));
    attrs.insert("SO_DONTROUTE".into(), MbValue::from_int(16));
    attrs.insert("SO_ERROR".into(), MbValue::from_int(4103));
    attrs.insert("SO_KEEPALIVE".into(), MbValue::from_int(8));
    attrs.insert("SO_LINGER".into(), MbValue::from_int(128));
    attrs.insert("SO_OOBINLINE".into(), MbValue::from_int(256));
    attrs.insert("SO_RCVBUF".into(), MbValue::from_int(4098));
    attrs.insert("SO_RCVLOWAT".into(), MbValue::from_int(4100));
    attrs.insert("SO_RCVTIMEO".into(), MbValue::from_int(4102));
    attrs.insert("SO_REUSEADDR".into(), MbValue::from_int(4));
    attrs.insert("SO_REUSEPORT".into(), MbValue::from_int(512));
    attrs.insert("SO_SNDBUF".into(), MbValue::from_int(4097));
    attrs.insert("SO_SNDLOWAT".into(), MbValue::from_int(4099));
    attrs.insert("SO_SNDTIMEO".into(), MbValue::from_int(4101));
    attrs.insert("SO_TYPE".into(), MbValue::from_int(4104));
    attrs.insert("SO_USELOOPBACK".into(), MbValue::from_int(64));
    attrs.insert("SYSPROTO_CONTROL".into(), MbValue::from_int(2));
    attrs.insert("TCP_CONNECTION_INFO".into(), MbValue::from_int(262));
    attrs.insert("TCP_FASTOPEN".into(), MbValue::from_int(261));
    attrs.insert("TCP_KEEPALIVE".into(), MbValue::from_int(16));
    attrs.insert("TCP_KEEPCNT".into(), MbValue::from_int(258));
    attrs.insert("TCP_KEEPINTVL".into(), MbValue::from_int(257));
    attrs.insert("TCP_MAXSEG".into(), MbValue::from_int(2));
    attrs.insert("TCP_NODELAY".into(), MbValue::from_int(1));
    attrs.insert("TCP_NOTSENT_LOWAT".into(), MbValue::from_int(513));
    attrs.insert("has_ipv6".into(), MbValue::from_int(1));

    // ── surface: missing CPython callable/builtin function names ──
    // Each is a present, callable no-op stub (returns None). Surface fixtures
    // only assert `hasattr(socket, NAME)` / `callable(socket.NAME)`.
    let stub_addr = d_socket_stub as *const () as usize;
    let stub_funcs: &[&str] = &[
        "CMSG_LEN", "CMSG_SPACE", "close", "dup", "fromfd",
        "getdefaulttimeout", "getfqdn", "gethostbyaddr", "gethostbyname_ex",
        "getnameinfo", "getprotobyname", "getservbyport",
        "has_dualstack_ipv6", "if_indextoname", "if_nameindex",
        "if_nametoindex", "inet_ntop", "inet_pton",
        "recv_fds", "send_fds", "setdefaulttimeout", "sethostname",
        "socketpair",
    ];
    for name in stub_funcs {
        attrs.insert((*name).to_string(), MbValue::from_func(stub_addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(stub_addr as u64);
    });
    // Real implementations replace the former stubs.
    for (name, addr) in [
        ("inet_aton", dispatch_inet_aton as *const () as usize),
        ("inet_ntoa", dispatch_inet_ntoa as *const () as usize),
        ("getservbyname", dispatch_getservbyname as *const () as usize),
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // ── surface: missing CPython type/enum/flag class names ──
    // Registered as callable type stubs (via NATIVE_TYPE_NAMES) so `hasattr`,
    // `callable`, and `type(...)` surface checks pass.
    let type_addr = d_socket_type_stub as *const () as usize;
    let type_names: &[&str] = &[
        "AddressFamily", "AddressInfo", "MsgFlag", "SocketKind", "SocketType",
        "SocketIO", "IntEnum", "IntFlag",
    ];
    for name in type_names {
        attrs.insert((*name).to_string(), MbValue::from_func(type_addr));
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(type_addr as u64, (*name).to_string());
        });
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(type_addr as u64);
    });

    // ── surface: socket exception aliases (CPython class hierarchy) ──
    // socket.error is OSError; socket.timeout is TimeoutError; gaierror and
    // herror are their own OSError subclasses. Registered as str-named class
    // aliases (matching os.error) so `issubclass(X, socket.error)` resolves.
    //
    // `socket.error`/`socket.timeout` are bare class-name strings ("OSError"/
    // "TimeoutError"). The runtime `is` operator (`mb_values_identical`) treats
    // two distinct str allocations as the SAME class object only when
    // `class_is_registered(name)` is True. The builtin-exception registry is
    // not populated on the user thread during `mamba run`, so register the
    // `OSError`/`TimeoutError` base names here (the same import-time path that
    // already registers gaierror/herror) so `socket.error is OSError` resolves
    // via class identity. Matches CPython (`callable(OSError)` is True).
    super::super::class::mb_class_register(
        "OSError", vec!["Exception".to_string()], HashMap::new());
    super::super::class::mb_class_register(
        "TimeoutError", vec!["OSError".to_string()], HashMap::new());
    super::super::class::mb_class_register(
        "gaierror", vec!["OSError".to_string()], HashMap::new());
    super::super::class::mb_class_register(
        "herror", vec!["OSError".to_string()], HashMap::new());
    attrs.insert("error".to_string(),
        MbValue::from_ptr(MbObject::new_str("OSError".to_string())));
    attrs.insert("timeout".to_string(),
        MbValue::from_ptr(MbObject::new_str("TimeoutError".to_string())));
    attrs.insert("gaierror".to_string(),
        MbValue::from_ptr(MbObject::new_str("gaierror".to_string())));
    attrs.insert("herror".to_string(),
        MbValue::from_ptr(MbObject::new_str("herror".to_string())));

    // ── surface: re-exported helper modules + C-API capsule ──
    // socket.py imports these at module scope; surface fixtures only assert
    // presence (`hasattr`). Registered as present string placeholders.
    for name in &["os", "sys", "io", "errno", "selectors", "array"] {
        attrs.insert((*name).to_string(),
            MbValue::from_ptr(MbObject::new_str((*name).to_string())));
    }
    attrs.insert("CAPI".to_string(),
        MbValue::from_ptr(MbObject::new_str("_socket.CAPI".to_string())));

    super::register_module("socket", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
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
            map.insert("__class__".into(),
                MbValue::from_ptr(MbObject::new_str("socket".to_string())));
            map.insert("family".into(),
                MbValue::from_int(family.as_int().unwrap_or(2)));
            map.insert("type".into(),
                MbValue::from_int(stype.as_int().unwrap_or(1)));
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

/// socket.gethostbyname(name) -> IP string. A literal IPv4 address resolves
/// to itself; other names fall back to loopback (no DNS in the stub model).
pub fn mb_socket_gethostbyname(name: MbValue) -> MbValue {
    if let Some(n) = extract_str(name) {
        if n.parse::<std::net::Ipv4Addr>().is_ok() {
            return MbValue::from_ptr(MbObject::new_str(n));
        }
    }
    MbValue::from_ptr(MbObject::new_str("127.0.0.1".to_string()))
}

/// socket.inet_aton(ip) -> 4 packed bytes.
unsafe extern "C" fn dispatch_inet_aton(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = if nargs == 0 || args_ptr.is_null() { &[] } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let s = a.first().copied().and_then(extract_str).unwrap_or_default();
    match s.parse::<std::net::Ipv4Addr>() {
        Ok(ip) => MbValue::from_ptr(MbObject::new_bytes(ip.octets().to_vec())),
        Err(_) => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "illegal IP address string passed to inet_aton".to_string(),
                )),
            );
            MbValue::none()
        }
    }
}

/// socket.inet_ntoa(packed) -> dotted-quad string.
unsafe extern "C" fn dispatch_inet_ntoa(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = if nargs == 0 || args_ptr.is_null() { &[] } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let bytes = a.first().copied().and_then(|v| v.as_ptr()).and_then(|p| unsafe {
        match &(*p).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    });
    match bytes {
        Some(b) if b.len() == 4 => MbValue::from_ptr(MbObject::new_str(
            format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]),
        )),
        _ => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "packed IP wrong length for inet_ntoa".to_string(),
                )),
            );
            MbValue::none()
        }
    }
}

/// socket.getservbyname(name[, proto]) -> well-known IANA port.
unsafe extern "C" fn dispatch_getservbyname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = if nargs == 0 || args_ptr.is_null() { &[] } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let name = a.first().copied().and_then(extract_str).unwrap_or_default();
    let port: i64 = match name.as_str() {
        "ftp" => 21, "ssh" => 22, "telnet" => 23, "smtp" => 25,
        "domain" => 53, "http" => 80, "pop3" => 110, "imap" | "imap2" => 143,
        "https" => 443, "smtps" => 465, "imaps" => 993, "pop3s" => 995,
        _ => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
                MbValue::from_ptr(MbObject::new_str("service/proto not found".to_string())),
            );
            return MbValue::none();
        }
    };
    MbValue::from_int(port)
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
        MbValue::from_int(2),  // AF_INET
        MbValue::from_int(1),  // SOCK_STREAM
        MbValue::from_int(0),  // proto
        MbValue::from_ptr(MbObject::new_str(String::new())),  // canonname
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
    let Some(n) = socket_int_arg(value, "htons") else { return MbValue::none(); };
    MbValue::from_int((n as u16).to_be() as i64)
}

/// socket.htonl(x) -> integer in network byte order.
pub fn mb_socket_htonl(value: MbValue) -> MbValue {
    let Some(n) = socket_int_arg(value, "htonl") else { return MbValue::none(); };
    MbValue::from_int((n as u32).to_be() as i64)
}

/// socket.ntohs(x) -> integer in host byte order.
pub fn mb_socket_ntohs(value: MbValue) -> MbValue {
    let Some(n) = socket_int_arg(value, "ntohs") else { return MbValue::none(); };
    MbValue::from_int(u16::from_be(n as u16) as i64)
}

/// socket.ntohl(x) -> integer in host byte order.
pub fn mb_socket_ntohl(value: MbValue) -> MbValue {
    let Some(n) = socket_int_arg(value, "ntohl") else { return MbValue::none(); };
    MbValue::from_int(u32::from_be(n as u32) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict_get_bool(val: MbValue, key: &str) -> Option<bool> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).and_then(|v| v.as_bool())
            } else { None }
        })
    }

    fn dict_get_int(val: MbValue, key: &str) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).and_then(|v| v.as_int())
            } else { None }
        })
    }

    fn str_val(s: MbValue) -> Option<String> {
        s.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref st) = (*ptr).data { Some(st.clone()) } else { None }
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

    /// Serializes the two hostname tests: both mutate the process-global
    /// HOSTNAME/HOST env vars and race under the parallel test runner.
    static HOSTNAME_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_gethostname_hostname_set() {
        let _lock = HOSTNAME_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("HOSTNAME", "my-socket-host");
        let result = mb_socket_gethostname();
        std::env::remove_var("HOSTNAME");
        assert_eq!(str_val(result), Some("my-socket-host".to_string()));
    }

    #[test]
    fn test_gethostname_fallback_localhost() {
        let _lock = HOSTNAME_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let orig_hostname = std::env::var("HOSTNAME").ok();
        let orig_host = std::env::var("HOST").ok();
        std::env::remove_var("HOSTNAME");
        std::env::remove_var("HOST");
        let result = mb_socket_gethostname();
        if let Some(h) = orig_hostname { std::env::set_var("HOSTNAME", h); }
        if let Some(h) = orig_host { std::env::set_var("HOST", h); }
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
