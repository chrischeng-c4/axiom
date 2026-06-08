# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_tempfile_ops"
# subject = "cpython321.test_tempfile_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_tempfile_ops.py"
# status = "filled"
# ///
"""cpython321.test_tempfile_ops: execute CPython 3.12 seed test_tempfile_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `tempfile.gettempdir`.
# Surface: gettempdir returns a non-empty string path that exists.
# Companion to stub/test_tempfile.py — vendored unittest seed.
import tempfile
import os
_ledger: list[int] = []
d = tempfile.gettempdir()
assert isinstance(d, str); _ledger.append(1)
assert len(d) > 0; _ledger.append(1)
assert d.startswith("/"); _ledger.append(1)
assert os.path.exists(d); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_tempfile_ops {sum(_ledger)} asserts")
