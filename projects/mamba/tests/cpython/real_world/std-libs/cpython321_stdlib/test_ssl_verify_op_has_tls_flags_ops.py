# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_ssl_verify_op_has_tls_flags_ops"
# subject = "cpython321.test_ssl_verify_op_has_tls_flags_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_ssl_verify_op_has_tls_flags_ops.py"
# status = "filled"
# ///
"""cpython321.test_ssl_verify_op_has_tls_flags_ops: execute CPython 3.12 seed test_ssl_verify_op_has_tls_flags_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `ssl` stdlib module's
# protocol-feature flag surface — the constants that every TLS client
# / server reaches through to (a) decide which TLS versions to refuse
# (`OP_NO_*`), (b) check whether the underlying OpenSSL build supports
# a given TLS version (`HAS_TLSv1*`, `HAS_ALPN`), (c) tune the
# certificate-verification flags (`VERIFY_*`), and (d) introspect the
# OpenSSL version pinned at build time (`OPENSSL_VERSION`,
# `OPENSSL_VERSION_NUMBER`). Existing `test_ssl_constants_ops.py`
# pins only `CERT_NONE/OPTIONAL/REQUIRED` + `PROTOCOL_TLS_CLIENT/SERVER`
# — this seed fills the rest of the matching constant surface plus
# the SSLError exception-hierarchy invariants and the OpenSSL build
# introspection.
#
# Surface:
#   • ssl.VERIFY_DEFAULT == 0
#       — neutral baseline (no extra cert-verify flags);
#   • ssl.VERIFY_CRL_CHECK_LEAF == 4
#       — enable CRL check on the leaf cert only;
#   • ssl.VERIFY_CRL_CHECK_CHAIN == 12
#       — enable CRL check on the whole chain (4 | 8);
#   • ssl.VERIFY_X509_STRICT == 32
#       — strict X.509 parsing;
#   • ssl.VERIFY_X509_TRUSTED_FIRST == 32768
#       — prefer trusted root, OpenSSL 1.0.2+ default;
#   • ssl.OP_NO_SSLv3 == 33554432 (= 1 << 25)
#       — disable SSLv3 fallback;
#   • ssl.OP_NO_TLSv1 == 67108864 (= 1 << 26)
#       — disable TLS 1.0;
#   • ssl.OP_NO_TLSv1_1 == 268435456 (= 1 << 28)
#       — disable TLS 1.1;
#   • ssl.OP_NO_TLSv1_2 == 134217728 (= 1 << 27)
#       — disable TLS 1.2;
#   • ssl.OP_NO_TLSv1_3 == 536870912 (= 1 << 29)
#       — disable TLS 1.3;
#   • ssl.OP_NO_COMPRESSION == 131072 (= 1 << 17)
#       — disable TLS-layer compression (CRIME mitigation);
#   • ssl.HAS_TLSv1 / HAS_TLSv1_1 / HAS_TLSv1_2 / HAS_TLSv1_3 — bool
#       (whether the linked OpenSSL build supports the version);
#   • ssl.HAS_ALPN — bool (ALPN extension support);
#   • ssl.OPENSSL_VERSION → str (e.g. "OpenSSL 3.0.x");
#   • ssl.OPENSSL_VERSION_NUMBER → int;
#   • ssl.SSLError → exception class;
#   • ssl.SSLZeroReturnError / SSLWantReadError / SSLWantWriteError /
#     SSLSyscallError → all `issubclass(..., SSLError)` returns True;
#   • module-level attribute discipline.
import ssl
_ledger: list[int] = []

# Cert-verify flags (matching subset)
assert ssl.VERIFY_DEFAULT == 0; _ledger.append(1)
assert ssl.VERIFY_CRL_CHECK_LEAF == 4; _ledger.append(1)
assert ssl.VERIFY_CRL_CHECK_CHAIN == 12; _ledger.append(1)
assert ssl.VERIFY_X509_STRICT == 32; _ledger.append(1)
assert ssl.VERIFY_X509_TRUSTED_FIRST == 32768; _ledger.append(1)

# VERIFY_CRL_CHECK_CHAIN is bitwise superset of CHECK_LEAF
assert ssl.VERIFY_CRL_CHECK_CHAIN & ssl.VERIFY_CRL_CHECK_LEAF == ssl.VERIFY_CRL_CHECK_LEAF; _ledger.append(1)
assert ssl.VERIFY_DEFAULT < ssl.VERIFY_CRL_CHECK_LEAF; _ledger.append(1)
assert ssl.VERIFY_CRL_CHECK_LEAF < ssl.VERIFY_CRL_CHECK_CHAIN; _ledger.append(1)
assert ssl.VERIFY_X509_STRICT < ssl.VERIFY_X509_TRUSTED_FIRST; _ledger.append(1)

# OP_NO_* TLS-version disable bits — canonical OpenSSL values
assert ssl.OP_NO_SSLv3 == 33554432; _ledger.append(1)
assert ssl.OP_NO_TLSv1 == 67108864; _ledger.append(1)
assert ssl.OP_NO_TLSv1_1 == 268435456; _ledger.append(1)
assert ssl.OP_NO_TLSv1_2 == 134217728; _ledger.append(1)
assert ssl.OP_NO_TLSv1_3 == 536870912; _ledger.append(1)
assert ssl.OP_NO_COMPRESSION == 131072; _ledger.append(1)

# Bit-shift identity (each OP_NO_* is a single bit)
assert ssl.OP_NO_SSLv3 == (1 << 25); _ledger.append(1)
assert ssl.OP_NO_TLSv1 == (1 << 26); _ledger.append(1)
assert ssl.OP_NO_TLSv1_2 == (1 << 27); _ledger.append(1)
assert ssl.OP_NO_TLSv1_1 == (1 << 28); _ledger.append(1)
assert ssl.OP_NO_TLSv1_3 == (1 << 29); _ledger.append(1)
assert ssl.OP_NO_COMPRESSION == (1 << 17); _ledger.append(1)

# OP_NO_* flags are pairwise distinct
_op_no_flags = {ssl.OP_NO_SSLv3, ssl.OP_NO_TLSv1, ssl.OP_NO_TLSv1_1,
                ssl.OP_NO_TLSv1_2, ssl.OP_NO_TLSv1_3,
                ssl.OP_NO_COMPRESSION}
assert len(_op_no_flags) == 6; _ledger.append(1)

# OR-combinable flags (no overlap with each other → OR ≥ each)
_combo = ssl.OP_NO_SSLv3 | ssl.OP_NO_TLSv1
assert _combo >= ssl.OP_NO_SSLv3; _ledger.append(1)
assert _combo >= ssl.OP_NO_TLSv1; _ledger.append(1)
assert _combo & ssl.OP_NO_SSLv3 == ssl.OP_NO_SSLv3; _ledger.append(1)
assert _combo & ssl.OP_NO_TLSv1 == ssl.OP_NO_TLSv1; _ledger.append(1)

# HAS_TLSv1* — bool feature flags (TLS 1.0+ is universal in 2026)
assert isinstance(ssl.HAS_TLSv1, bool); _ledger.append(1)
assert isinstance(ssl.HAS_TLSv1_1, bool); _ledger.append(1)
assert isinstance(ssl.HAS_TLSv1_2, bool); _ledger.append(1)
assert isinstance(ssl.HAS_TLSv1_3, bool); _ledger.append(1)
assert isinstance(ssl.HAS_ALPN, bool); _ledger.append(1)

# Modern OpenSSL builds support all major TLS versions
assert ssl.HAS_TLSv1_2 == True; _ledger.append(1)
assert ssl.HAS_TLSv1_3 == True; _ledger.append(1)

# OPENSSL_VERSION introspection
assert isinstance(ssl.OPENSSL_VERSION, str); _ledger.append(1)
assert len(ssl.OPENSSL_VERSION) > 0; _ledger.append(1)
assert "OpenSSL" in ssl.OPENSSL_VERSION; _ledger.append(1)
assert isinstance(ssl.OPENSSL_VERSION_NUMBER, int); _ledger.append(1)
assert ssl.OPENSSL_VERSION_NUMBER > 0; _ledger.append(1)

# SSL exception hierarchy — SSLError variants subclass SSLError
assert issubclass(ssl.SSLZeroReturnError, ssl.SSLError); _ledger.append(1)
assert issubclass(ssl.SSLWantReadError, ssl.SSLError); _ledger.append(1)
assert issubclass(ssl.SSLWantWriteError, ssl.SSLError); _ledger.append(1)
assert issubclass(ssl.SSLSyscallError, ssl.SSLError); _ledger.append(1)

# Module-level attribute discipline
for _name in ['SSLContext', 'SSLError', 'SSLZeroReturnError',
              'SSLWantReadError', 'SSLWantWriteError',
              'SSLSyscallError', 'CertificateError',
              'PROTOCOL_TLS', 'PROTOCOL_TLS_CLIENT',
              'PROTOCOL_TLS_SERVER',
              'CERT_NONE', 'CERT_OPTIONAL', 'CERT_REQUIRED',
              'VERIFY_DEFAULT', 'VERIFY_CRL_CHECK_LEAF',
              'VERIFY_CRL_CHECK_CHAIN', 'VERIFY_X509_STRICT',
              'VERIFY_X509_TRUSTED_FIRST',
              'OP_NO_SSLv3', 'OP_NO_TLSv1', 'OP_NO_TLSv1_1',
              'OP_NO_TLSv1_2', 'OP_NO_TLSv1_3',
              'OP_NO_COMPRESSION',
              'HAS_TLSv1', 'HAS_TLSv1_1', 'HAS_TLSv1_2',
              'HAS_TLSv1_3', 'HAS_ALPN',
              'OPENSSL_VERSION', 'OPENSSL_VERSION_NUMBER',
              'Purpose', 'create_default_context']:
    assert hasattr(ssl, _name); _ledger.append(1)

# Module name discipline
assert ssl.__name__ == 'ssl'; _ledger.append(1)

# create_default_context is callable
assert callable(ssl.create_default_context); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_ssl_verify_op_has_tls_flags_ops {sum(_ledger)} asserts")
