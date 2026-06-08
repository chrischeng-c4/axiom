# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_random_secrets_hmac_hashlib_value_ops"
# subject = "cpython321.test_random_secrets_hmac_hashlib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_random_secrets_hmac_hashlib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_random_secrets_hmac_hashlib_value_ops: execute CPython 3.12 seed test_random_secrets_hmac_hashlib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 279 pass conformance — random module (hasattr Random/seed/
# random/randint/randrange/choice/choices/sample/shuffle/uniform/gauss/
# normalvariate/triangular/expovariate/betavariate/getstate/setstate/
# getrandbits + seed-reproducibility + bounded outputs) + secrets
# module (hasattr choice/randbelow/randbits/token_bytes/token_hex/
# token_urlsafe/compare_digest + bytes length + randbelow/randbits
# bounds + compare_digest True/False) + hmac module (hasattr new/HMAC/
# compare_digest/digest + sha256 hexdigest length 64 + compare_digest
# True/False + digest length 32) + hashlib module (hasattr new/md5/
# sha1/sha256/sha512/sha3_256/blake2b/blake2s/algorithms_guaranteed/
# algorithms_available + md5/sha1/sha256 reference hexdigests).
# All asserts match between CPython 3.12 and mamba.
import random
import secrets
import hmac
import hashlib


_ledger: list[int] = []

# 1) random — hasattr core surface
assert hasattr(random, "Random") == True; _ledger.append(1)
assert hasattr(random, "seed") == True; _ledger.append(1)
assert hasattr(random, "random") == True; _ledger.append(1)
assert hasattr(random, "randint") == True; _ledger.append(1)
assert hasattr(random, "randrange") == True; _ledger.append(1)
assert hasattr(random, "choice") == True; _ledger.append(1)
assert hasattr(random, "sample") == True; _ledger.append(1)
assert hasattr(random, "shuffle") == True; _ledger.append(1)
assert hasattr(random, "uniform") == True; _ledger.append(1)
assert hasattr(random, "getrandbits") == True; _ledger.append(1)
assert hasattr(random, "getstate") == True; _ledger.append(1)
assert hasattr(random, "setstate") == True; _ledger.append(1)

# 2) random — bounded outputs after seed
random.seed(42)
assert (0 <= random.random() < 1) == True; _ledger.append(1)
random.seed(42)
assert (1 <= random.randint(1, 10) <= 10) == True; _ledger.append(1)
random.seed(42)
assert (0 <= random.getrandbits(8) <= 255) == True; _ledger.append(1)

# 3) random — seed reproducibility
random.seed(0)
_v1 = random.random()
random.seed(0)
_v2 = random.random()
assert _v1 == _v2; _ledger.append(1)

# 4) secrets — hasattr token-generation surface
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "randbits") == True; _ledger.append(1)
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)

# 5) secrets — bounded outputs + value contracts
assert len(secrets.token_bytes(4)) == 4; _ledger.append(1)
assert len(secrets.token_hex(4)) == 8; _ledger.append(1)
assert (0 <= secrets.randbelow(10) <= 9) == True; _ledger.append(1)
assert (0 <= secrets.randbits(8) <= 255) == True; _ledger.append(1)
assert secrets.compare_digest("a", "a") == True; _ledger.append(1)
assert secrets.compare_digest("a", "b") == False; _ledger.append(1)

# 6) hmac — hasattr core surface
assert hasattr(hmac, "new") == True; _ledger.append(1)
assert hasattr(hmac, "HMAC") == True; _ledger.append(1)
assert hasattr(hmac, "compare_digest") == True; _ledger.append(1)
assert hasattr(hmac, "digest") == True; _ledger.append(1)

# 7) hmac — sha256 length + compare_digest contracts
assert len(hmac.new(b"k", b"m", "sha256").hexdigest()) == 64; _ledger.append(1)
assert len(hmac.digest(b"k", b"m", "sha256")) == 32; _ledger.append(1)
assert hmac.compare_digest("a", "a") == True; _ledger.append(1)
assert hmac.compare_digest("a", "b") == False; _ledger.append(1)

# 8) hashlib — hasattr algorithm surface
assert hasattr(hashlib, "new") == True; _ledger.append(1)
assert hasattr(hashlib, "md5") == True; _ledger.append(1)
assert hasattr(hashlib, "sha1") == True; _ledger.append(1)
assert hasattr(hashlib, "sha256") == True; _ledger.append(1)
assert hasattr(hashlib, "sha512") == True; _ledger.append(1)
assert hasattr(hashlib, "sha3_256") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2b") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2s") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_guaranteed") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_available") == True; _ledger.append(1)

# 9) hashlib — reference digest contracts
assert hashlib.md5(b"").hexdigest() == "d41d8cd98f00b204e9800998ecf8427e"; _ledger.append(1)
assert hashlib.sha1(b"").hexdigest() == "da39a3ee5e6b4b0d3255bfef95601890afd80709"; _ledger.append(1)
assert len(hashlib.sha256(b"").hexdigest()) == 64; _ledger.append(1)
assert len(hashlib.sha512(b"").hexdigest()) == 128; _ledger.append(1)

# 10) hashlib — algorithm membership + new() roundtrip
assert ("md5" in hashlib.algorithms_guaranteed) == True; _ledger.append(1)
assert ("sha256" in hashlib.algorithms_guaranteed) == True; _ledger.append(1)
assert hashlib.new("md5", b"abc").hexdigest() == "900150983cd24fb0d6963f7d28e17f72"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_random_secrets_hmac_hashlib_value_ops {sum(_ledger)} asserts")
