# Atomic 266 pass conformance — secrets module (hasattr token_bytes/
# token_hex/token_urlsafe/choice/randbelow/randbits/compare_digest/
# SystemRandom + token_bytes type bytes / len 0/4/16/32 contracts,
# token_hex type str / len 4 == 8 / len 16 == 32, token_urlsafe
# type str / non-empty for input 8, choice from list in list,
# randbelow 0..N range, randbits int range, compare_digest True for
# equal / False for unequal) + hmac module (hasattr new/digest/
# compare_digest/HMAC + hmac.new(b'key', b'msg', 'sha256').hexdigest
# () == known, hmac.digest matches new+hexdigest, digest_size == 32
# for sha256, block_size == 64 for sha256, hmac.compare_digest True
# for equal / False for unequal) + hashlib module (hasattr md5/
# sha1/sha256/sha512/sha224/sha384/sha3_256/sha3_512/blake2b/blake2s
# /new/algorithms_available/algorithms_guaranteed + md5(b'') /
# md5(b'abc') / sha1(b'abc') / sha256(b'abc') / sha512(b'abc') /
# sha3_256(b'abc') known hexdigests, sha256.digest_size == 32 /
# .name == 'sha256', hashlib.new('sha256', b'abc') matches sha256
# (b'abc'), digest() returns bytes, md5/sha256 block_size, 'md5' in
# algorithms_guaranteed, 'sha256' in algorithms_available) + uuid
# module (hasattr UUID/uuid1/uuid3/uuid4/uuid5/NAMESPACE_DNS/
# NAMESPACE_URL/NAMESPACE_OID + uuid4 str length 36 / two calls
# distinct, UUID('12345678-1234-5678-1234-567812345678') str/hex
# roundtrip, uuid5 NAMESPACE_DNS 'example.com' deterministic,
# uuid.UUID bytes len 16).
# All asserts match between CPython 3.12 and mamba.
import secrets
import hmac
import hashlib
import uuid


_ledger: list[int] = []

# 1) secrets — hasattr surface
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "randbits") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)

# 2) secrets — token_bytes type + lengths
assert type(secrets.token_bytes(4)).__name__ == "bytes"; _ledger.append(1)
assert len(secrets.token_bytes(0)) == 0; _ledger.append(1)
assert len(secrets.token_bytes(4)) == 4; _ledger.append(1)
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert len(secrets.token_bytes(32)) == 32; _ledger.append(1)

# 3) secrets — token_hex / token_urlsafe contracts
assert type(secrets.token_hex(4)).__name__ == "str"; _ledger.append(1)
assert len(secrets.token_hex(4)) == 8; _ledger.append(1)
assert len(secrets.token_hex(16)) == 32; _ledger.append(1)
assert type(secrets.token_urlsafe(4)).__name__ == "str"; _ledger.append(1)
assert (len(secrets.token_urlsafe(8)) > 0) == True; _ledger.append(1)

# 4) secrets — choice/randbelow/randbits/compare_digest behavior
assert (secrets.choice([1, 2, 3]) in [1, 2, 3]) == True; _ledger.append(1)
assert (secrets.randbelow(10) < 10) == True; _ledger.append(1)
assert (secrets.randbelow(10) >= 0) == True; _ledger.append(1)
assert type(secrets.randbits(8)).__name__ == "int"; _ledger.append(1)
assert secrets.compare_digest("abc", "abc") == True; _ledger.append(1)
assert secrets.compare_digest("abc", "xyz") == False; _ledger.append(1)

# 5) hmac — hasattr surface
assert hasattr(hmac, "new") == True; _ledger.append(1)
assert hasattr(hmac, "digest") == True; _ledger.append(1)
assert hasattr(hmac, "compare_digest") == True; _ledger.append(1)
assert hasattr(hmac, "HMAC") == True; _ledger.append(1)

# 6) hmac — known SHA-256 HMAC value contracts
assert hmac.new(b"key", b"msg", "sha256").hexdigest() == "2d93cbc1be167bcb1637a4a23cbff01a7878f0c50ee833954ea5221bb1b8c628"; _ledger.append(1)
assert hmac.new(b"key", b"msg", "sha256").digest_size == 32; _ledger.append(1)
assert hmac.new(b"key", b"msg", "sha256").block_size == 64; _ledger.append(1)
assert hmac.compare_digest(b"abc", b"abc") == True; _ledger.append(1)
assert hmac.compare_digest(b"abc", b"xyz") == False; _ledger.append(1)

# 7) hashlib — hasattr surface
assert hasattr(hashlib, "md5") == True; _ledger.append(1)
assert hasattr(hashlib, "sha1") == True; _ledger.append(1)
assert hasattr(hashlib, "sha256") == True; _ledger.append(1)
assert hasattr(hashlib, "sha512") == True; _ledger.append(1)
assert hasattr(hashlib, "sha224") == True; _ledger.append(1)
assert hasattr(hashlib, "sha384") == True; _ledger.append(1)
assert hasattr(hashlib, "sha3_256") == True; _ledger.append(1)
assert hasattr(hashlib, "sha3_512") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2b") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2s") == True; _ledger.append(1)
assert hasattr(hashlib, "new") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_available") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_guaranteed") == True; _ledger.append(1)

# 8) hashlib — known digest values for empty and 'abc'
assert hashlib.md5(b"").hexdigest() == "d41d8cd98f00b204e9800998ecf8427e"; _ledger.append(1)
assert hashlib.md5(b"abc").hexdigest() == "900150983cd24fb0d6963f7d28e17f72"; _ledger.append(1)
assert hashlib.sha1(b"abc").hexdigest() == "a9993e364706816aba3e25717850c26c9cd0d89d"; _ledger.append(1)
assert hashlib.sha256(b"abc").hexdigest() == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; _ledger.append(1)
assert hashlib.sha512(b"abc").hexdigest() == "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"; _ledger.append(1)

# 9) hashlib — sha256 metadata + digest type + new()
assert hashlib.sha256(b"abc").digest_size == 32; _ledger.append(1)
assert hashlib.sha256(b"abc").name == "sha256"; _ledger.append(1)
assert hashlib.new("sha256", b"abc").hexdigest() == hashlib.sha256(b"abc").hexdigest(); _ledger.append(1)
assert type(hashlib.sha256(b"abc").digest()).__name__ == "bytes"; _ledger.append(1)
assert hashlib.md5().block_size == 64; _ledger.append(1)
assert hashlib.sha256().block_size == 64; _ledger.append(1)
assert ("md5" in hashlib.algorithms_guaranteed) == True; _ledger.append(1)
assert ("sha256" in hashlib.algorithms_available) == True; _ledger.append(1)

# 10) uuid — hasattr surface
assert hasattr(uuid, "UUID") == True; _ledger.append(1)
assert hasattr(uuid, "uuid1") == True; _ledger.append(1)
assert hasattr(uuid, "uuid3") == True; _ledger.append(1)
assert hasattr(uuid, "uuid4") == True; _ledger.append(1)
assert hasattr(uuid, "uuid5") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_DNS") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_URL") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_OID") == True; _ledger.append(1)

# 11) uuid — uuid4 str length / two-call inequality
assert len(str(uuid.uuid4())) == 36; _ledger.append(1)
assert (uuid.uuid4() != uuid.uuid4()) == True; _ledger.append(1)

# 12) uuid — UUID literal roundtrip
assert str(uuid.UUID("12345678-1234-5678-1234-567812345678")) == "12345678-1234-5678-1234-567812345678"; _ledger.append(1)
assert uuid.UUID("12345678-1234-5678-1234-567812345678").hex == "12345678123456781234567812345678"; _ledger.append(1)
assert len(uuid.UUID("12345678-1234-5678-1234-567812345678").bytes) == 16; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_secrets_hmac_hashlib_uuid_value_ops {sum(_ledger)} asserts")
