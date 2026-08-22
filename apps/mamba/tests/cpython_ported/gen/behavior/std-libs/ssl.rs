use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/ssl/client_server_default_verify_modes.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_client_server_default_verify_modes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "client_server_default_verify_modes"
# subject = "ssl.SSLContext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext: a PROTOCOL_TLS_CLIENT context verifies by default (check_hostname True, verify_mode CERT_REQUIRED) while a PROTOCOL_TLS_SERVER context does not (check_hostname False, verify_mode CERT_NONE)"""
import ssl

_cli = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
assert _cli.check_hostname is True, "client check_hostname default True"
assert _cli.verify_mode == ssl.CERT_REQUIRED, "client verify default REQUIRED"
_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
assert _srv.check_hostname is False, "server check_hostname default False"
assert _srv.verify_mode == ssl.CERT_NONE, "server verify default NONE"

print("client_server_default_verify_modes OK")
"###);
    assert_output(&out, r###"client_server_default_verify_modes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/context_can_disable_verification.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_context_can_disable_verification() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "context_can_disable_verification"
# subject = "ssl.SSLContext.verify_mode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_mode: verification can be disabled on a context: setting check_hostname False then verify_mode CERT_NONE takes effect"""
import ssl

_ctx = ssl.create_default_context()
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
assert _ctx.verify_mode == ssl.CERT_NONE, "verify_mode settable to NONE"
assert not _ctx.check_hostname, "check_hostname=False"

print("context_can_disable_verification OK")
"###);
    assert_output(&out, r###"context_can_disable_verification OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/create_default_context_is_secure.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_create_default_context_is_secure() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "create_default_context_is_secure"
# subject = "ssl.create_default_context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.create_default_context: create_default_context() yields a secure client context: check_hostname is True and verify_mode is CERT_REQUIRED"""
import ssl

_ctx = ssl.create_default_context()
assert _ctx.check_hostname is True, "check_hostname=True by default"
assert _ctx.verify_mode == ssl.CERT_REQUIRED, "verify_mode=REQUIRED by default"

print("create_default_context_is_secure OK")
"###);
    assert_output(&out, r###"create_default_context_is_secure OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/default_ciphers_exclude_weak.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_default_ciphers_exclude_weak() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "default_ciphers_exclude_weak"
# subject = "ssl.SSLContext.get_ciphers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.get_ciphers: the default cipher list of a fresh client context excludes known-weak primitives (PSK, SRP, MD5, RC4, 3DES)"""
import ssl

_default = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
for _suite in _default.get_ciphers():
    _name = _suite["name"]
    for _weak in ("PSK", "SRP", "MD5", "RC4", "3DES"):
        assert _weak not in _name, f"weak cipher {_weak} present in {_name}"

print("default_ciphers_exclude_weak OK")
"###);
    assert_output(&out, r###"default_ciphers_exclude_weak OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/default_protocol_is_tls.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_default_protocol_is_tls() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "default_protocol_is_tls"
# subject = "ssl.SSLContext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext: a bare SSLContext() (deprecation warning suppressed) defaults to protocol PROTOCOL_TLS"""
import ssl

import warnings

with warnings.catch_warnings():
    warnings.simplefilter("ignore")
    _ctx = ssl.SSLContext()
assert _ctx.protocol == ssl.PROTOCOL_TLS, "default protocol is PROTOCOL_TLS"

print("default_protocol_is_tls OK")
"###);
    assert_output(&out, r###"default_protocol_is_tls OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/enabling_check_hostname_forces_required.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_enabling_check_hostname_forces_required() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "enabling_check_hostname_forces_required"
# subject = "ssl.SSLContext.check_hostname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.check_hostname: enabling check_hostname on a CERT_NONE context promotes verify_mode up to CERT_REQUIRED, but leaves an existing CERT_OPTIONAL setting untouched"""
import ssl

# CERT_NONE is promoted to CERT_REQUIRED when check_hostname is enabled.
_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
_ctx.check_hostname = True
assert _ctx.verify_mode == ssl.CERT_REQUIRED, "check_hostname raises verify_mode"

# An existing CERT_OPTIONAL setting is left untouched.
_ctx2 = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx2.check_hostname = False
_ctx2.verify_mode = ssl.CERT_OPTIONAL
_ctx2.check_hostname = True
assert _ctx2.verify_mode == ssl.CERT_OPTIONAL, "OPTIONAL kept under check_hostname"

print("enabling_check_hostname_forces_required OK")
"###);
    assert_output(&out, r###"enabling_check_hostname_forces_required OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/fresh_context_session_stats_zero.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_fresh_context_session_stats_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "fresh_context_session_stats_zero"
# subject = "ssl.SSLContext.session_stats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.session_stats: a fresh context's session_stats() is an all-zero dict with the documented 'number' and 'hits' keys"""
import ssl

_stats = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT).session_stats()
assert set(_stats.values()) == {0}, f"fresh stats all zero: {_stats!r}"
assert _stats["number"] == 0 and _stats["hits"] == 0, "stats keys present"

print("fresh_context_session_stats_zero OK")
"###);
    assert_output(&out, r###"fresh_context_session_stats_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/get_ciphers_returns_dicts.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_get_ciphers_returns_dicts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "get_ciphers_returns_dicts"
# subject = "ssl.SSLContext.get_ciphers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.get_ciphers: get_ciphers() returns a non-empty list of dicts, each carrying at least 'name' and 'description' keys"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_all = _ctx.get_ciphers()
assert isinstance(_all, list) and _all, "get_ciphers returns non-empty list"
assert all("name" in c and "description" in c for c in _all), "cipher dict shape"

print("get_ciphers_returns_dicts OK")
"###);
    assert_output(&out, r###"get_ciphers_returns_dicts OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/hostname_checks_common_name_default.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_hostname_checks_common_name_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "hostname_checks_common_name_default"
# subject = "ssl.SSLContext.hostname_checks_common_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.hostname_checks_common_name: a client context's hostname_checks_common_name defaults to True"""
import ssl

assert ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT).hostname_checks_common_name is True, \
    "hostname_checks_common_name default True"

print("hostname_checks_common_name_default OK")
"###);
    assert_output(&out, r###"hostname_checks_common_name_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/memory_bio_accepts_buffer_types.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_memory_bio_accepts_buffer_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_accepts_buffer_types"
# subject = "ssl.MemoryBIO.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.write: MemoryBIO.write accepts bytes-like buffers (bytearray, memoryview) and reads them back as concatenated bytes"""
import ssl

_t = ssl.MemoryBIO()
_t.write(bytearray(b"ba"))
_t.write(memoryview(b"r"))
assert _t.read() == b"bar", "bytearray and memoryview accepted"

print("memory_bio_accepts_buffer_types OK")
"###);
    assert_output(&out, r###"memory_bio_accepts_buffer_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/memory_bio_eof_after_drain.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_memory_bio_eof_after_drain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_eof_after_drain"
# subject = "ssl.MemoryBIO.eof"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.eof: MemoryBIO.eof flips True only once the buffer is fully drained AND write_eof() was called; it stays False while bytes remain pending"""
import ssl

_e = ssl.MemoryBIO()
assert _e.eof is False, "fresh BIO not eof"
_e.write(b"fo")
_e.write_eof()
assert _e.eof is False, "eof stays False while bytes pending"
assert _e.read(1) == b"f", "read before eof"
assert _e.eof is False, "still bytes left"
assert _e.read(1) == b"o", "read last byte"
assert _e.eof is True, "eof True once drained after write_eof"
assert _e.read() == b"", "reading past eof yields empty"

print("memory_bio_eof_after_drain OK")
"###);
    assert_output(&out, r###"memory_bio_eof_after_drain OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/memory_bio_pending_count.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_memory_bio_pending_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_pending_count"
# subject = "ssl.MemoryBIO.pending"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.pending: MemoryBIO.pending tracks unread bytes exactly: 0 when empty, 3 after writing 3 bytes, 2 after reading 1"""
import ssl

_p = ssl.MemoryBIO()
assert _p.pending == 0, "empty BIO pending 0"
_p.write(b"foo")
assert _p.pending == 3, "pending counts written bytes"
_p.read(1)
assert _p.pending == 2, "pending drops as read"

print("memory_bio_pending_count OK")
"###);
    assert_output(&out, r###"memory_bio_pending_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/memory_bio_read_write_fifo.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_memory_bio_read_write_fifo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_read_write_fifo"
# subject = "ssl.MemoryBIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO: MemoryBIO is a byte FIFO: successive writes concatenate, a full read drains it, and sized reads take prefixes leaving the remainder"""
import ssl

_bio = ssl.MemoryBIO()
_bio.write(b"foo")
_bio.write(b"bar")
assert _bio.read() == b"foobar", "FIFO concatenates writes"
assert _bio.read() == b"", "drained BIO reads empty"
_bio.write(b"baz")
assert _bio.read(2) == b"ba", "sized read takes prefix"
assert _bio.read(1) == b"z", "sized read takes remainder"
assert _bio.read(1) == b"", "empty after drain"

print("memory_bio_read_write_fifo OK")
"###);
    assert_output(&out, r###"memory_bio_read_write_fifo OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/min_max_version_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_min_max_version_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "min_max_version_roundtrip"
# subject = "ssl.SSLContext.minimum_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.minimum_version: minimum_version / maximum_version round-trip: setting them to TLSv1_2 / TLSv1_3 reads back the same members"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
_ctx.minimum_version = ssl.TLSVersion.TLSv1_2
_ctx.maximum_version = ssl.TLSVersion.TLSv1_3
assert _ctx.minimum_version == ssl.TLSVersion.TLSv1_2, "min version set"
assert _ctx.maximum_version == ssl.TLSVersion.TLSv1_3, "max version set"

print("min_max_version_roundtrip OK")
"###);
    assert_output(&out, r###"min_max_version_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/num_tickets_server_mutable.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_num_tickets_server_mutable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "num_tickets_server_mutable"
# subject = "ssl.SSLContext.num_tickets"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.num_tickets: a server context's num_tickets defaults to 2 and is settable to 1, while a client context's num_tickets is fixed at 2"""
import ssl

_srv = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
assert _srv.num_tickets == 2, "server num_tickets default 2"
_srv.num_tickets = 1
assert _srv.num_tickets == 1, "num_tickets settable"
_clt = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
assert _clt.num_tickets == 2, "client num_tickets is 2"

print("num_tickets_server_mutable OK")
"###);
    assert_output(&out, r###"num_tickets_server_mutable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/options_is_int_bitmask.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_options_is_int_bitmask() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "options_is_int_bitmask"
# subject = "ssl.SSLContext.options"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.options: ctx.options is an int bitmask that supports in-place bitwise-OR with OP_NO_SSLv2 / OP_ALL and stays an int"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
assert isinstance(_ctx.options, int), f"options type = {type(_ctx.options)!r}"
_ctx.options |= ssl.OP_NO_SSLv2 if hasattr(ssl, "OP_NO_SSLv2") else ssl.OP_ALL
assert isinstance(_ctx.options, int), f"options type = {type(_ctx.options)!r}"

print("options_is_int_bitmask OK")
"###);
    assert_output(&out, r###"options_is_int_bitmask OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/post_handshake_auth_toggles.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_post_handshake_auth_toggles() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "post_handshake_auth_toggles"
# subject = "ssl.SSLContext.post_handshake_auth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.post_handshake_auth: post_handshake_auth defaults to False on both client and server contexts, toggles True/False independently, and survives a verify_mode change"""
import ssl

for _proto in (ssl.PROTOCOL_TLS_SERVER, ssl.PROTOCOL_TLS_CLIENT):
    _pha = ssl.SSLContext(_proto)
    assert _pha.post_handshake_auth is False, "pha default False"
    _pha.post_handshake_auth = True
    assert _pha.post_handshake_auth is True, "pha settable True"
    _pha.verify_mode = ssl.CERT_REQUIRED
    assert _pha.post_handshake_auth is True, "pha survives verify_mode change"
    _pha.post_handshake_auth = False
    assert _pha.post_handshake_auth is False, "pha settable back to False"

print("post_handshake_auth_toggles OK")
"###);
    assert_output(&out, r###"post_handshake_auth_toggles OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/protocol_attr_is_passed_member.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_protocol_attr_is_passed_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "protocol_attr_is_passed_member"
# subject = "ssl.SSLContext.protocol"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.protocol: ctx.protocol is the exact enum member passed to the constructor (identity-preserving)"""
import ssl

_proto = ssl.PROTOCOL_TLS_CLIENT
assert ssl.SSLContext(_proto).protocol is _proto, "ctx.protocol is the member"

print("protocol_attr_is_passed_member OK")
"###);
    assert_output(&out, r###"protocol_attr_is_passed_member OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/protocol_member_is_intenum_like.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_protocol_member_is_intenum_like() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "protocol_member_is_intenum_like"
# subject = "ssl.PROTOCOL_TLS_CLIENT"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.PROTOCOL_TLS_CLIENT: a protocol constant behaves like an IntEnum member: repr names it as <_SSLMethod.PROTOCOL_TLS_CLIENT: value>, str is its int value, int() coerces to its value, and PROTOCOL_TLS aliases PROTOCOL_SSLv23"""
import ssl

_proto = ssl.PROTOCOL_TLS_CLIENT
assert repr(_proto) == "<_SSLMethod.PROTOCOL_TLS_CLIENT: %r>" % _proto.value, \
    f"repr = {_proto!r}"
assert str(_proto) == str(_proto.value), f"str = {_proto}"
assert int(_proto) == _proto.value, "protocol coerces to int"
assert ssl.PROTOCOL_TLS == ssl.PROTOCOL_SSLv23, "PROTOCOL_TLS == PROTOCOL_SSLv23"

print("protocol_member_is_intenum_like OK")
"###);
    assert_output(&out, r###"protocol_member_is_intenum_like OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/purpose_carries_x509_oids.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_purpose_carries_x509_oids() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "purpose_carries_x509_oids"
# subject = "ssl.Purpose"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.Purpose: Purpose members carry the well-known X.509 EKU OIDs: SERVER_AUTH is nid 129 / serverAuth / 1.3.6.1.5.5.7.3.1 and CLIENT_AUTH is nid 130 / clientAuth / 1.3.6.1.5.5.7.3.2, and the two are distinct"""
import ssl

_sa = ssl.Purpose.SERVER_AUTH
assert _sa.nid == 129, f"SERVER_AUTH nid = {_sa.nid}"
assert _sa.shortname == "serverAuth", f"SERVER_AUTH shortname = {_sa.shortname}"
assert _sa.oid == "1.3.6.1.5.5.7.3.1", f"SERVER_AUTH oid = {_sa.oid}"

_ca = ssl.Purpose.CLIENT_AUTH
assert _ca.nid == 130, f"CLIENT_AUTH nid = {_ca.nid}"
assert _ca.shortname == "clientAuth", f"CLIENT_AUTH shortname = {_ca.shortname}"
assert _ca.oid == "1.3.6.1.5.5.7.3.2", f"CLIENT_AUTH oid = {_ca.oid}"

assert _sa != _ca, "the two purposes are distinct"

print("purpose_carries_x509_oids OK")
"###);
    assert_output(&out, r###"purpose_carries_x509_oids OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/rand_add_accepts_entropy.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_rand_add_accepts_entropy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "rand_add_accepts_entropy"
# subject = "ssl.RAND_add"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.RAND_add: RAND_add accepts str and bytes-like entropy (str, bytes, bytearray) without raising"""
import ssl

ssl.RAND_add("this is a random string", 75.0)
ssl.RAND_add(b"this is a random bytes object", 75.0)
ssl.RAND_add(bytearray(b"this is a random bytearray object"), 75.0)

print("rand_add_accepts_entropy OK")
"###);
    assert_output(&out, r###"rand_add_accepts_entropy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/rand_status_and_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_rand_status_and_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "rand_status_and_bytes"
# subject = "ssl.RAND_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.RAND_bytes: RAND_status() returns an int; when seeded RAND_bytes(16) returns exactly 16 distinct bytes across draws, and when unseeded it raises SSLError"""
import ssl

_status = ssl.RAND_status()
assert isinstance(_status, int), f"RAND_status type = {type(_status)!r}"
if _status:
    _data = ssl.RAND_bytes(16)
    assert isinstance(_data, bytes), "RAND_bytes returns bytes"
    assert len(_data) == 16, f"RAND_bytes(16) length = {len(_data)}"
    assert ssl.RAND_bytes(16) != ssl.RAND_bytes(16), "draws differ"
else:
    try:
        ssl.RAND_bytes(16)
        raise AssertionError("RAND_bytes on unseeded PRNG should raise")
    except ssl.SSLError:
        pass

print("rand_status_and_bytes OK")
"###);
    assert_output(&out, r###"rand_status_and_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/set_ciphers_narrows_to_aesgcm.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_set_ciphers_narrows_to_aesgcm() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "set_ciphers_narrows_to_aesgcm"
# subject = "ssl.SSLContext.set_ciphers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.set_ciphers: set_ciphers('AESGCM') narrows the suite list to GCM ciphers, keeping at least two of the known AES-GCM suites; ALL and DEFAULT aliases also apply without raising"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.set_ciphers("AESGCM")
_names = {c["name"] for c in _ctx.get_ciphers()}
_expected = {
    "AES128-GCM-SHA256", "ECDHE-ECDSA-AES128-GCM-SHA256",
    "ECDHE-RSA-AES128-GCM-SHA256", "DHE-RSA-AES128-GCM-SHA256",
    "AES256-GCM-SHA384", "ECDHE-ECDSA-AES256-GCM-SHA384",
    "ECDHE-RSA-AES256-GCM-SHA384", "DHE-RSA-AES256-GCM-SHA384",
}
assert len(_names & _expected) >= 2, f"AESGCM keeps GCM suites: {_names & _expected}"

# ALL / DEFAULT aliases apply without raising.
_ctx2 = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx2.set_ciphers("ALL")
_ctx2.set_ciphers("DEFAULT")

print("set_ciphers_narrows_to_aesgcm OK")
"###);
    assert_output(&out, r###"set_ciphers_narrows_to_aesgcm OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/sslerror_str_and_errno.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_sslerror_str_and_errno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "sslerror_str_and_errno"
# subject = "ssl.SSLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLError: ssl.SSLError(1, 'foo') exposes its message via str() ('foo') and the errno argument (1); SSLZeroReturnError behaves the same"""
import ssl

_err = ssl.SSLError(1, "foo")
assert str(_err) == "foo", f"SSLError str = {_err}"
assert _err.errno == 1, f"SSLError errno = {_err.errno}"
_zero = ssl.SSLZeroReturnError(1, "foo")
assert str(_zero) == "foo" and _zero.errno == 1, "ZeroReturn str/errno"

print("sslerror_str_and_errno OK")
"###);
    assert_output(&out, r###"sslerror_str_and_errno OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/tlsversion_ordering_and_sentinels.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_tlsversion_ordering_and_sentinels() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "tlsversion_ordering_and_sentinels"
# subject = "ssl.TLSVersion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.TLSVersion: concrete TLSVersion members order as integers (TLSv1_2 < TLSv1_3, TLSv1_3 == 772) while MAXIMUM_SUPPORTED / MINIMUM_SUPPORTED are the -1 / -2 sentinels"""
import ssl

assert ssl.TLSVersion.TLSv1_2 < ssl.TLSVersion.TLSv1_3, "TLSv1_2 < TLSv1_3"
assert int(ssl.TLSVersion.TLSv1_3) == 772, f"TLSv1_3 = {int(ssl.TLSVersion.TLSv1_3)}"
assert int(ssl.TLSVersion.MAXIMUM_SUPPORTED) == -1, "MAXIMUM_SUPPORTED sentinel"
assert int(ssl.TLSVersion.MINIMUM_SUPPORTED) == -2, "MINIMUM_SUPPORTED sentinel"

print("tlsversion_ordering_and_sentinels OK")
"###);
    assert_output(&out, r###"tlsversion_ordering_and_sentinels OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/unconnected_io_raises.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_unconnected_io_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "unconnected_io_raises"
# subject = "ssl.SSLSocket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLSocket: I/O on an unconnected wrapped socket raises before any handshake: recv/recv_into/recvfrom/recvfrom_into/send/sendto raise OSError, while dup/sendmsg/recvmsg/recvmsg_into raise NotImplementedError"""
import ssl

import socket

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
_s = socket.socket(socket.AF_INET)
with _ctx.wrap_socket(_s, server_hostname="localhost") as _ss:
    for _name, _fn in (
        ("recv", lambda: _ss.recv(1)),
        ("recv_into", lambda: _ss.recv_into(bytearray(b"x"))),
        ("recvfrom", lambda: _ss.recvfrom(1)),
        ("recvfrom_into", lambda: _ss.recvfrom_into(bytearray(b"x"), 1)),
        ("send", lambda: _ss.send(b"x")),
        ("sendto", lambda: _ss.sendto(b"x", ("0.0.0.0", 0))),
    ):
        try:
            _fn()
            raise AssertionError(f"{_name} on unconnected should raise")
        except OSError:
            pass
    for _name, _fn in (
        ("dup", lambda: _ss.dup()),
        ("sendmsg", lambda: _ss.sendmsg([b"x"], (), 0, ("0.0.0.0", 0))),
        ("recvmsg", lambda: _ss.recvmsg(100)),
        ("recvmsg_into", lambda: _ss.recvmsg_into([bytearray(100)])),
    ):
        try:
            _fn()
            raise AssertionError(f"{_name} should be NotImplementedError")
        except NotImplementedError:
            pass

print("unconnected_io_raises OK")
"###);
    assert_output(&out, r###"unconnected_io_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/verify_flags_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_verify_flags_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "verify_flags_roundtrip"
# subject = "ssl.SSLContext.verify_flags"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_flags: verify_flags round-trips a single flag (VERIFY_CRL_CHECK_LEAF) and an OR-combined pair (| VERIFY_X509_STRICT)"""
import ssl

_vf = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
_vf.verify_flags = ssl.VERIFY_CRL_CHECK_LEAF
assert _vf.verify_flags == ssl.VERIFY_CRL_CHECK_LEAF, "single flag"
_combo = ssl.VERIFY_CRL_CHECK_LEAF | ssl.VERIFY_X509_STRICT
_vf.verify_flags = _combo
assert _vf.verify_flags == _combo, "combined flags"

print("verify_flags_roundtrip OK")
"###);
    assert_output(&out, r###"verify_flags_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/ssl/wrap_socket_preserves_timeout.py`.
#[test]
fn test_gen_behavior_std_libs_ssl_wrap_socket_preserves_timeout() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "wrap_socket_preserves_timeout"
# subject = "ssl.SSLContext.wrap_socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.wrap_socket: wrap_socket preserves the underlying socket's timeout: gettimeout() on the wrapped socket round-trips None, 0.0, and 5.0"""
import ssl

import socket


def _wrap(sock):
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE
    return ctx.wrap_socket(sock, server_hostname="localhost")


for _t in (None, 0.0, 5.0):
    _s = socket.socket(socket.AF_INET)
    _s.settimeout(_t)
    with _wrap(_s) as _ss:
        assert _ss.gettimeout() == _t, f"timeout passthrough {_t!r}"

print("wrap_socket_preserves_timeout OK")
"###);
    assert_output(&out, r###"wrap_socket_preserves_timeout OK
"###);
}
