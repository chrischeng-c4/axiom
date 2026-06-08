# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_canary_realworld"
# subject = "cpython321.test_canary_realworld"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_canary_realworld.py"
# status = "filled"
# ///
"""cpython321.test_canary_realworld: execute CPython 3.12 seed test_canary_realworld"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_canary_realworld.py — minimal hello-world seed proving the
# runner walks the `realworld/` fixture root (#3335). Pinned at
# AssertionPass via the `MAMBA_ASSERTION_PASS:` marker so any
# regression in the walker (e.g. silently dropping a root) flips
# this to a missing-stem drift.
#
# Marker token convention: per `test_math.py`, the token after
# `MAMBA_ASSERTION_PASS:` strips the leading `test_` prefix from the
# stem, so this seed emits `canary_realworld`.

_ledger: list[int] = []

assert 1 == 1, "identity"
_ledger.append(1)

assert 1 + 1 == 2, "addition"
_ledger.append(1)

assert "abc"[0] == "a", "string index"
_ledger.append(1)

assert [1, 2, 3][-1] == 3, "list index"
_ledger.append(1)

assert {"k": "v"}["k"] == "v", "dict index"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: canary_realworld {sum(_ledger)} asserts")
