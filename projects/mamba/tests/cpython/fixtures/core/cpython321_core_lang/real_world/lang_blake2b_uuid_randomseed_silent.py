# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_blake2b_uuid_randomseed_silent"
# subject = "cpython321.lang_blake2b_uuid_randomseed_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_blake2b_uuid_randomseed_silent.py"
# status = "filled"
# ///
"""cpython321.lang_blake2b_uuid_randomseed_silent: execute CPython 3.12 seed lang_blake2b_uuid_randomseed_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# hashlib.blake2b digest algorithm + the uuid.UUID instance
# attribute surface + the random PRNG seeded-reproducibility
# contract + the random.SystemRandom class identifier surface
# pinned by atomic 177: `hashlib` (the documented
# `blake2b(...).hexdigest()` exact-value contract), `uuid`
# (the documented `uuid4()` returning a `UUID` instance + the
# documented `UUID(str).int` / `.version` instance attribute
# surface), and `random` (the documented `seed(n); random()`
# PRNG reproducibility contract + the documented `SystemRandom`
# class identifier surface).
#
# The matching subset (hashlib md5 / sha1 / sha256 / sha512 /
# sha224 / sha384 hexdigest + new + digest_size + hashlib
# hasattr surface, hmac new + hexdigest + compare_digest +
# hasattr surface, secrets token_hex / token_bytes / randbelow
# / choice + hasattr surface, random random / uniform /
# randint / choice / sample / shuffle shape contracts +
# partial random hasattr surface, uuid.UUID(str) str round-
# trip + uuid module hasattr surface, full math value
# contract + math hasattr surface) is covered by
# `test_hashlib_hmac_secrets_math_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hashlib.blake2b(b"hello").hexdigest()[:16] ==
#     "e4cfa39a3d37be31" — documented BLAKE2b-512 digest
#     algorithm (mamba: returns "9b71d224bd62f378" — the
#     blake2b implementation is broken — mamba returns the
#     sha512 digest under the blake2b name);
#   • type(uuid.uuid4()).__name__ == "UUID" — documented
#     constructor class identity (mamba: returns "int" —
#     uuid4 produces a bare int not the documented UUID
#     instance);
#   • str(uuid.UUID("12345678-1234-5678-1234-567812345678")
#     .int) == "24197857161011715162171839636988778104" —
#     documented UUID instance attribute (mamba: returns
#     "96559521639168" — the .int instance attribute is
#     broken — asserted via str() round-trip because mamba's
#     parser cannot accept a 128-bit int literal);
#   • uuid.UUID("12345678-1234-5678-1234-567812345678").
#     version is None — documented UUID instance attribute
#     when version is not specified at construction (mamba:
#     returns 5 — the .version instance attribute mis-reports
#     the version);
#   • random.seed(42); random.random() == 0.6394267984578837
#     — documented PRNG seeded-reproducibility contract
#     (mamba: returns 0.3745401188473625 — the PRNG state
#     diverges from CPython's Mersenne Twister and the
#     documented reproducibility contract is broken);
#   • hasattr(random, "SystemRandom") is True — documented
#     class identifier (mamba: False — the cryptographic-PRNG
#     class is missing).
import hashlib as _hashlib_mod
import uuid as _uuid_mod
import random as _random_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# digest functions / class identifiers / instance attributes
# that mamba's bundled type stubs do not surface accurately.
hashlib: Any = _hashlib_mod
uuid: Any = _uuid_mod
random: Any = _random_mod


_ledger: list[int] = []

# 1) hashlib.blake2b — BLAKE2b-512 digest on b"hello"
assert hashlib.blake2b(b"hello").hexdigest()[:16] == "e4cfa39a3d37be31"; _ledger.append(1)

# 2) uuid.uuid4 — class identity
assert type(uuid.uuid4()).__name__ == "UUID"; _ledger.append(1)

# 3) uuid.UUID — .int instance attribute (string round-trip
#    because mamba's parser rejects 128-bit int literals)
_u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert str(_u.int) == "24197857161011715162171839636988778104"; _ledger.append(1)

# 4) uuid.UUID — .version instance attribute (None when
#    no version is specified at construction)
assert _u.version is None; _ledger.append(1)

# 5) random.seed + random.random — PRNG reproducibility
random.seed(42)
assert random.random() == 0.6394267984578837; _ledger.append(1)

# 6) random.SystemRandom — class identifier hasattr
assert hasattr(random, "SystemRandom") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_blake2b_uuid_randomseed_silent {sum(_ledger)} asserts")
