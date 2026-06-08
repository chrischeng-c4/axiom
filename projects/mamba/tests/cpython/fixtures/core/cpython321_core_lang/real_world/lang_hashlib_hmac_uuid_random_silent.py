# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_hashlib_hmac_uuid_random_silent"
# subject = "cpython321.lang_hashlib_hmac_uuid_random_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_hashlib_hmac_uuid_random_silent.py"
# status = "filled"
# ///
"""cpython321.lang_hashlib_hmac_uuid_random_silent: execute CPython 3.12 seed lang_hashlib_hmac_uuid_random_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# hashlib HASH instance class-identity contract + hmac HMAC
# instance class-identity contract + uuid UUID instance class-
# identity contract + random.seed Mersenne-Twister stable-
# sequence value contract pinned by atomic 188: `hashlib` (the
# documented `type(hashlib.sha256(b"hello")).__name__ == "HASH"`
# class-identity contract for the digest instance), `hmac` (the
# documented `type(hmac.new(b"key", b"msg", "sha256")).__name__
# == "HMAC"` class-identity contract for the keyed-hash
# instance), `uuid` (the documented `type(uuid.uuid4()).__name__
# == "UUID"` class-identity contract for the random UUID
# instance), and `random` (the documented `random.seed(42);
# random.randint(1, 10) == 2` Mersenne-Twister stable-sequence
# value contract — CPython's documented seeded RNG output for a
# given seed is part of the stable public contract since the
# adoption of MT19937).
#
# The matching subset (full hashlib hasattr + hexdigest values
# + digest_size, full hmac hasattr + hexdigest + compare_digest,
# full secrets hasattr + token_bytes / token_hex return-type +
# length, full uuid hasattr + UUID round-trip string + uuid4
# hex/str length, full random hasattr + random in [0,1) + seed
# determinism for back-to-back same-seed calls) is covered by
# `test_hashlib_hmac_secrets_uuid_random_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(hashlib.sha256(b"hello")).__name__ == "HASH" —
#     documented class identity for the digest instance
#     (mamba: returns "int" — the digest is rebound to an
#     integer-handle placeholder that still happens to
#     surface .hexdigest());
#   • type(hmac.new(b"key", b"msg", "sha256")).__name__ ==
#     "HMAC" — documented class identity for the keyed-hash
#     instance (mamba: returns "int" — same integer-handle
#     placeholder pattern as hashlib);
#   • type(uuid.uuid4()).__name__ == "UUID" — documented
#     class identity for the random UUID instance (mamba:
#     returns "int" — same integer-handle placeholder
#     pattern; the surface .hex / str(.) accessors still
#     surface but the class identity collapses);
#   • random.seed(42); random.randint(1, 10) == 2 —
#     documented Mersenne-Twister stable-sequence value
#     contract (mamba: returns 10 — the RNG sequence
#     diverges from CPython's documented MT19937 output for
#     the same seed).
import hashlib as _hashlib_mod
import hmac as _hmac_mod
import uuid as _uuid_mod
import random as _random_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
hashlib: Any = _hashlib_mod
hmac: Any = _hmac_mod
uuid: Any = _uuid_mod
random: Any = _random_mod


_ledger: list[int] = []

# 1) hashlib — HASH digest-instance class-identity contract
_h = hashlib.sha256(b"hello")
assert type(_h).__name__ == "HASH"; _ledger.append(1)

# 2) hmac — HMAC keyed-hash-instance class-identity contract
_m = hmac.new(b"key", b"msg", "sha256")
assert type(_m).__name__ == "HMAC"; _ledger.append(1)

# 3) uuid — UUID random-instance class-identity contract
_u = uuid.uuid4()
assert type(_u).__name__ == "UUID"; _ledger.append(1)

# 4) random.seed — Mersenne-Twister stable-sequence value
random.seed(42)
assert random.randint(1, 10) == 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_hashlib_hmac_uuid_random_silent {sum(_ledger)} asserts")
