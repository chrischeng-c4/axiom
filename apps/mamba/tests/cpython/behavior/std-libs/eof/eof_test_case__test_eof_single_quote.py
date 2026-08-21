# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eof"
# dimension = "behavior"
# case = "eof_test_case__test_eof_single_quote"
# subject = "cpython.test_eof.EOFTestCase.test_EOF_single_quote"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_eof.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_eof.py::EOFTestCase::test_EOF_single_quote
"""Auto-ported test: EOFTestCase::test_EOF_single_quote (CPython 3.12 oracle)."""


import sys
from codecs import BOM_UTF8
from test import support
from test.support import os_helper
from test.support import script_helper
from test.support import warnings_helper
import unittest


'test script for a few new invalid token catches'


# --- test body ---
expect = 'unterminated string literal (detected at line 1) (<string>, line 1)'
for quote in ("'", '"'):
    try:
        eval(f'{quote}this is a test                ')
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert str(cm.exception) == expect

    assert cm.exception.offset == 1
print("EOFTestCase::test_EOF_single_quote: ok")
