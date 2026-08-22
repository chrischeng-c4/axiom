use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/socket/bad_address_family_raises.py`.
#[test]
fn test_gen_errors_std_libs_socket_bad_address_family_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "bad_address_family_raises"
# subject = "socket.socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: bad_address_family_raises (errors)."""
import socket

_raised = False
try:
    socket.socket(99999, socket.SOCK_STREAM)
except OSError:
    _raised = True
assert _raised, "bad_address_family_raises: expected OSError"
print("bad_address_family_raises OK")
"###);
    assert_output(&out, r###"bad_address_family_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/close_bad_fd_oserror.py`.
#[test]
fn test_gen_errors_std_libs_socket_close_bad_fd_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "close_bad_fd_oserror"
# subject = "socket.close"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.close: close_bad_fd_oserror (errors)."""
import socket

_raised = False
try:
    socket.close(-1)
except OSError:
    _raised = True
assert _raised, "close_bad_fd_oserror: expected OSError"
print("close_bad_fd_oserror OK")
"###);
    assert_output(&out, r###"close_bad_fd_oserror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/fileno_negative_message_echoes_descriptor.py`.
#[test]
fn test_gen_errors_std_libs_socket_fileno_negative_message_echoes_descriptor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "fileno_negative_message_echoes_descriptor"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: socket(fileno=-1) / fileno=-42 raises ValueError whose message names 'negative file descriptor'"""
import socket

for bad in (-1, -42):
    raised = False
    try:
        socket.socket(socket.AF_INET, socket.SOCK_STREAM, fileno=bad)
    except ValueError as e:
        raised = True
        assert "negative file descriptor" in str(e), str(e)
    assert raised, f"fileno={bad} should raise ValueError"
print("fileno_negative_message_echoes_descriptor OK")
"###);
    assert_output(&out, r###"fileno_negative_message_echoes_descriptor OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/fileno_negative_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_socket_fileno_negative_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "fileno_negative_valueerror"
# subject = "socket.socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: fileno_negative_valueerror (errors)."""
import socket

_raised = False
try:
    socket.socket(socket.AF_INET, socket.SOCK_STREAM, fileno=-1)
except ValueError:
    _raised = True
assert _raised, "fileno_negative_valueerror: expected ValueError"
print("fileno_negative_valueerror OK")
"###);
    assert_output(&out, r###"fileno_negative_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/getaddrinfo_out_of_range_port_is_gaierror.py`.
#[test]
fn test_gen_errors_std_libs_socket_getaddrinfo_out_of_range_port_is_gaierror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "getaddrinfo_out_of_range_port_is_gaierror"
# subject = "socket.getaddrinfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getaddrinfo: an out-of-range port (2**64, 2**63, -(2**63)-1) surfaces as socket.gaierror, never OverflowError; boundary ports 0 and 65535 are accepted"""
import socket

for bad_port in (2**64, 2**63, -(2**63) - 1):
    raised = False
    try:
        socket.getaddrinfo(None, bad_port, type=socket.SOCK_STREAM)
    except socket.gaierror:
        raised = True
    except OverflowError:
        raise AssertionError(f"port {bad_port}: got OverflowError, expected gaierror")
    assert raised, f"port {bad_port} should raise gaierror"

# Boundary ports are accepted.
socket.getaddrinfo(None, 0, type=socket.SOCK_STREAM)
socket.getaddrinfo(None, 65535, type=socket.SOCK_STREAM)
print("getaddrinfo_out_of_range_port_is_gaierror OK")
"###);
    assert_output(&out, r###"getaddrinfo_out_of_range_port_is_gaierror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/getaddrinfo_unresolvable_host_raises.py`.
#[test]
fn test_gen_errors_std_libs_socket_getaddrinfo_unresolvable_host_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "getaddrinfo_unresolvable_host_raises"
# subject = "socket.getaddrinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getaddrinfo: getaddrinfo_unresolvable_host_raises (errors)."""
import socket

_raised = False
try:
    socket.getaddrinfo("definitely_not_a_real_host_xyzzy.invalid", 80)
except socket.gaierror:
    _raised = True
assert _raised, "getaddrinfo_unresolvable_host_raises: expected socket.gaierror"
print("getaddrinfo_unresolvable_host_raises OK")
"###);
    assert_output(&out, r###"getaddrinfo_unresolvable_host_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/inet_aton_bad_ip_raises.py`.
#[test]
fn test_gen_errors_std_libs_socket_inet_aton_bad_ip_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "inet_aton_bad_ip_raises"
# subject = "socket.inet_aton"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.inet_aton: inet_aton_bad_ip_raises (errors)."""
import socket

_raised = False
try:
    socket.inet_aton("not.an.ip.address")
except OSError:
    _raised = True
assert _raised, "inet_aton_bad_ip_raises: expected OSError"
print("inet_aton_bad_ip_raises OK")
"###);
    assert_output(&out, r###"inet_aton_bad_ip_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/recv_on_closed_socket_raises.py`.
#[test]
fn test_gen_errors_std_libs_socket_recv_on_closed_socket_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "recv_on_closed_socket_raises"
# subject = "socket.socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: recv_on_closed_socket_raises (errors)."""
import socket

_raised = False
try:
    (lambda s: (s.close(), s.recv(1024)))(socket.socket())
except OSError:
    _raised = True
assert _raised, "recv_on_closed_socket_raises: expected OSError"
print("recv_on_closed_socket_raises OK")
"###);
    assert_output(&out, r###"recv_on_closed_socket_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/socket/timeout_is_timeouterror_alias.py`.
#[test]
fn test_gen_errors_std_libs_socket_timeout_is_timeouterror_alias() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "timeout_is_timeouterror_alias"
# subject = "socket.timeout"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.timeout: socket.timeout is the builtin TimeoutError alias (Python 3.10+)"""
import socket

assert socket.timeout is TimeoutError, f"socket.timeout = {socket.timeout!r}"
print("timeout_is_timeouterror_alias OK")
"###);
    assert_output(&out, r###"timeout_is_timeouterror_alias OK
"###);
}
