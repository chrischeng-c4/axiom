# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utf8source"
# dimension = "behavior"
# case = "pep3120_test__test_badsyntax"
# subject = "cpython.test_utf8source.PEP3120Test.test_badsyntax"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_utf8source.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_utf8source.py::PEP3120Test::test_badsyntax
"""Auto-ported test: PEP3120Test::test_badsyntax (CPython 3.12 oracle)."""


import unittest


# --- test body ---
try:
    import test.tokenizedata.badsyntax_pep3120
except SyntaxError as msg:
    msg = str(msg).lower()

    assert 'utf-8' in msg
else:

    raise AssertionError("expected exception didn't occur")
print("PEP3120Test::test_badsyntax: ok")
