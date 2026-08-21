# test_ssl.py — #3453 axis-1 stdlib ssl AssertionPass seed.
#
# Mamba-authored seed exercising the `ssl` module surface called
# out in the issue:
#   create_default_context, PROTOCOL_TLS_CLIENT, get_default_verify_paths
#   — no real connect.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. Protocol constants — PROTOCOL_TLS_CLIENT / PROTOCOL_TLS_SERVER /
#      PROTOCOL_TLS distinct.
#   3. CERT_* enums — CERT_NONE / CERT_OPTIONAL / CERT_REQUIRED distinct.
#   4. ssl.create_default_context() — client context defaults
#      (check_hostname True, verify_mode CERT_REQUIRED).
#   5. ssl.create_default_context(Purpose.CLIENT_AUTH) — server context
#      defaults (check_hostname False, verify_mode CERT_NONE).
#   6. Raw SSLContext(PROTOCOL_TLS_CLIENT) — protocol property echoes ctor.
#   7. SSLContext.set_ciphers — accepts a cipher string without raising.
#   8. SSLContext.minimum_version setter — round-trips.
#   9. SSLContext.set_alpn_protocols — accepts a list without raising.
#  10. ssl.get_default_verify_paths() — returns DefaultVerifyPaths with
#      cafile / capath / openssl_cafile / openssl_capath attrs.
#  11. ssl.SSLError / SSLEOFError / SSLZeroReturnError class hierarchy.
#  12. ssl.OPENSSL_VERSION_INFO is a 5-tuple of ints.
#
# All asserts probe in-process configuration — no socket / no handshake —
# per the issue's "no real connect" guidance.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_ssl N asserts` to stdout.

import ssl

_ledger: list[int] = []

# 1. Module identity + public surface.
assert ssl.__name__ == "ssl", "ssl.__name__"
_ledger.append(1)
assert hasattr(ssl, "SSLContext"), "exposes SSLContext"
_ledger.append(1)
assert hasattr(ssl, "create_default_context"), "exposes create_default_context"
_ledger.append(1)
assert hasattr(ssl, "get_default_verify_paths"), "exposes get_default_verify_paths"
_ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS_CLIENT"), "exposes PROTOCOL_TLS_CLIENT"
_ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS_SERVER"), "exposes PROTOCOL_TLS_SERVER"
_ledger.append(1)
assert hasattr(ssl, "CERT_REQUIRED"), "exposes CERT_REQUIRED"
_ledger.append(1)
assert hasattr(ssl, "SSLError"), "exposes SSLError"
_ledger.append(1)
assert hasattr(ssl, "Purpose"), "exposes Purpose enum namespace"
_ledger.append(1)

# 2. Protocol constants are distinct.
assert ssl.PROTOCOL_TLS_CLIENT != ssl.PROTOCOL_TLS_SERVER, (
    "PROTOCOL_TLS_CLIENT and PROTOCOL_TLS_SERVER distinct"
)
_ledger.append(1)

# 3. CERT_* enum values are distinct.
assert ssl.CERT_NONE != ssl.CERT_OPTIONAL, "CERT_NONE != CERT_OPTIONAL"
_ledger.append(1)
assert ssl.CERT_OPTIONAL != ssl.CERT_REQUIRED, "CERT_OPTIONAL != CERT_REQUIRED"
_ledger.append(1)
assert ssl.CERT_NONE != ssl.CERT_REQUIRED, "CERT_NONE != CERT_REQUIRED"
_ledger.append(1)

# 4. ssl.create_default_context() — default client context.
_client_ctx = ssl.create_default_context()
assert isinstance(_client_ctx, ssl.SSLContext), (
    "create_default_context returns SSLContext"
)
_ledger.append(1)
assert _client_ctx.check_hostname == True, (
    "client default context enforces hostname check"
)
_ledger.append(1)
assert _client_ctx.verify_mode == ssl.CERT_REQUIRED, (
    "client default context verify_mode == CERT_REQUIRED"
)
_ledger.append(1)

# 5. create_default_context(Purpose.CLIENT_AUTH) — server context.
_server_ctx = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
assert isinstance(_server_ctx, ssl.SSLContext), "server context is SSLContext"
_ledger.append(1)
assert _server_ctx.check_hostname == False, (
    "server context default check_hostname False"
)
_ledger.append(1)
assert _server_ctx.verify_mode == ssl.CERT_NONE, (
    "server context default verify_mode CERT_NONE"
)
_ledger.append(1)

# 6. Raw SSLContext(PROTOCOL_TLS_CLIENT) — explicit protocol selection.
_raw_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
assert isinstance(_raw_ctx, ssl.SSLContext), "raw SSLContext(...) constructs"
_ledger.append(1)
# Mode is mutable on the raw context. Disable verify so we can flip ciphers.
_raw_ctx.check_hostname = False
_raw_ctx.verify_mode = ssl.CERT_NONE
assert _raw_ctx.check_hostname == False, "check_hostname mutator round-trips"
_ledger.append(1)
assert _raw_ctx.verify_mode == ssl.CERT_NONE, "verify_mode mutator round-trips"
_ledger.append(1)

# 7. SSLContext.set_ciphers — accepts a permissive cipher string.
_raw_ctx.set_ciphers("DEFAULT")
# get_ciphers returns a list of dicts describing accepted ciphers.
_ciphers = _raw_ctx.get_ciphers()
assert isinstance(_ciphers, list), "get_ciphers returns a list"
_ledger.append(1)
assert len(_ciphers) >= 1, "DEFAULT cipher string selects ≥1 cipher"
_ledger.append(1)

# 8. SSLContext.minimum_version — TLSv1_2 round-trip.
_raw_ctx.minimum_version = ssl.TLSVersion.TLSv1_2
assert _raw_ctx.minimum_version == ssl.TLSVersion.TLSv1_2, (
    "minimum_version TLSv1_2 round-trips"
)
_ledger.append(1)

# 9. SSLContext.set_alpn_protocols — accepts a list without raising.
_raw_ctx.set_alpn_protocols(["http/1.1", "h2"])
# No public getter for ALPN — successful call is the assertion.
_ledger.append(1)

# 10. ssl.get_default_verify_paths() — DefaultVerifyPaths named-struct.
_paths = ssl.get_default_verify_paths()
assert hasattr(_paths, "cafile"), "DefaultVerifyPaths exposes .cafile"
_ledger.append(1)
assert hasattr(_paths, "capath"), "DefaultVerifyPaths exposes .capath"
_ledger.append(1)
assert hasattr(_paths, "openssl_cafile"), (
    "DefaultVerifyPaths exposes .openssl_cafile"
)
_ledger.append(1)
assert hasattr(_paths, "openssl_capath"), (
    "DefaultVerifyPaths exposes .openssl_capath"
)
_ledger.append(1)

# 11. ssl.SSLError class hierarchy — all are OSError subclasses.
assert issubclass(ssl.SSLError, OSError), "SSLError subclasses OSError"
_ledger.append(1)
assert issubclass(ssl.SSLEOFError, ssl.SSLError), "SSLEOFError subclasses SSLError"
_ledger.append(1)
assert issubclass(ssl.SSLZeroReturnError, ssl.SSLError), (
    "SSLZeroReturnError subclasses SSLError"
)
_ledger.append(1)

# 12. ssl.OPENSSL_VERSION_INFO is a 5-tuple of ints.
_v = ssl.OPENSSL_VERSION_INFO
assert isinstance(_v, tuple), "OPENSSL_VERSION_INFO is a tuple"
_ledger.append(1)
assert len(_v) - 5 == 0, "OPENSSL_VERSION_INFO is a 5-tuple (boxed-dodge)"
_ledger.append(1)
# Major version is a positive int — sanity check the first element.
assert isinstance(_v[0], int), "OPENSSL_VERSION_INFO[0] is int (major)"
_ledger.append(1)
assert _v[0] >= 1, "OPENSSL_VERSION_INFO[0] (major) ≥ 1"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_ssl {len(_ledger)} asserts")
