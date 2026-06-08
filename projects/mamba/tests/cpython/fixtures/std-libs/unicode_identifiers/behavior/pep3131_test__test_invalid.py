# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode_identifiers"
# dimension = "behavior"
# case = "pep3131_test__test_invalid"
# subject = "cpython.test_unicode_identifiers.PEP3131Test.test_invalid"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode_identifiers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode_identifiers.py::PEP3131Test::test_invalid
"""Auto-ported test: PEP3131Test::test_invalid (CPython 3.12 oracle)."""


import unittest


# --- test body ---
try:
    from test.tokenizedata import badsyntax_3131
except SyntaxError as err:

    assert str(err) == "invalid character '€' (U+20AC) (badsyntax_3131.py, line 2)"

    assert err.lineno == 2

    assert err.offset == 1
else:

    raise AssertionError("expected exception didn't occur")
print("PEP3131Test::test_invalid: ok")
