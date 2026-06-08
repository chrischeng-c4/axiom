# Atomic 294 pass conformance — hashlib module (hasattr md5/sha1/
# sha224/sha256/sha384/sha512/blake2b/blake2s/sha3_256/new/
# algorithms_available/algorithms_guaranteed + sha256(b'abc')
# .hexdigest()/digest_size + md5/sha1 hexdigest + sha512.digest_size
# + new('sha256') hexdigest) + hmac module (hasattr new/compare_
# digest/digest/HMAC + compare_digest True/False + new sha256
# hexdigest/digest_size/name) + secrets module (hasattr token_bytes/
# token_hex/token_urlsafe/choice/randbelow/randbits/compare_digest/
# SystemRandom + token_bytes bytes len 8 + token_hex str len 16) +
# uuid module (hasattr uuid1/uuid3/uuid4/uuid5/UUID/NAMESPACE_DNS/
# NAMESPACE_URL/NAMESPACE_OID/NAMESPACE_X500/RFC_4122/RESERVED_NCS +
# str(uuid4()) len 36) + base64 module (hasattr b64encode/b64decode/
# standard_b64encode/standard_b64decode/urlsafe_b64encode/urlsafe_
# b64decode/b32encode/b32decode/b16encode/b16decode/a85encode/
# a85decode/b85encode/b85decode/encodebytes/decodebytes + b64/b16
# round-trips + urlsafe/standard/b32/encodebytes/a85/b85 outputs) +
# math module (hasattr factorial/gcd/lcm/perm/comb/prod/isqrt +
# integer-returning operations).
# All asserts match between CPython 3.12 and mamba.
import hashlib
import hmac
import secrets
import uuid
import base64
import math


_ledger: list[int] = []

# 1) hashlib — hasattr core surface (conformant subset)
assert hasattr(hashlib, "md5") == True; _ledger.append(1)
assert hasattr(hashlib, "sha1") == True; _ledger.append(1)
assert hasattr(hashlib, "sha224") == True; _ledger.append(1)
assert hasattr(hashlib, "sha256") == True; _ledger.append(1)
assert hasattr(hashlib, "sha384") == True; _ledger.append(1)
assert hasattr(hashlib, "sha512") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2b") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2s") == True; _ledger.append(1)
assert hasattr(hashlib, "sha3_256") == True; _ledger.append(1)
assert hasattr(hashlib, "new") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_available") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_guaranteed") == True; _ledger.append(1)

# 2) hashlib — known-vector hexdigests
assert hashlib.sha256(b"abc").hexdigest() == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; _ledger.append(1)
assert hashlib.sha256(b"abc").digest_size == 32; _ledger.append(1)
assert hashlib.md5(b"abc").hexdigest() == "900150983cd24fb0d6963f7d28e17f72"; _ledger.append(1)
assert hashlib.sha1(b"abc").hexdigest() == "a9993e364706816aba3e25717850c26c9cd0d89d"; _ledger.append(1)
assert hashlib.sha512(b"abc").digest_size == 64; _ledger.append(1)
assert hashlib.new("sha256", b"abc").hexdigest() == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; _ledger.append(1)

# 3) hmac — hasattr core surface
assert hasattr(hmac, "new") == True; _ledger.append(1)
assert hasattr(hmac, "compare_digest") == True; _ledger.append(1)
assert hasattr(hmac, "digest") == True; _ledger.append(1)
assert hasattr(hmac, "HMAC") == True; _ledger.append(1)

# 4) hmac — value contracts
assert hmac.compare_digest(b"a", b"a") == True; _ledger.append(1)
assert hmac.compare_digest(b"a", b"b") == False; _ledger.append(1)
assert hmac.new(b"k", b"m", "sha256").hexdigest() == "b60090e3052297aeb5a080889ce2fc4bca957e756faeb4df7d31800ca1e771ec"; _ledger.append(1)
assert hmac.new(b"k", b"m", "sha256").digest_size == 32; _ledger.append(1)
assert hmac.new(b"k", b"m", "sha256").name == "hmac-sha256"; _ledger.append(1)

# 5) secrets — hasattr core surface
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "randbits") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)

# 6) secrets — value contracts (length-stable outputs)
assert isinstance(secrets.token_bytes(8), bytes) == True; _ledger.append(1)
assert len(secrets.token_bytes(8)) == 8; _ledger.append(1)
assert isinstance(secrets.token_hex(8), str) == True; _ledger.append(1)
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)

# 7) uuid — hasattr core surface
assert hasattr(uuid, "uuid1") == True; _ledger.append(1)
assert hasattr(uuid, "uuid3") == True; _ledger.append(1)
assert hasattr(uuid, "uuid4") == True; _ledger.append(1)
assert hasattr(uuid, "uuid5") == True; _ledger.append(1)
assert hasattr(uuid, "UUID") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_DNS") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_URL") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_OID") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_X500") == True; _ledger.append(1)
assert hasattr(uuid, "RFC_4122") == True; _ledger.append(1)
assert hasattr(uuid, "RESERVED_NCS") == True; _ledger.append(1)

# 8) uuid — len of str(uuid4())
assert len(str(uuid.uuid4())) == 36; _ledger.append(1)

# 9) base64 — hasattr core surface
assert hasattr(base64, "b64encode") == True; _ledger.append(1)
assert hasattr(base64, "b64decode") == True; _ledger.append(1)
assert hasattr(base64, "standard_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "standard_b64decode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64encode") == True; _ledger.append(1)
assert hasattr(base64, "urlsafe_b64decode") == True; _ledger.append(1)
assert hasattr(base64, "b32encode") == True; _ledger.append(1)
assert hasattr(base64, "b32decode") == True; _ledger.append(1)
assert hasattr(base64, "b16encode") == True; _ledger.append(1)
assert hasattr(base64, "b16decode") == True; _ledger.append(1)
assert hasattr(base64, "a85encode") == True; _ledger.append(1)
assert hasattr(base64, "a85decode") == True; _ledger.append(1)
assert hasattr(base64, "b85encode") == True; _ledger.append(1)
assert hasattr(base64, "b85decode") == True; _ledger.append(1)
assert hasattr(base64, "encodebytes") == True; _ledger.append(1)
assert hasattr(base64, "decodebytes") == True; _ledger.append(1)

# 10) base64 — value contracts
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.b16encode(b"hello") == b"68656C6C6F"; _ledger.append(1)
assert base64.b16decode(b"68656C6C6F") == b"hello"; _ledger.append(1)
assert base64.urlsafe_b64encode(b"\xfb\xff") == b"-_8="; _ledger.append(1)
assert base64.standard_b64encode(b"\xfb\xff") == b"+/8="; _ledger.append(1)
assert base64.b32encode(b"hi") == b"NBUQ===="; _ledger.append(1)
assert base64.encodebytes(b"hi") == b"aGk=\n"; _ledger.append(1)
assert base64.a85encode(b"hi") == b"BP@"; _ledger.append(1)
assert base64.b85encode(b"hi") == b"XlV"; _ledger.append(1)

# 11) math — hasattr integer-returning operations
assert hasattr(math, "factorial") == True; _ledger.append(1)
assert hasattr(math, "gcd") == True; _ledger.append(1)
assert hasattr(math, "lcm") == True; _ledger.append(1)
assert hasattr(math, "perm") == True; _ledger.append(1)
assert hasattr(math, "comb") == True; _ledger.append(1)
assert hasattr(math, "prod") == True; _ledger.append(1)
assert hasattr(math, "isqrt") == True; _ledger.append(1)

# 12) math — integer-arithmetic value contracts (avoid float-returning ops)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.gcd(12, 8) == 4; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.perm(5, 2) == 20; _ledger.append(1)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.prod([1, 2, 3]) == 6; _ledger.append(1)
assert math.isqrt(10) == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_hashlib_hmac_secrets_uuid_base64_value_ops {sum(_ledger)} asserts")
