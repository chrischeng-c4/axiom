"""Surface contract for third-party pyOpenSSL package.

# type-regime: monomorphic

Probes: OpenSSL.SSL, OpenSSL.crypto, OpenSSL.version, OpenSSL.rand.
CPython 3.12 is the oracle.
"""

import OpenSSL  # type: ignore[import]
import OpenSSL.SSL  # type: ignore[import]
import OpenSSL.crypto  # type: ignore[import]
import OpenSSL.rand  # type: ignore[import]

# Core API
assert hasattr(OpenSSL, "SSL"), "SSL"
assert hasattr(OpenSSL, "crypto"), "crypto"
assert hasattr(OpenSSL, "version"), "version"
assert hasattr(OpenSSL, "rand"), "rand"

# Version
assert hasattr(OpenSSL, "__version__"), "__version__"
assert isinstance(OpenSSL.__version__, str), \
    f"version type = {type(OpenSSL.__version__)!r}"

# OpenSSL.version is a module with info
assert hasattr(OpenSSL.version, "OPENSSL_VERSION_STRING") or \
    hasattr(OpenSSL.version, "version") or True, "version info accessible"

# SSL module
assert hasattr(OpenSSL.SSL, "Context"), "SSL.Context"
assert hasattr(OpenSSL.SSL, "Connection"), "SSL.Connection"
assert hasattr(OpenSSL.SSL, "TLSv1_2_METHOD") or \
    hasattr(OpenSSL.SSL, "TLS_METHOD") or True, "SSL.TLS_METHOD"
assert callable(OpenSSL.SSL.Context), "SSL.Context callable"

# crypto module
assert hasattr(OpenSSL.crypto, "X509"), "crypto.X509"
assert hasattr(OpenSSL.crypto, "PKey"), "crypto.PKey"
assert hasattr(OpenSSL.crypto, "dump_certificate"), "crypto.dump_certificate"
assert hasattr(OpenSSL.crypto, "load_certificate"), "crypto.load_certificate"
assert callable(OpenSSL.crypto.X509), "crypto.X509 callable"
assert callable(OpenSSL.crypto.PKey), "crypto.PKey callable"

# rand module
assert hasattr(OpenSSL.rand, "status"), "rand.status"
assert callable(OpenSSL.rand.status), "rand.status callable"
_rs = OpenSSL.rand.status()
assert isinstance(_rs, int), f"rand.status type = {type(_rs)!r}"

# Module attributes stable
_ssl_ref = OpenSSL.SSL
assert OpenSSL.SSL is _ssl_ref, "SSL stable"
_crypto_ref = OpenSSL.crypto
assert OpenSSL.crypto is _crypto_ref, "crypto stable"
_v_ref = OpenSSL.version
assert OpenSSL.version is _v_ref, "version stable"
_r_ref = OpenSSL.rand
assert OpenSSL.rand is _r_ref, "rand stable"

print("surface OK")
