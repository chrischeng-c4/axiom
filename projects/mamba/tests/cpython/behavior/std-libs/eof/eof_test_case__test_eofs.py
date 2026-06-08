# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eof"
# dimension = "behavior"
# case = "eof_test_case__test_eofs"
# subject = "cpython.test_eof.EOFTestCase.test_EOFS"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_eof.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_eof.py::EOFTestCase::test_EOFS
"""Auto-ported test: EOFTestCase::test_EOFS (CPython 3.12 oracle)."""


import sys
from codecs import BOM_UTF8
from test import support
from test.support import os_helper
from test.support import script_helper
from test.support import warnings_helper
import unittest


'test script for a few new invalid token catches'


# --- test body ---
expect = 'unterminated triple-quoted string literal (detected at line 3) (<string>, line 1)'
try:
    eval("ä = '''thîs is \na \ntest")
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expect

assert cm.exception.text == "ä = '''thîs is "

assert cm.exception.offset == 5
try:
    eval("ä = '''thîs is \na \ntest".encode())
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expect

assert cm.exception.text == "ä = '''thîs is "

assert cm.exception.offset == 5
try:
    eval(BOM_UTF8 + "ä = '''thîs is \na \ntest".encode())
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expect

assert cm.exception.text == "ä = '''thîs is "

assert cm.exception.offset == 5
try:
    eval("# coding: latin1\nä = '''thîs is \na \ntest".encode('latin1'))
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == 'unterminated triple-quoted string literal (detected at line 4) (<string>, line 2)'

assert cm.exception.text == "ä = '''thîs is "

assert cm.exception.offset == 5
print("EOFTestCase::test_EOFS: ok")
