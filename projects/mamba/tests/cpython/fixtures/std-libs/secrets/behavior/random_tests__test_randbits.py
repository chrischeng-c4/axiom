# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "random_tests__test_randbits"
# subject = "cpython.test_secrets.Random_Tests.test_randbits"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_secrets.py::Random_Tests::test_randbits
"""Auto-ported test: Random_Tests::test_randbits (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
errmsg = 'randbits(%d) returned %d'
for numbits in (3, 12, 30):
    for i in range(6):
        n = secrets.randbits(numbits)

        assert 0 <= n < 2 ** numbits
print("Random_Tests::test_randbits: ok")
