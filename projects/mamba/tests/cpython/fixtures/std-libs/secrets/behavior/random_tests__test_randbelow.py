# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "random_tests__test_randbelow"
# subject = "cpython.test_secrets.Random_Tests.test_randbelow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_secrets.py::Random_Tests::test_randbelow
"""Auto-ported test: Random_Tests::test_randbelow (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for i in range(2, 10):

    assert secrets.randbelow(i) in range(i)

try:
    secrets.randbelow(0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    secrets.randbelow(-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("Random_Tests::test_randbelow: ok")
