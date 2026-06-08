# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_tests__test_bad_types"
# subject = "cpython.test_secrets.Compare_Digest_Tests.test_bad_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_secrets.py::Compare_Digest_Tests::test_bad_types
"""Auto-ported test: Compare_Digest_Tests::test_bad_types (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
a = 'abcde'
b = a.encode('utf-8')
assert isinstance(a, str)
assert isinstance(b, bytes)

try:
    secrets.compare_digest(a, b)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    secrets.compare_digest(b, a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("Compare_Digest_Tests::test_bad_types: ok")
