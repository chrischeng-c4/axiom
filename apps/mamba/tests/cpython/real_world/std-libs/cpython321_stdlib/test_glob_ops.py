# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_glob_ops"
# subject = "cpython321.test_glob_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_glob_ops.py"
# status = "filled"
# ///
"""cpython321.test_glob_ops: execute CPython 3.12 seed test_glob_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `glob.glob` shell-style
# wildcard filename matching.
# Surface: returns a list (possibly empty); no-match returns the
# empty list rather than raising. Asserts result-type contract; does
# not assert specific file enumeration (which depends on the runner
# filesystem).
# Companion to stub/test_glob.py — vendored unittest seed.
import glob
_ledger: list[int] = []
# No-match pattern → empty list (well-defined)
empty = glob.glob("/tmp/_mamba_zzz_definitely_does_not_exist_*.zzz")
assert isinstance(empty, list); _ledger.append(1)
assert empty == []; _ledger.append(1)
assert len(empty) == 0; _ledger.append(1)
# A pattern that targets a real directory's listing → still a list
results = glob.glob("/tmp/*.txt")
assert isinstance(results, list); _ledger.append(1)
# Result length is non-negative (trivially true; pins the API)
assert len(results) >= 0; _ledger.append(1)
# Distinct no-match patterns both return empty (idempotent)
e1 = glob.glob("/tmp/_mamba_zzz_a_*.zzz")
e2 = glob.glob("/tmp/_mamba_zzz_b_*.zzz")
assert e1 == e2; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_glob_ops {sum(_ledger)} asserts")
