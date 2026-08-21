"""Behavior contract for third-party pyOpenSSL package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import OpenSSL  # type: ignore[import]
import OpenSSL.SSL  # type: ignore[import]
import OpenSSL.crypto  # type: ignore[import]
import OpenSSL.rand  # type: ignore[import]

# Rule 1: rand.status reports OpenSSL PRNG availability
_s1 = OpenSSL.rand.status()
assert isinstance(_s1, int), f"type = {type(_s1)!r}"
assert _s1 in (0, 1), f"status = {_s1!r}"

# Rule 2: X509 construction
_cert2 = OpenSSL.crypto.X509()
assert hasattr(_cert2, "get_subject"), "X509.get_subject"
assert hasattr(_cert2, "get_issuer"), "X509.get_issuer"
assert hasattr(_cert2, "get_serial_number"), "X509.get_serial_number"
assert hasattr(_cert2, "gmtime_adj_notBefore"), "X509.gmtime_adj_notBefore"

# Rule 3: PKey generation
_key3 = OpenSSL.crypto.PKey()
assert hasattr(_key3, "generate_key"), "PKey.generate_key"
assert callable(_key3.generate_key), "generate_key callable"
_key3.generate_key(OpenSSL.crypto.TYPE_RSA, 2048)
assert _key3.bits() == 2048, f"key bits = {_key3.bits()}"

# Rule 4: SSL.Context accepts method
_ctx4 = OpenSSL.SSL.Context(OpenSSL.SSL.TLS_METHOD)
assert hasattr(_ctx4, "use_certificate"), "ctx.use_certificate"
assert hasattr(_ctx4, "use_privatekey"), "ctx.use_privatekey"
assert hasattr(_ctx4, "set_verify"), "ctx.set_verify"

# Rule 5: __version__ is accessible
_v5 = OpenSSL.__version__
assert isinstance(_v5, str), f"version type = {type(_v5)!r}"
assert len(_v5) > 0, "version not empty"

# Rule 6: Module attributes are identity-stable
_ssl_ref = OpenSSL.SSL
_crypto_ref = OpenSSL.crypto
_v_ref = OpenSSL.version
_r_ref = OpenSSL.rand
for _ in range(5):
    assert OpenSSL.SSL is _ssl_ref, "SSL stable"
    assert OpenSSL.crypto is _crypto_ref, "crypto stable"
    assert OpenSSL.version is _v_ref, "version stable"
    assert OpenSSL.rand is _r_ref, "rand stable"

print("behavior OK")
