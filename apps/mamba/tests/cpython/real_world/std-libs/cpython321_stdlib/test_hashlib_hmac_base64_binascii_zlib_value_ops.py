# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_hashlib_hmac_base64_binascii_zlib_value_ops"
# subject = "cpython321.test_hashlib_hmac_base64_binascii_zlib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_hashlib_hmac_base64_binascii_zlib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_hashlib_hmac_base64_binascii_zlib_value_ops: execute CPython 3.12 seed test_hashlib_hmac_base64_binascii_zlib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 225 pass conformance — hashlib/hmac/base64/binascii/zlib/secrets/
# uuid/statistics/functools/operator value ops that match between CPython 3.12
# and mamba.
import hashlib
import hmac
import base64
import binascii
import zlib
import secrets
import uuid
import statistics
import functools
import operator
import math
import struct

_ledger: list[int] = []

# 1) hashlib — canonical digests for "abc"
assert hashlib.sha256(b"abc").hexdigest() == \
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; _ledger.append(1)
assert hashlib.sha1(b"abc").hexdigest() == \
    "a9993e364706816aba3e25717850c26c9cd0d89d"; _ledger.append(1)
assert hashlib.md5(b"abc").hexdigest() == \
    "900150983cd24fb0d6963f7d28e17f72"; _ledger.append(1)
assert hashlib.sha256(b"").digest_size == 32; _ledger.append(1)
assert hashlib.sha256(b"").name == "sha256"; _ledger.append(1)
assert hashlib.new("sha256", b"abc").hexdigest() == \
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; _ledger.append(1)

h = hashlib.sha256()
h.update(b"a")
h.update(b"bc")
assert h.hexdigest() == \
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; _ledger.append(1)

# 2) hmac
assert hmac.new(b"k", b"m", hashlib.sha256).hexdigest() == \
    "b60090e3052297aeb5a080889ce2fc4bca957e756faeb4df7d31800ca1e771ec"; _ledger.append(1)
assert hmac.digest(b"k", b"m", "sha256").hex() == \
    "b60090e3052297aeb5a080889ce2fc4bca957e756faeb4df7d31800ca1e771ec"; _ledger.append(1)
assert hmac.compare_digest(b"a", b"a") == True; _ledger.append(1)
assert hmac.compare_digest(b"a", b"b") == False; _ledger.append(1)
assert hmac.compare_digest("a", "a") == True; _ledger.append(1)

# 3) base64 — core encodings
assert base64.b64encode(b"abc") == b"YWJj"; _ledger.append(1)
assert base64.b64decode(b"YWJj") == b"abc"; _ledger.append(1)
assert base64.b32encode(b"abc") == b"MFRGG==="; _ledger.append(1)
assert base64.b32decode(b"MFRGG===") == b"abc"; _ledger.append(1)
assert base64.b16encode(b"abc") == b"616263"; _ledger.append(1)
assert base64.b16decode(b"616263") == b"abc"; _ledger.append(1)
assert base64.b64encode(b"hello world") == b"aGVsbG8gd29ybGQ="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8gd29ybGQ=") == b"hello world"; _ledger.append(1)

# 4) binascii — hex and base64 helpers
assert binascii.hexlify(b"abc") == b"616263"; _ledger.append(1)
assert binascii.unhexlify(b"616263") == b"abc"; _ledger.append(1)
assert binascii.b2a_hex(b"abc") == b"616263"; _ledger.append(1)
assert binascii.a2b_hex(b"616263") == b"abc"; _ledger.append(1)
assert binascii.b2a_base64(b"abc") == b"YWJj\n"; _ledger.append(1)
assert binascii.hexlify(b"\xde\xad\xbe\xef") == b"deadbeef"; _ledger.append(1)
assert binascii.unhexlify("deadbeef") == b"\xde\xad\xbe\xef"; _ledger.append(1)

# 5) zlib — roundtrip and checksums
assert zlib.decompress(zlib.compress(b"abc")) == b"abc"; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"hello world")) == b"hello world"; _ledger.append(1)
assert zlib.adler32(b"abc") == 38600999; _ledger.append(1)
assert zlib.crc32(b"abc") == 891568578; _ledger.append(1)
assert zlib.adler32(b"") == 1; _ledger.append(1)
assert zlib.crc32(b"") == 0; _ledger.append(1)

# 6) secrets — sizes, randomness invariants
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)
assert len(secrets.token_bytes(8)) == 8; _ledger.append(1)
assert len(secrets.token_urlsafe(8)) >= 8; _ledger.append(1)
assert secrets.randbelow(10) < 10; _ledger.append(1)
assert secrets.choice(["x"]) == "x"; _ledger.append(1)
assert secrets.compare_digest("a", "a") == True; _ledger.append(1)
assert secrets.compare_digest("a", "b") == False; _ledger.append(1)

# 7) uuid — round-trip and accessors
u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert u.hex == "12345678123456781234567812345678"; _ledger.append(1)
assert len(u.bytes) == 16; _ledger.append(1)
assert uuid.uuid4().version == 4; _ledger.append(1)
assert len(str(uuid.uuid4())) == 36; _ledger.append(1)

# 8) statistics — selectors that return integer-or-rounded equality on both
assert statistics.median([1, 2, 3]) == 2; _ledger.append(1)
assert statistics.median([1, 2, 3, 4]) == 2.5; _ledger.append(1)
assert statistics.median_low([1, 2, 3, 4]) == 2; _ledger.append(1)
assert statistics.median_high([1, 2, 3, 4]) == 3; _ledger.append(1)
assert statistics.mode([1, 2, 2, 3]) == 2; _ledger.append(1)
assert round(statistics.stdev([1, 2, 3]), 6) == 1.0; _ledger.append(1)
assert round(statistics.pstdev([1, 2, 3]), 6) == 0.816497; _ledger.append(1)
assert round(statistics.pvariance([1, 2, 3]), 6) == 0.666667; _ledger.append(1)
assert round(statistics.harmonic_mean([1, 2, 3]), 6) == 1.636364; _ledger.append(1)
assert round(statistics.geometric_mean([1, 2, 3]), 6) == 1.817121; _ledger.append(1)
assert statistics.fmean([1, 2, 3]) == 2.0; _ledger.append(1)

# 9) functools
assert functools.reduce(operator.add, [1, 2, 3, 4]) == 10; _ledger.append(1)
add10 = functools.partial(operator.add, 10)
assert add10(5) == 15; _ledger.append(1)

@functools.lru_cache(maxsize=2)
def _double(x):
    return x * 2

_r1 = _double(3)
_r2 = _double(3)
assert _r1 == 6; _ledger.append(1)
assert _r2 == 6; _ledger.append(1)

@functools.cache
def _plus1(x):
    return x + 1

_r3 = _plus1(5)
assert _r3 == 6; _ledger.append(1)

# 10) operator — basic numeric/comparison ops
assert operator.add(2, 3) == 5; _ledger.append(1)
assert operator.sub(5, 2) == 3; _ledger.append(1)
assert operator.mul(2, 3) == 6; _ledger.append(1)
assert operator.eq(2, 2) == True; _ledger.append(1)
assert operator.lt(1, 2) == True; _ledger.append(1)
assert operator.gt(3, 2) == True; _ledger.append(1)
assert operator.neg(5) == -5; _ledger.append(1)
assert operator.contains([1, 2, 3], 2) == True; _ledger.append(1)
assert operator.getitem([1, 2, 3], 1) == 2; _ledger.append(1)

# 11) math — number-theoretic and rounding
assert math.gcd(12, 18) == 6; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.perm(5, 2) == 20; _ledger.append(1)
assert math.isqrt(10) == 3; _ledger.append(1)
assert math.prod([1, 2, 3, 4]) == 24; _ledger.append(1)
assert math.floor(3.7) == 3; _ledger.append(1)
assert math.ceil(3.2) == 4; _ledger.append(1)
assert math.trunc(3.7) == 3; _ledger.append(1)
assert math.isnan(float("nan")) == True; _ledger.append(1)
assert math.isinf(float("inf")) == True; _ledger.append(1)
assert math.isfinite(1.0) == True; _ledger.append(1)

# 12) struct — calcsize agrees on canonical format strings
assert struct.calcsize(">I") == 4; _ledger.append(1)
assert struct.calcsize("<H") == 2; _ledger.append(1)
assert struct.calcsize("<Hq") == 10; _ledger.append(1)
assert struct.pack(">I", 1).hex() == "00000001"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_hashlib_hmac_base64_binascii_zlib_value_ops {sum(_ledger)} asserts")
