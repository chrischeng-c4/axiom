# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "random_tests__test_choice"
# subject = "cpython.test_secrets.Random_Tests.test_choice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_secrets.py::Random_Tests::test_choice
"""Auto-ported test: Random_Tests::test_choice (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
items = [1, 2, 4, 8, 16, 32, 64]
for i in range(10):

    assert secrets.choice(items) in items
print("Random_Tests::test_choice: ok")
