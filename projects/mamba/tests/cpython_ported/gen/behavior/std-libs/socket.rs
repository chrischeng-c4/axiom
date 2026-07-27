use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/socket/connect_refused_to_unused_port_raises.py`.
#[test]
fn test_gen_behavior_std_libs_socket_connect_refused_to_unused_port_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "connect_refused_to_unused_port_raises"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: connecting (with a short timeout) to loopback port 1 raises a ConnectionRefusedError / timeout / OSError"""
import socket

_raised = False
try:
    _s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    _s.settimeout(1.0)
    _s.connect(("127.0.0.1", 1))  # port 1 is privileged/unused
    _s.close()
except (ConnectionRefusedError, socket.timeout, OSError):
    _raised = True
assert _raised, "refused connection raises"
print("connect_refused_to_unused_port_raises OK")
"###);
    assert_output(&out, r###"connect_refused_to_unused_port_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/create_connection_all_errors_exceptiongroup.py`.
#[test]
fn test_gen_behavior_std_libs_socket_create_connection_all_errors_exceptiongroup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "create_connection_all_errors_exceptiongroup"
# subject = "socket.create_connection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.create_connection: create_connection(..., all_errors=True) to a closed port raises an ExceptionGroup of one OSError per resolved address"""
import socket

probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
probe.bind(("127.0.0.1", 0))
closed_port = probe.getsockname()[1]
probe.close()
try:
    socket.create_connection(("localhost", closed_port), timeout=2, all_errors=True)
    raise AssertionError("connection to closed port should fail")
except ExceptionGroup as eg:
    assert all(isinstance(e, OSError) for e in eg.exceptions), "all sub-errors OSError"
    addrs = socket.getaddrinfo("localhost", closed_port, 0, socket.SOCK_STREAM)
    assert len(addrs) == len(eg.exceptions), "one error per resolved address"
print("create_connection_all_errors_exceptiongroup OK")
"###);
    assert_output(&out, r###"create_connection_all_errors_exceptiongroup OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/create_server_binds_loopback_listener.py`.
#[test]
fn test_gen_behavior_std_libs_socket_create_server_binds_loopback_listener() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "create_server_binds_loopback_listener"
# subject = "socket.create_server"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.create_server: create_server(('127.0.0.1', 0)) returns an AF_INET SOCK_STREAM socket bound to 127.0.0.1 on a positive ephemeral port; reuse_port toggles SO_REUSEPORT"""
import socket

with socket.create_server(("127.0.0.1", 0)) as sock:
    assert sock.family == socket.AF_INET, f"family = {sock.family!r}"
    assert sock.type == socket.SOCK_STREAM, f"type = {sock.type!r}"
    host, port = sock.getsockname()
    assert host == "127.0.0.1", f"host = {host!r}"
    assert isinstance(port, int) and port > 0, f"port = {port!r}"

if hasattr(socket, "SO_REUSEPORT"):
    with socket.create_server(("127.0.0.1", 0)) as sock:
        assert sock.getsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT) == 0, "default off"
    with socket.create_server(("127.0.0.1", 0), reuse_port=True) as sock:
        assert sock.getsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT) != 0, "reuse_port on"
else:
    raised = False
    try:
        socket.create_server(("127.0.0.1", 0), reuse_port=True)
    except ValueError:
        raised = True
    assert raised, "reuse_port without support should raise ValueError"
print("create_server_binds_loopback_listener OK")
"###);
    assert_output(&out, r###"create_server_binds_loopback_listener OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/getaddrinfo_parses_numeric_ipv6_and_scope.py`.
#[test]
fn test_gen_behavior_std_libs_socket_getaddrinfo_parses_numeric_ipv6_and_scope() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getaddrinfo_parses_numeric_ipv6_and_scope"
# subject = "socket.getaddrinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getaddrinfo: getaddrinfo lower-cases a numeric IPv6 literal and threads a %scope id into the sockaddr's scope-id field"""
import socket

(*_, sockaddr), = socket.getaddrinfo(
    "ff02::1de:c0:face:8D", 1234,
    socket.AF_INET6, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
assert sockaddr == ("ff02::1de:c0:face:8d", 1234, 0, 0), sockaddr

(*_, scoped), = socket.getaddrinfo(
    "ff02::1de:c0:face:8D%42", 1234,
    socket.AF_INET6, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
assert scoped == ("ff02::1de:c0:face:8d", 1234, 0, 42), scoped
print("getaddrinfo_parses_numeric_ipv6_and_scope OK")
"###);
    assert_output(&out, r###"getaddrinfo_parses_numeric_ipv6_and_scope OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/gethostbyname_literal_ipv4_is_identity.py`.
#[test]
fn test_gen_behavior_std_libs_socket_gethostbyname_literal_ipv4_is_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "gethostbyname_literal_ipv4_is_identity"
# subject = "socket.gethostbyname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.gethostbyname: a literal IPv4 address resolves to itself with no DNS round-trip (127.0.0.1, 10.0.0.1, 255.255.255.255)"""
import socket

for _addr in ("127.0.0.1", "10.0.0.1", "255.255.255.255"):
    assert socket.gethostbyname(_addr) == _addr, f"{_addr} should resolve to itself"
print("gethostbyname_literal_ipv4_is_identity OK")
"###);
    assert_output(&out, r###"gethostbyname_literal_ipv4_is_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/getnameinfo_numeric_flags_reverse.py`.
#[test]
fn test_gen_behavior_std_libs_socket_getnameinfo_numeric_flags_reverse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getnameinfo_numeric_flags_reverse"
# subject = "socket.getnameinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getnameinfo: getnameinfo with NI_NUMERICHOST|NI_NUMERICSERV reverses a numeric IPv6 address/port without DNS, lower-casing the host"""
import socket

ni = socket.getnameinfo(
    ("ff02::1de:c0:face:8D", 1234, 0, 0),
    socket.NI_NUMERICHOST | socket.NI_NUMERICSERV)
assert ni == ("ff02::1de:c0:face:8d", "1234"), ni
print("getnameinfo_numeric_flags_reverse OK")
"###);
    assert_output(&out, r###"getnameinfo_numeric_flags_reverse OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/getservbyname_well_known_ports.py`.
#[test]
fn test_gen_behavior_std_libs_socket_getservbyname_well_known_ports() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getservbyname_well_known_ports"
# subject = "socket.getservbyname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getservbyname: getservbyname maps well-known service names to their IANA ports: http=80, https=443, ftp=21, ssh=22"""
import socket

assert socket.getservbyname("http") == 80, "http = 80"
assert socket.getservbyname("https") == 443, "https = 443"
assert socket.getservbyname("ftp") == 21, "ftp = 21"
assert socket.getservbyname("ssh") == 22, "ssh = 22"
print("getservbyname_well_known_ports OK")
"###);
    assert_output(&out, r###"getservbyname_well_known_ports OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/getsockname_returns_bound_address.py`.
#[test]
fn test_gen_behavior_std_libs_socket_getsockname_returns_bound_address() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getsockname_returns_bound_address"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: after bind(('127.0.0.1', 0)) getsockname() reports host '127.0.0.1' and a positive integer ephemeral port"""
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_s.bind(("127.0.0.1", 0))
_name = _s.getsockname()
assert _name[0] == "127.0.0.1", f"getsockname host = {_name[0]!r}"
assert isinstance(_name[1], int), f"getsockname port = {_name[1]!r}"
assert _name[1] > 0, "port is positive"
_s.close()
print("getsockname_returns_bound_address OK")
"###);
    assert_output(&out, r###"getsockname_returns_bound_address OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/inet_aton_ntoa_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_socket_inet_aton_ntoa_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "inet_aton_ntoa_roundtrip"
# subject = "socket.inet_aton"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.inet_aton: inet_ntoa(inet_aton(ip)) is the identity for several dotted-quad IPv4 addresses including 255.255.255.255"""
import socket

for _ip in ("192.168.0.1", "10.0.0.1", "172.16.0.1", "255.255.255.255"):
    assert socket.inet_ntoa(socket.inet_aton(_ip)) == _ip, f"round-trip {_ip}"
print("inet_aton_ntoa_roundtrip OK")
"###);
    assert_output(&out, r###"inet_aton_ntoa_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/inheritable_flag_toggles_and_dup.py`.
#[test]
fn test_gen_behavior_std_libs_socket_inheritable_flag_toggles_and_dup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "inheritable_flag_toggles_and_dup"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: sockets are non-inheritable by default, set_inheritable toggles the flag both ways, and dup() yields an independent non-inheritable live descriptor"""
import socket

# Sockets are non-inheritable by default (FD_CLOEXEC set).
sock = socket.socket()
assert sock.get_inheritable() is False, "default should be non-inheritable"

# set_inheritable toggles the flag both ways.
sock.set_inheritable(True)
assert sock.get_inheritable() is True, "after set_inheritable(True)"
sock.set_inheritable(False)
assert sock.get_inheritable() is False, "after set_inheritable(False)"

# A duplicated socket is its own non-inheritable descriptor and survives
# the original being closed.
dup = sock.dup()
sock.close()
assert dup.get_inheritable() is False, "dup() result should be non-inheritable"
assert dup.fileno() >= 0, "dup() should have a live descriptor"
dup.close()
print("inheritable_flag_toggles_and_dup OK")
"###);
    assert_output(&out, r###"inheritable_flag_toggles_and_dup OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/live_socket_not_picklable_enums_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_socket_live_socket_not_picklable_enums_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "live_socket_not_picklable_enums_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a live socket object is unpicklable (TypeError) at every protocol, while the AddressFamily/SocketKind IntEnum constants round-trip through pickle preserving value"""
import pickle
import socket

# A live socket object cannot be pickled (TypeError at every protocol).
sock = socket.socket()
with sock:
    for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
        raised = False
        try:
            pickle.dumps(sock, protocol)
        except TypeError:
            raised = True
        assert raised, f"socket pickling should fail at protocol {protocol}"

# The IntEnum constants round-trip through pickle, preserving value/identity.
for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
    family = pickle.loads(pickle.dumps(socket.AF_INET, protocol))
    assert family == socket.AF_INET, f"AF_INET round-trip at {protocol}: {family!r}"
    kind = pickle.loads(pickle.dumps(socket.SOCK_STREAM, protocol))
    assert kind == socket.SOCK_STREAM, f"SOCK_STREAM round-trip at {protocol}: {kind!r}"
print("live_socket_not_picklable_enums_roundtrip OK")
"###);
    assert_output(&out, r###"live_socket_not_picklable_enums_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/makefile_returns_readable_file.py`.
#[test]
fn test_gen_behavior_std_libs_socket_makefile_returns_readable_file() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "makefile_returns_readable_file"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: makefile('rb', buffering=0) is readable but not writable/seekable, raises ValueError on capability queries after close, and a closed socket-file reprs with name=-1"""
import socket

# An unbuffered read file is readable but not writable or seekable.
with socket.socket() as sock:
    fp = sock.makefile("rb", buffering=0)
    assert fp.readable(), "rb file should be readable"
    assert not fp.writable(), "rb file should not be writable"
    assert not fp.seekable(), "socket file should not be seekable"

    # Once closed, the capability queries raise ValueError.
    fp.close()
    for method in ("readable", "writable", "seekable"):
        raised = False
        try:
            getattr(fp, method)()
        except ValueError:
            raised = True
        assert raised, f"{method}() on closed file should raise ValueError"

# A closed socket-file reports a -1 file descriptor in its repr.
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    fp = sock.makefile("rb")
    fp.close()
    assert repr(fp) == "<_io.BufferedReader name=-1>", repr(fp)
print("makefile_returns_readable_file OK")
"###);
    assert_output(&out, r###"makefile_returns_readable_file OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/nonblocking_accept_raises_blockingio.py`.
#[test]
fn test_gen_behavior_std_libs_socket_nonblocking_accept_raises_blockingio() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "nonblocking_accept_raises_blockingio"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a listening socket put in setblocking(False) with no pending connection raises BlockingIOError (a socket.error) from accept()"""
import socket

_nb = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_nb.bind(("127.0.0.1", 0))
_nb.listen(1)
_nb.setblocking(False)
_raised = False
try:
    _nb.accept()
except (BlockingIOError, socket.error):
    _raised = True
_nb.close()
assert _raised, "non-blocking accept raises BlockingIOError"
print("nonblocking_accept_raises_blockingio OK")
"###);
    assert_output(&out, r###"nonblocking_accept_raises_blockingio OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/repr_reports_fd_family_type_and_state.py`.
#[test]
fn test_gen_behavior_std_libs_socket_repr_reports_fd_family_type_and_state() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "repr_reports_fd_family_type_and_state"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: repr() of a socket shows fd/family/type/proto, gains laddr after bind, and shows [closed] after close; the family/type IntEnum members carry rich repr and numeric str"""
import socket

# An open, unbound socket reports fd/family/type/proto and no remote address.
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
r = repr(s)
assert ("fd=%i" % s.fileno()) in r, f"fd missing: {r}"
assert ("family=%s" % socket.AF_INET) in r, f"family missing: {r}"
assert ("type=%s" % socket.SOCK_STREAM) in r, f"type missing: {r}"
assert "proto=0" in r, f"proto missing: {r}"
assert "raddr" not in r, f"unexpected raddr: {r}"

# After binding, the local address shows up in the repr.
s.bind(("127.0.0.1", 0))
r = repr(s)
assert "laddr" in r, f"laddr missing after bind: {r}"
assert str(s.getsockname()) in r, f"sockname missing: {r}"

# A closed socket reports [closed] and drops the local address.
s.close()
r = repr(s)
assert "[closed]" in r, f"closed marker missing: {r}"
assert "laddr" not in r, f"laddr leaked after close: {r}"

# socket.family / socket.type are IntEnum members with rich repr.
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s2:
    assert repr(s2.family) == "<AddressFamily.AF_INET: %r>" % s2.family.value, repr(s2.family)
    assert repr(s2.type) == "<SocketKind.SOCK_STREAM: %r>" % s2.type.value, repr(s2.type)
    # str() of an IntEnum member is its numeric value (Python 3.11+ behavior).
    assert str(s2.family) == str(s2.family.value), str(s2.family)
    assert str(s2.type) == str(s2.type.value), str(s2.type)
print("repr_reports_fd_family_type_and_state OK")
"###);
    assert_output(&out, r###"repr_reports_fd_family_type_and_state OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/so_reuseaddr_setsockopt_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_socket_so_reuseaddr_setsockopt_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "so_reuseaddr_setsockopt_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: setsockopt(SOL_SOCKET, SO_REUSEADDR, 1) is observable via getsockopt as a nonzero value"""
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
_val = _s.getsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR)
assert _val != 0, f"SO_REUSEADDR set: {_val!r}"
_s.close()
print("so_reuseaddr_setsockopt_roundtrip OK")
"###);
    assert_output(&out, r###"so_reuseaddr_setsockopt_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/tcp_pair_send_recv_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_socket_tcp_pair_send_recv_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "tcp_pair_send_recv_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a loopback TCP socket pair (server thread + client) exchanges ping/pong: the server receives b'ping' and the client receives the b'pong' reply"""
import socket
import threading

_srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
_srv.bind(("127.0.0.1", 0))
_port = _srv.getsockname()[1]
_srv.listen(1)

_received = []


def _server_thread():
    _conn, _ = _srv.accept()
    _data = _conn.recv(1024)
    _received.append(_data)
    _conn.sendall(b"pong")
    _conn.close()


_t = threading.Thread(target=_server_thread)
_t.start()

_cli = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_cli.connect(("127.0.0.1", _port))
_cli.sendall(b"ping")
_response = _cli.recv(1024)
_cli.close()
_t.join()
_srv.close()

assert _received == [b"ping"], f"server received: {_received!r}"
assert _response == b"pong", f"client received: {_response!r}"
print("tcp_pair_send_recv_roundtrip OK")
"###);
    assert_output(&out, r###"tcp_pair_send_recv_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/udp_sendto_recvfrom_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_socket_udp_sendto_recvfrom_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "udp_sendto_recvfrom_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a connectionless UDP socket pair on loopback delivers a datagram: sendto(b'hello udp') is read back verbatim by recvfrom on the bound server"""
import socket

_udp_srv = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
_udp_srv.bind(("127.0.0.1", 0))
_udp_port = _udp_srv.getsockname()[1]
_udp_srv.settimeout(2.0)

_udp_cli = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
_udp_cli.sendto(b"hello udp", ("127.0.0.1", _udp_port))
_udp_data, _udp_addr = _udp_srv.recvfrom(1024)
assert _udp_data == b"hello udp", f"UDP data: {_udp_data!r}"
_udp_srv.close()
_udp_cli.close()
print("udp_sendto_recvfrom_roundtrip OK")
"###);
    assert_output(&out, r###"udp_sendto_recvfrom_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/socket/weakref_proxy_dies_after_collection.py`.
#[test]
fn test_gen_behavior_std_libs_socket_weakref_proxy_dies_after_collection() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "weakref_proxy_dies_after_collection"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a weakref.proxy mirrors a live socket's fileno but raises ReferenceError once the socket is dropped and gc-collected"""
import gc
import socket
from weakref import proxy

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    p = proxy(s)
    assert p.fileno() == s.fileno(), "proxy should mirror the live socket"
s = None
gc.collect()
dead = False
try:
    p.fileno()
except ReferenceError:
    dead = True
assert dead, "proxy should raise ReferenceError after collection"
print("weakref_proxy_dies_after_collection OK")
"###);
    assert_output(&out, r###"weakref_proxy_dies_after_collection OK
"###);
}
