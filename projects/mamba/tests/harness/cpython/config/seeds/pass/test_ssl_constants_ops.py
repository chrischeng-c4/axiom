# Operational AssertionPass seed for `ssl` protocol + cert-verify
# constants.
# Surface: PROTOCOL_TLS_CLIENT (=16), PROTOCOL_TLS_SERVER (=17),
# CERT_NONE (=0), CERT_OPTIONAL (=1), CERT_REQUIRED (=2). The TLS
# protocol values changed across OpenSSL releases historically; this
# fixture pins the CPython 3.10+ documented surface that mamba
# matches.
# Companion to stub/test_ssl.py — vendored unittest seed.
import ssl
_ledger: list[int] = []
# Cert-verify modes
assert ssl.CERT_NONE == 0; _ledger.append(1)
assert ssl.CERT_OPTIONAL == 1; _ledger.append(1)
assert ssl.CERT_REQUIRED == 2; _ledger.append(1)
# Strict ordering on the cert-verify axis (none < optional < required)
assert ssl.CERT_NONE < ssl.CERT_OPTIONAL; _ledger.append(1)
assert ssl.CERT_OPTIONAL < ssl.CERT_REQUIRED; _ledger.append(1)
# TLS purpose constants
assert ssl.PROTOCOL_TLS_CLIENT == 16; _ledger.append(1)
assert ssl.PROTOCOL_TLS_SERVER == 17; _ledger.append(1)
# Client and server purposes are distinct adjacent values
assert ssl.PROTOCOL_TLS_CLIENT != ssl.PROTOCOL_TLS_SERVER; _ledger.append(1)
assert ssl.PROTOCOL_TLS_SERVER - ssl.PROTOCOL_TLS_CLIENT == 1; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_ssl_constants_ops {sum(_ledger)} asserts")
