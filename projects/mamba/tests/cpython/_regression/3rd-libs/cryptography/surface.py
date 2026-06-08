"""Surface contract for third-party cryptography package.

# type-regime: monomorphic

Probes: cryptography.fernet.Fernet, cryptography.hazmat.primitives.hashes,
cryptography.hazmat.primitives.hmac, cryptography.hazmat.primitives.asymmetric,
cryptography.hazmat.backends.default_backend.
CPython 3.12 is the oracle.
"""

import cryptography
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes, hmac
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.backends import default_backend

# Version exists
assert hasattr(cryptography, "__version__"), "__version__"

# Fernet
assert hasattr(Fernet, "generate_key"), "Fernet.generate_key"
_key = Fernet.generate_key()
assert isinstance(_key, bytes), f"key type = {type(_key)!r}"
assert len(_key) == 44, f"key len = {len(_key)!r}"  # base64url-encoded 32 bytes

_f = Fernet(_key)
assert hasattr(_f, "encrypt"), "Fernet.encrypt"
assert hasattr(_f, "decrypt"), "Fernet.decrypt"

# Encrypt/decrypt round-trip
_token = _f.encrypt(b"secret message")
assert isinstance(_token, bytes), f"token type = {type(_token)!r}"
_plain = _f.decrypt(_token)
assert _plain == b"secret message", f"decrypted = {_plain!r}"

# Hashes
assert hasattr(hashes, "SHA256"), "hashes.SHA256"
assert hasattr(hashes, "SHA512"), "hashes.SHA512"
assert hasattr(hashes, "MD5"), "hashes.MD5"
assert hasattr(hashes, "SHA1"), "hashes.SHA1"

_sha256 = hashes.SHA256()
assert _sha256.name == "sha256", f"sha256 name = {_sha256.name!r}"
assert _sha256.digest_size == 32, f"sha256 digest_size = {_sha256.digest_size!r}"

_sha512 = hashes.SHA512()
assert _sha512.digest_size == 64, f"sha512 digest_size = {_sha512.digest_size!r}"

# HMAC
from cryptography.hazmat.primitives import hashes as _h
from cryptography.hazmat.backends import default_backend as _db
_hmac = hmac.HMAC(b"key_bytes_here!!", _h.SHA256(), backend=_db())
assert hasattr(_hmac, "update"), "hmac.update"
assert hasattr(_hmac, "finalize"), "hmac.finalize"
assert hasattr(_hmac, "copy"), "hmac.copy"

# RSA key generation
assert hasattr(rsa, "generate_private_key"), "rsa.generate_private_key"

# Module attributes stable
_fernet_ref = Fernet
assert Fernet is _fernet_ref, "Fernet stable"

print("surface OK")
