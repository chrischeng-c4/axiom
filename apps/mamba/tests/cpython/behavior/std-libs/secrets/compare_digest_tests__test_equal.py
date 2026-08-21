# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_tests__test_equal"
# subject = "cpython.test_secrets.Compare_Digest_Tests.test_equal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Compare_Digest_Tests::test_equal
"""Auto-ported test: Compare_Digest_Tests::test_equal (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for s in ('a', 'bcd', 'xyz123'):
    a = s * 100
    b = s * 100

    assert secrets.compare_digest(a, b)

    assert secrets.compare_digest(a.encode('utf-8'), b.encode('utf-8'))
print("Compare_Digest_Tests::test_equal: ok")
