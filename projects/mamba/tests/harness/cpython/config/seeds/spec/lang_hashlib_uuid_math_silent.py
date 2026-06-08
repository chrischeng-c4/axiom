# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(hashlib, 'shake_128')` (the
# documented "hashlib exposes the SHAKE-128 variable-length hash" —
# mamba returns False), `hasattr(hashlib, 'pbkdf2_hmac')` (the
# documented "hashlib exposes the PBKDF2-HMAC key-derivation helper"
# — mamba returns False), `hasattr(hashlib, 'scrypt')` (the
# documented "hashlib exposes the scrypt KDF helper" — mamba returns
# False), `hashlib.sha256.__name__ == 'openssl_sha256'` (the
# documented "hashlib.sha256 is a callable whose __name__ is its
# openssl identifier" — mamba returns None — attribute resolves to
# None placeholder), `hasattr(random, 'SystemRandom')` (the
# documented "random exposes the SystemRandom os.urandom-backed RNG
# class" — mamba returns False), `math.sqrt(4) == 2.0` (the
# documented "math.sqrt returns a Python float" — mamba returns
# 4611686018427387904 — IEEE-754 bit pattern of 2.0 as int),
# `math.cbrt(27) == 3.0` (the documented "math.cbrt returns a
# Python float" — mamba returns 4613937818241073152 — IEEE-754 bit
# pattern of 3.0 as int), `math.hypot(3, 4) == 5.0` (the documented
# "math.hypot returns a Python float" — mamba returns
# 4617315517961601024 — IEEE-754 bit pattern of 5.0 as int),
# `str(uuid.UUID('12345678-1234-5678-1234-567812345678')) ==
# '12345678-1234-5678-1234-567812345678'` (the documented "str(UUID)
# emits the canonical 8-4-4-4-12 hex form" — mamba returns
# '2199023255556' — bare integer handle), and `type(uuid.NAMESPACE_
# DNS).__name__ == 'UUID'` (the documented "uuid.NAMESPACE_DNS is a
# UUID instance" — mamba returns 'int' — handle-typed sentinel).
# Ten-pack pinned to atomic 294.
#
# Behavioral edges that CONFORM on mamba (hashlib — hasattr md5/
# sha1/sha224/sha256/sha384/sha512/blake2b/blake2s/sha3_256/new/
# algorithms_available/algorithms_guaranteed + sha256/md5/sha1
# hexdigest + sha512.digest_size + new. hmac — hasattr new/compare_
# digest/digest/HMAC + compare_digest + sha256 hexdigest/digest_
# size/name. secrets — hasattr token_bytes/token_hex/token_urlsafe/
# choice/randbelow/randbits/compare_digest/SystemRandom + token_
# bytes len + token_hex len. uuid — hasattr uuid1/uuid3/uuid4/uuid5/
# UUID/NAMESPACE_DNS/NAMESPACE_URL/NAMESPACE_OID/NAMESPACE_X500/RFC_
# 4122/RESERVED_NCS + len str(uuid4()). base64 — hasattr b64/b16/
# urlsafe/standard/b32/a85/b85/encodebytes/decodebytes + b64/b16
# round-trips. math — hasattr factorial/gcd/lcm/perm/comb/prod/
# isqrt + integer-returning ops) are covered in the matching pass
# fixture `test_hashlib_hmac_secrets_uuid_base64_value_ops`.
import hashlib
import random
import math
import uuid


_ledger: list[int] = []

# 1) hasattr(hashlib, 'shake_128') — SHAKE-128 variable-length hash
#    (mamba: returns False)
assert hasattr(hashlib, "shake_128") == True; _ledger.append(1)

# 2) hasattr(hashlib, 'pbkdf2_hmac') — PBKDF2-HMAC KDF helper
#    (mamba: returns False)
assert hasattr(hashlib, "pbkdf2_hmac") == True; _ledger.append(1)

# 3) hasattr(hashlib, 'scrypt') — scrypt KDF helper
#    (mamba: returns False)
assert hasattr(hashlib, "scrypt") == True; _ledger.append(1)

# 4) hashlib.sha256.__name__ == 'openssl_sha256' — callable identifier
#    (mamba: attribute resolves to None placeholder)
assert hashlib.sha256.__name__ == "openssl_sha256"; _ledger.append(1)

# 5) hasattr(random, 'SystemRandom') — os.urandom-backed RNG class
#    (mamba: returns False)
assert hasattr(random, "SystemRandom") == True; _ledger.append(1)

# 6) math.sqrt(4) == 2.0 — float-returning square root
#    (mamba: returns 4611686018427387904 — i64 bit pattern of 2.0)
assert math.sqrt(4) == 2.0; _ledger.append(1)

# 7) math.cbrt(27) == 3.0 — float-returning cube root
#    (mamba: returns 4613937818241073152 — i64 bit pattern of 3.0)
assert math.cbrt(27) == 3.0; _ledger.append(1)

# 8) math.hypot(3, 4) == 5.0 — float-returning euclidean norm
#    (mamba: returns 4617315517961601024 — i64 bit pattern of 5.0)
assert math.hypot(3, 4) == 5.0; _ledger.append(1)

# 9) str(uuid.UUID('12345678-1234-5678-1234-567812345678')) — canonical 8-4-4-4-12 hex
#    (mamba: returns '2199023255556' — bare integer handle)
assert str(uuid.UUID("12345678-1234-5678-1234-567812345678")) == "12345678-1234-5678-1234-567812345678"; _ledger.append(1)

# 10) type(uuid.NAMESPACE_DNS).__name__ == 'UUID' — UUID-typed sentinel
#     (mamba: returns 'int' — handle-typed sentinel)
assert type(uuid.NAMESPACE_DNS).__name__ == "UUID"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_hashlib_uuid_math_silent {sum(_ledger)} asserts")
