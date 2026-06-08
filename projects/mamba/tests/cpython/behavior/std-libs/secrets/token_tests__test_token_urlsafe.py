# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_urlsafe"
# subject = "cpython.test_secrets.Token_Tests.test_token_urlsafe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_urlsafe
"""Auto-ported test: Token_Tests::test_token_urlsafe (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
legal = string.ascii_letters + string.digits + '-_'
for n in (1, 11, 28, 76):
    s = secrets.token_urlsafe(n)

    assert isinstance(s, str)

    assert all((c in legal for c in s))
print("Token_Tests::test_token_urlsafe: ok")
