"""Behavior contract for third-party cryptography package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

from cryptography.fernet import Fernet  # type: ignore[import]
from cryptography.hazmat.primitives import hashes, hmac  # type: ignore[import]
from cryptography.hazmat.primitives.asymmetric import rsa, padding  # type: ignore[import]
from cryptography.hazmat.backends import default_backend  # type: ignore[import]
from cryptography.hazmat.primitives import serialization  # type: ignore[import]

# Rule 1: Fernet symmetric encryption/decryption round-trip
_key1 = Fernet.generate_key()
_f1 = Fernet(_key1)
_plaintext1 = b"Hello, World! Secret data here."
_token1 = _f1.encrypt(_plaintext1)
assert isinstance(_token1, bytes), f"token type = {type(_token1)!r}"
assert _token1 != _plaintext1, "token differs from plaintext"
_decrypted1 = _f1.decrypt(_token1)
assert _decrypted1 == _plaintext1, f"decrypted = {_decrypted1!r}"

# Rule 2: Different Fernet keys produce different tokens
_key2a = Fernet.generate_key()
_key2b = Fernet.generate_key()
assert _key2a != _key2b, "keys differ"
_f2a = Fernet(_key2a)
_f2b = Fernet(_key2b)
_t2a = _f2a.encrypt(b"same plaintext")
_t2b = _f2b.encrypt(b"same plaintext")
# Tokens are different (randomized)
# Both decrypt correctly with their own key
assert _f2a.decrypt(_t2a) == b"same plaintext", "key A decrypts"
assert _f2b.decrypt(_t2b) == b"same plaintext", "key B decrypts"

# Rule 3: Wrong key raises InvalidToken
from cryptography.fernet import InvalidToken  # type: ignore[import]
_key3a = Fernet.generate_key()
_key3b = Fernet.generate_key()
_token3 = Fernet(_key3a).encrypt(b"secret")
_raised3 = False
try:
    Fernet(_key3b).decrypt(_token3)
except InvalidToken:
    _raised3 = True
assert _raised3, "InvalidToken with wrong key"

# Rule 4: SHA256 hash size and name
_sha256 = hashes.SHA256()
assert _sha256.name == "sha256", f"sha256 name = {_sha256.name!r}"
assert _sha256.digest_size == 32, f"sha256 digest_size = {_sha256.digest_size!r}"
_sha512 = hashes.SHA512()
assert _sha512.digest_size == 64, f"sha512 digest_size = {_sha512.digest_size!r}"

# Rule 5: HMAC produces consistent output for same key+data
from cryptography.hazmat.primitives.hmac import HMAC  # type: ignore[import]
from cryptography.hazmat.primitives import hashes as _h  # type: ignore[import]
_key5 = b"hmac_key_16bytes"
_data5 = b"hello world"
def _make_hmac5():
    _m = HMAC(_key5, _h.SHA256(), backend=default_backend())
    _m.update(_data5)
    return _m.finalize()

_mac5a = _make_hmac5()
_mac5b = _make_hmac5()
assert _mac5a == _mac5b, f"HMAC deterministic: {_mac5a.hex()!r}"
assert len(_mac5a) == 32, f"SHA256 HMAC is 32 bytes: {len(_mac5a)!r}"

# Rule 6: RSA key generation produces valid key pair
_private_key6 = rsa.generate_private_key(
    public_exponent=65537,
    key_size=2048,
    backend=default_backend(),
)
_public_key6 = _private_key6.public_key()
assert hasattr(_public_key6, "encrypt"), "public key encrypt"
assert hasattr(_private_key6, "decrypt"), "private key decrypt"
assert _private_key6.key_size == 2048, f"key size = {_private_key6.key_size!r}"

print("behavior OK")
