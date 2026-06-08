# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_hashlib_hmac_secrets_uuid_random_value_ops"
# subject = "cpython321.test_hashlib_hmac_secrets_uuid_random_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_hashlib_hmac_secrets_uuid_random_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_hashlib_hmac_secrets_uuid_random_value_ops: execute CPython 3.12 seed test_hashlib_hmac_secrets_uuid_random_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `hashlib` / `hmac` / `secrets` / `uuid` / `random` five-pack
# pinned to atomic 188: `hashlib` (the documented full module-
# level helper hasattr surface — `sha256` / `sha1` / `md5` /
# `sha512` / `sha224` / `sha384` / `blake2b` / `blake2s` /
# `new` / `algorithms_guaranteed` / `algorithms_available` +
# the documented hashlib.sha256 / sha1 / md5 hexdigest value
# contract + the documented sha256 digest_size attribute
# contract), `hmac` (the documented full module-level helper
# hasattr surface — `new` / `compare_digest` / `digest` /
# `HMAC` + the documented hmac.new hexdigest value contract +
# the documented hmac.compare_digest True / False value
# contract), `secrets` (the documented full module-level
# helper hasattr surface — `token_bytes` / `token_hex` /
# `token_urlsafe` / `choice` / `randbelow` / `randbits` /
# `compare_digest` / `SystemRandom` + the documented
# secrets.token_bytes / token_hex return-type and length
# contract), `uuid` (the documented full module-level helper
# hasattr surface — `uuid1` / `uuid3` / `uuid4` / `uuid5` /
# `UUID` / `NAMESPACE_DNS` / `NAMESPACE_URL` + the documented
# UUID round-trip string-value contract + the documented
# uuid4 .hex / str length contract), and `random` (the
# documented full module-level helper hasattr surface —
# `random` / `randint` / `choice` / `shuffle` / `sample` /
# `Random` / `seed` / `uniform` / `gauss` / `randrange` +
# the documented random.random in [0, 1) value contract +
# the documented random.seed determinism contract).
#
# The matching subset between mamba and CPython is the full
# `hashlib` module hasattr surface + the hexdigest value
# layer + the digest_size attribute layer (the `type(sha256
# (b"hello")).__name__ == "HASH"` class-identity layer
# DIVERGES — mamba returns `"int"`), the full `hmac` module
# hasattr surface + the hexdigest value layer + the
# compare_digest value layer (the `type(hmac.new(...))
# .__name__ == "HMAC"` class-identity layer DIVERGES —
# mamba returns `"int"`), the full `secrets` module hasattr
# surface + the token_bytes / token_hex return-type and
# length value layer, the full `uuid` module hasattr surface
# + the UUID round-trip string value layer + the .hex / str
# length contract (the `type(uuid.uuid4()).__name__ ==
# "UUID"` class-identity layer DIVERGES — mamba returns
# `"int"`), and the full `random` module hasattr surface +
# the random.random in [0, 1) value layer + the seed
# determinism contract (the Mersenne-Twister stable-sequence
# `random.seed(42); random.randint(1, 10) == 2` value layer
# DIVERGES — mamba returns 10).
#
# Surface in this fixture:
#   • hashlib — full module hasattr surface (sha256 / sha1
#     / md5 / sha512 / sha224 / sha384 / blake2b / blake2s
#     / new / algorithms_guaranteed / algorithms_available);
#   • hashlib.sha256 / sha1 / md5 — hexdigest value
#     contract;
#   • hashlib.sha256 — digest_size attribute contract;
#   • hmac — full module hasattr surface (new /
#     compare_digest / digest / HMAC);
#   • hmac.new — hexdigest value contract;
#   • hmac.compare_digest — True / False value contract;
#   • secrets — full module hasattr surface (token_bytes /
#     token_hex / token_urlsafe / choice / randbelow /
#     randbits / compare_digest / SystemRandom);
#   • secrets.token_bytes / token_hex — return-type and
#     length value contract;
#   • uuid — full module hasattr surface (uuid1 / uuid3 /
#     uuid4 / uuid5 / UUID / NAMESPACE_DNS / NAMESPACE_URL);
#   • uuid.UUID — round-trip string value contract;
#   • uuid.uuid4 — .hex / str length contract;
#   • random — full module hasattr surface (random / randint
#     / choice / shuffle / sample / Random / seed / uniform
#     / gauss / randrange);
#   • random.random — value-in-range contract;
#   • random.seed — determinism contract (same seed → same
#     randint result twice in a row).
#
# Behavioral edges that DIVERGE on mamba
# (type(hashlib.sha256(b"hello")).__name__ returns "int" not
# "HASH", type(hmac.new(...)).__name__ returns "int" not
# "HMAC", type(uuid.uuid4()).__name__ returns "int" not
# "UUID", random.seed(42); random.randint(1, 10) returns 10
# not 2 — the Mersenne Twister seeded sequence diverges from
# CPython's documented contract) are covered in the matching
# spec fixture `lang_hashlib_hmac_uuid_random_silent`.
import hashlib
import hmac
import secrets
import uuid
import random


_ledger: list[int] = []

# 1) hashlib — full module hasattr surface
assert hasattr(hashlib, "sha256") == True; _ledger.append(1)
assert hasattr(hashlib, "sha1") == True; _ledger.append(1)
assert hasattr(hashlib, "md5") == True; _ledger.append(1)
assert hasattr(hashlib, "sha512") == True; _ledger.append(1)
assert hasattr(hashlib, "sha224") == True; _ledger.append(1)
assert hasattr(hashlib, "sha384") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2b") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2s") == True; _ledger.append(1)
assert hasattr(hashlib, "new") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_guaranteed") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_available") == True; _ledger.append(1)

# 2) hashlib hexdigest value contract
assert hashlib.sha256(b"hello").hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)
assert hashlib.md5(b"hello").hexdigest() == "5d41402abc4b2a76b9719d911017c592"; _ledger.append(1)
assert hashlib.sha1(b"hello").hexdigest() == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"; _ledger.append(1)

# 3) hashlib.sha256 — digest_size attribute contract
assert hashlib.sha256(b"hello").digest_size == 32; _ledger.append(1)

# 4) hmac — full module hasattr surface
assert hasattr(hmac, "new") == True; _ledger.append(1)
assert hasattr(hmac, "compare_digest") == True; _ledger.append(1)
assert hasattr(hmac, "digest") == True; _ledger.append(1)
assert hasattr(hmac, "HMAC") == True; _ledger.append(1)

# 5) hmac.new — hexdigest value contract
assert hmac.new(b"key", b"msg", "sha256").hexdigest() == "2d93cbc1be167bcb1637a4a23cbff01a7878f0c50ee833954ea5221bb1b8c628"; _ledger.append(1)

# 6) hmac.compare_digest — True / False value contract
assert hmac.compare_digest("a", "a") == True; _ledger.append(1)
assert hmac.compare_digest("a", "b") == False; _ledger.append(1)

# 7) secrets — full module hasattr surface
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "randbits") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)

# 8) secrets — token return-type / length value contract
assert type(secrets.token_bytes(16)).__name__ == "bytes"; _ledger.append(1)
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert type(secrets.token_hex(8)).__name__ == "str"; _ledger.append(1)
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)

# 9) uuid — full module hasattr surface
assert hasattr(uuid, "uuid1") == True; _ledger.append(1)
assert hasattr(uuid, "uuid3") == True; _ledger.append(1)
assert hasattr(uuid, "uuid4") == True; _ledger.append(1)
assert hasattr(uuid, "uuid5") == True; _ledger.append(1)
assert hasattr(uuid, "UUID") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_DNS") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_URL") == True; _ledger.append(1)

# 10) uuid.UUID — round-trip string value contract
assert str(uuid.UUID("12345678-1234-5678-1234-567812345678")) == "12345678-1234-5678-1234-567812345678"; _ledger.append(1)

# 11) uuid.uuid4 — .hex / str length contract
_u = uuid.uuid4()
assert len(str(_u)) == 36; _ledger.append(1)
assert len(_u.hex) == 32; _ledger.append(1)

# 12) random — full module hasattr surface
assert hasattr(random, "random") == True; _ledger.append(1)
assert hasattr(random, "randint") == True; _ledger.append(1)
assert hasattr(random, "choice") == True; _ledger.append(1)
assert hasattr(random, "shuffle") == True; _ledger.append(1)
assert hasattr(random, "sample") == True; _ledger.append(1)
assert hasattr(random, "Random") == True; _ledger.append(1)
assert hasattr(random, "seed") == True; _ledger.append(1)
assert hasattr(random, "uniform") == True; _ledger.append(1)
assert hasattr(random, "gauss") == True; _ledger.append(1)
assert hasattr(random, "randrange") == True; _ledger.append(1)

# 13) random.random — value-in-range contract
_r = random.random()
assert type(_r).__name__ == "float"; _ledger.append(1)
assert 0 <= _r < 1; _ledger.append(1)

# 14) random.seed — determinism (same seed → same randint twice)
random.seed(42)
_first = random.randint(1, 10)
random.seed(42)
_second = random.randint(1, 10)
assert _first == _second; _ledger.append(1)

# NB: type(hashlib.sha256(b"hello")).__name__ returns "int" on
# mamba, type(hmac.new(b"key", b"msg", "sha256")).__name__
# returns "int" on mamba, type(uuid.uuid4()).__name__ returns
# "int" on mamba, and random.seed(42); random.randint(1, 10)
# returns 10 on mamba (not 2 — the documented Mersenne Twister
# seeded sequence diverges) — all DIVERGE on mamba — moved to
# the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_hashlib_hmac_secrets_uuid_random_value_ops {sum(_ledger)} asserts")
