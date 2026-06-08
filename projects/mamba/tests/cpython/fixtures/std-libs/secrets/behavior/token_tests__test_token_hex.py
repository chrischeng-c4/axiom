# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_hex"
# subject = "cpython.test_secrets.Token_Tests.test_token_hex"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_hex
"""Auto-ported test: Token_Tests::test_token_hex (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for n in (1, 12, 25, 90):
    s = secrets.token_hex(n)

    assert isinstance(s, str)

    assert len(s) == 2 * n

    assert all((c in string.hexdigits for c in s))
print("Token_Tests::test_token_hex: ok")
