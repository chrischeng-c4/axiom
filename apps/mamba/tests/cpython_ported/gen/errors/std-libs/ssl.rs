use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/_ssl/certificate_no_public_constructor.py`.
#[test]
fn test_gen_errors_std_libs__ssl_certificate_no_public_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "errors"
# case = "certificate_no_public_constructor"
# subject = "_ssl.Certificate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "CPython 3.12 _ssl module"
# status = "filled"
# ///
"""_ssl.Certificate: public construction is rejected with CPython safety errors."""
from _ssl import Certificate

try:
    Certificate()
    raise AssertionError("Certificate() should raise")
except TypeError as _e:
    assert "cannot create '_ssl.Certificate' instances" in str(_e), str(_e)

try:
    object.__new__(Certificate)
    raise AssertionError("object.__new__(Certificate) should raise")
except TypeError as _e:
    assert "object.__new__(_ssl.Certificate) is not safe" in str(_e), str(_e)

print("certificate_no_public_constructor OK")
"###);
    assert_output(&out, r###"certificate_no_public_constructor OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/bad_cipher_string_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_bad_cipher_string_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "bad_cipher_string_raises"
# subject = "ssl.SSLContext.set_ciphers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.set_ciphers: bad_cipher_string_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().set_ciphers('no_such_cipher_suite_xyzzy')
except ssl.SSLError:
    _raised = True
assert _raised, "bad_cipher_string_raises: expected ssl.SSLError"
print("bad_cipher_string_raises OK")
"###);
    assert_output(&out, r###"bad_cipher_string_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/bad_protocol_number_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_bad_protocol_number_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "bad_protocol_number_raises"
# subject = "ssl.SSLContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext: bad_protocol_number_raises (errors)."""
import ssl

_raised = False
try:
    ssl.SSLContext(42)
except ValueError:
    _raised = True
assert _raised, "bad_protocol_number_raises: expected ValueError"
print("bad_protocol_number_raises OK")
"###);
    assert_output(&out, r###"bad_protocol_number_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/check_hostname_forbids_cert_none.py`.
#[test]
fn test_gen_errors_std_libs_ssl_check_hostname_forbids_cert_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "check_hostname_forbids_cert_none"
# subject = "ssl.SSLContext.verify_mode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_mode: with check_hostname enabled, assigning verify_mode = CERT_NONE raises ValueError (hostname checking requires a verifying mode)"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_OPTIONAL
_ctx.check_hostname = True
assert _ctx.verify_mode == ssl.CERT_OPTIONAL, "OPTIONAL kept under check_hostname"
try:
    _ctx.verify_mode = ssl.CERT_NONE
    raise AssertionError("CERT_NONE under check_hostname should raise")
except ValueError:
    pass

print("check_hostname_forbids_cert_none OK")
"###);
    assert_output(&out, r###"check_hostname_forbids_cert_none OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/load_cert_chain_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_load_cert_chain_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "load_cert_chain_missing_file_raises"
# subject = "ssl.SSLContext.load_cert_chain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.load_cert_chain: load_cert_chain_missing_file_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().load_cert_chain('/no/such/cert.pem')
except FileNotFoundError:
    _raised = True
assert _raised, "load_cert_chain_missing_file_raises: expected FileNotFoundError"
print("load_cert_chain_missing_file_raises OK")
"###);
    assert_output(&out, r###"load_cert_chain_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/load_verify_locations_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_load_verify_locations_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "load_verify_locations_missing_file_raises"
# subject = "ssl.SSLContext.load_verify_locations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.load_verify_locations: load_verify_locations_missing_file_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().load_verify_locations('/no/such/ca.pem')
except FileNotFoundError:
    _raised = True
assert _raised, "load_verify_locations_missing_file_raises: expected FileNotFoundError"
print("load_verify_locations_missing_file_raises OK")
"###);
    assert_output(&out, r###"load_verify_locations_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/memory_bio_write_rejects_non_buffer.py`.
#[test]
fn test_gen_errors_std_libs_ssl_memory_bio_write_rejects_non_buffer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "memory_bio_write_rejects_non_buffer"
# subject = "ssl.MemoryBIO.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.write: MemoryBIO.write rejects non-bytes-like inputs (str, None, bool, int) with TypeError and a non-contiguous writable memoryview with BufferError"""
import ssl

_t = ssl.MemoryBIO()
for _bad in ("foo", None, True, 1):
    try:
        _t.write(_bad)
        raise AssertionError(f"write({_bad!r}) should raise")
    except TypeError:
        pass

# Non-contiguous writable memoryview is rejected with BufferError.
_m = memoryview(bytearray(b"noncontig"))[::-2]
try:
    _t.write(memoryview(_m))
    raise AssertionError("non-contiguous buffer should raise")
except BufferError:
    pass

print("memory_bio_write_rejects_non_buffer OK")
"###);
    assert_output(&out, r###"memory_bio_write_rejects_non_buffer OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/minimum_version_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_minimum_version_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "minimum_version_int_raises"
# subject = "ssl.SSLContext.minimum_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.minimum_version: minimum_version_int_raises (errors)."""
import ssl

_raised = False
try:
    setattr(ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER), 'minimum_version', 42)
except ValueError:
    _raised = True
assert _raised, "minimum_version_int_raises: expected ValueError"
print("minimum_version_int_raises OK")
"###);
    assert_output(&out, r###"minimum_version_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/no_public_sslobject_constructor.py`.
#[test]
fn test_gen_errors_std_libs_ssl_no_public_sslobject_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "no_public_sslobject_constructor"
# subject = "ssl.SSLObject"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLObject: SSLObject has no public constructor: ssl.SSLObject(MemoryBIO(), MemoryBIO()) raises TypeError naming the missing public constructor; the type is built only via wrap_bio"""
import ssl

try:
    ssl.SSLObject(ssl.MemoryBIO(), ssl.MemoryBIO())
    raise AssertionError("SSLObject() should raise")
except TypeError as _e:
    assert "public constructor" in str(_e), f"SSLObject msg: {_e}"

print("no_public_sslobject_constructor OK")
"###);
    assert_output(&out, r###"no_public_sslobject_constructor OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/no_public_sslsocket_constructor.py`.
#[test]
fn test_gen_errors_std_libs_ssl_no_public_sslsocket_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "no_public_sslsocket_constructor"
# subject = "ssl.SSLSocket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLSocket: SSLSocket has no public constructor: ssl.SSLSocket(sock) raises TypeError naming the missing public constructor; the type is built only via wrap_socket"""
import ssl

import socket

try:
    with socket.socket() as _s:
        ssl.SSLSocket(_s)
    raise AssertionError("SSLSocket() should raise")
except TypeError as _e:
    assert "public constructor" in str(_e), f"SSLSocket msg: {_e}"

print("no_public_sslsocket_constructor OK")
"###);
    assert_output(&out, r###"no_public_sslsocket_constructor OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/num_tickets_invalid_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_num_tickets_invalid_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "num_tickets_invalid_raises"
# subject = "ssl.SSLContext.num_tickets"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.num_tickets: server num_tickets rejects a negative int with ValueError and None with TypeError, while a client context's num_tickets is read-only and raises ValueError on assignment"""
import ssl

_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
try:
    _srv.num_tickets = -1
    raise AssertionError("negative num_tickets should raise")
except ValueError:
    pass
try:
    _srv.num_tickets = None
    raise AssertionError("None num_tickets should raise")
except TypeError:
    pass

_clt = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
try:
    _clt.num_tickets = 1
    raise AssertionError("client num_tickets should be read-only")
except ValueError:
    pass

print("num_tickets_invalid_raises OK")
"###);
    assert_output(&out, r###"num_tickets_invalid_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/options_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_options_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "options_out_of_range_raises"
# subject = "ssl.SSLContext.options"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.options: the options bitmask rejects bad values: -1 and 2**100 raise OverflowError, a str raises TypeError"""
import ssl

_opt = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
assert isinstance(_opt.options, int), "options is int"
for _bad, _exc in ((-1, OverflowError), (2 ** 100, OverflowError), ("abc", TypeError)):
    try:
        _opt.options = _bad
        raise AssertionError(f"options={_bad!r} should raise")
    except (OverflowError, TypeError) as _e:
        assert isinstance(_e, _exc), f"options={_bad!r} -> {type(_e).__name__}"

print("options_out_of_range_raises OK")
"###);
    assert_output(&out, r###"options_out_of_range_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/rand_bytes_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_rand_bytes_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "rand_bytes_negative_raises"
# subject = "ssl.RAND_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.RAND_bytes: RAND_bytes rejects a negative byte count with ValueError regardless of PRNG seeding state"""
import ssl

try:
    ssl.RAND_bytes(-5)
    raise AssertionError("RAND_bytes(-5) should raise")
except ValueError:
    pass

print("rand_bytes_negative_raises OK")
"###);
    assert_output(&out, r###"rand_bytes_negative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/server_side_with_hostname_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_server_side_with_hostname_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "server_side_with_hostname_raises"
# subject = "ssl.SSLContext.wrap_socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_socket: a server-side wrap_socket given a server_hostname raises ValueError (hostname only makes sense client-side)"""
import ssl

import socket

_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
try:
    with socket.socket() as _sock:
        _srv.wrap_socket(_sock, True, server_hostname="some.hostname")
    raise AssertionError("server_side + server_hostname should raise")
except ValueError:
    pass

print("server_side_with_hostname_raises OK")
"###);
    assert_output(&out, r###"server_side_with_hostname_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/verify_mode_bad_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_verify_mode_bad_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "verify_mode_bad_int_raises"
# subject = "ssl.SSLContext.verify_mode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_mode: verify_mode_bad_int_raises (errors)."""
import ssl

_raised = False
try:
    setattr(ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER), 'verify_mode', 42)
except ValueError:
    _raised = True
assert _raised, "verify_mode_bad_int_raises: expected ValueError"
print("verify_mode_bad_int_raises OK")
"###);
    assert_output(&out, r###"verify_mode_bad_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/verify_mode_none_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_verify_mode_none_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "verify_mode_none_raises"
# subject = "ssl.SSLContext.verify_mode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_mode: verify_mode_none_raises (errors)."""
import ssl

_raised = False
try:
    setattr(ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER), 'verify_mode', None)
except TypeError:
    _raised = True
assert _raised, "verify_mode_none_raises: expected TypeError"
print("verify_mode_none_raises OK")
"###);
    assert_output(&out, r###"verify_mode_none_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/wrap_bio_invalid_server_hostname_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_wrap_bio_invalid_server_hostname_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "wrap_bio_invalid_server_hostname_raises"
# subject = "ssl.SSLContext.wrap_bio"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_bio: wrap_bio validates server_hostname: an empty string and a leading-dot name each raise ValueError, while an embedded NUL byte raises TypeError"""
import ssl

_ctx = ssl.create_default_context()

# Empty hostname -> ValueError.
try:
    _ctx.wrap_bio(ssl.MemoryBIO(), ssl.MemoryBIO(), server_hostname="")
    raise AssertionError("empty server_hostname should raise")
except ValueError:
    pass

# Leading-dot hostname -> ValueError (UnicodeError is a ValueError subclass).
try:
    _ctx.wrap_bio(ssl.MemoryBIO(), ssl.MemoryBIO(), server_hostname=".example.org")
    raise AssertionError("leading-dot hostname should raise")
except ValueError:
    pass

# Embedded NUL byte -> TypeError.
try:
    _ctx.wrap_bio(ssl.MemoryBIO(), ssl.MemoryBIO(),
                  server_hostname="example.org\x00evil.com")
    raise AssertionError("NUL in hostname should raise")
except TypeError:
    pass

print("wrap_bio_invalid_server_hostname_raises OK")
"###);
    assert_output(&out, r###"wrap_bio_invalid_server_hostname_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/ssl/wrap_socket_non_socket_raises.py`.
#[test]
fn test_gen_errors_std_libs_ssl_wrap_socket_non_socket_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "wrap_socket_non_socket_raises"
# subject = "ssl.SSLContext.wrap_socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_socket: wrap_socket_non_socket_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().wrap_socket(42)
except AttributeError:
    _raised = True
assert _raised, "wrap_socket_non_socket_raises: expected AttributeError"
print("wrap_socket_non_socket_raises OK")
"###);
    assert_output(&out, r###"wrap_socket_non_socket_raises OK
"###);
}
