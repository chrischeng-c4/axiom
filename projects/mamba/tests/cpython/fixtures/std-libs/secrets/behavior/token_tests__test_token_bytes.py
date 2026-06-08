# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_bytes"
# subject = "cpython.test_secrets.Token_Tests.test_token_bytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_bytes
"""Auto-ported test: Token_Tests::test_token_bytes (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for n in (1, 8, 17, 100):

    assert isinstance(secrets.token_bytes(n), bytes)

    assert len(secrets.token_bytes(n)) == n
print("Token_Tests::test_token_bytes: ok")
