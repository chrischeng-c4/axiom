# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_defaults"
# subject = "cpython.test_secrets.Token_Tests.test_token_defaults"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_defaults
"""Auto-ported test: Token_Tests::test_token_defaults (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for func in (secrets.token_bytes, secrets.token_hex, secrets.token_urlsafe):
    name = func.__name__
    try:
        func()
    except TypeError:

        raise AssertionError('%s cannot be called with no argument' % name)
    try:
        func(None)
    except TypeError:

        raise AssertionError('%s cannot be called with None' % name)
size = secrets.DEFAULT_ENTROPY

assert len(secrets.token_bytes(None)) == size

assert len(secrets.token_hex(None)) == 2 * size
print("Token_Tests::test_token_defaults: ok")
