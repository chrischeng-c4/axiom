# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_mimetypes_ops"
# subject = "cpython321.test_mimetypes_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_mimetypes_ops.py"
# status = "filled"
# ///
"""cpython321.test_mimetypes_ops: execute CPython 3.12 seed test_mimetypes_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `mimetypes.guess_type` +
# `mimetypes.guess_extension`.
# Surface: guess_type returns (content_type, encoding) tuple — for
# the standard suffixes the type is fully determined; for an unknown
# suffix both fields are None. guess_extension returns the canonical
# extension string for a given content type.
# Companion to stub/test_mimetypes.py — vendored unittest seed.
import mimetypes
_ledger: list[int] = []
# Common content-type suffixes
assert mimetypes.guess_type("file.html") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_type("file.json") == ("application/json", None); _ledger.append(1)
assert mimetypes.guess_type("file.png") == ("image/png", None); _ledger.append(1)
# Unknown suffix → (None, None) — both slots None, no exception
assert mimetypes.guess_type("file.unknown_xyz_zzz") == (None, None); _ledger.append(1)
# guess_extension is the inverse: content type → canonical extension
assert mimetypes.guess_extension("text/html") == ".html"; _ledger.append(1)
# Result tuple-vs-pair invariants
got = mimetypes.guess_type("file.json")
assert len(got) == 2; _ledger.append(1)
assert got[0] == "application/json"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_mimetypes_ops {sum(_ledger)} asserts")
