# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_tests__test_bool"
# subject = "cpython.test_secrets.Compare_Digest_Tests.test_bool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Compare_Digest_Tests::test_bool
"""Auto-ported test: Compare_Digest_Tests::test_bool (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---

assert isinstance(secrets.compare_digest('abc', 'abc'), bool)

assert isinstance(secrets.compare_digest('abc', 'xyz'), bool)
print("Compare_Digest_Tests::test_bool: ok")
