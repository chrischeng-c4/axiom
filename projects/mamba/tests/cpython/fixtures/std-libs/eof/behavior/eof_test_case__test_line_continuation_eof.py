# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eof"
# dimension = "behavior"
# case = "eof_test_case__test_line_continuation_eof"
# subject = "cpython.test_eof.EOFTestCase.test_line_continuation_EOF"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_eof.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_eof.py::EOFTestCase::test_line_continuation_EOF
"""Auto-ported test: EOFTestCase::test_line_continuation_EOF (CPython 3.12 oracle)."""


import sys
from codecs import BOM_UTF8
from test import support
from test.support import os_helper
from test.support import script_helper
from test.support import warnings_helper
import unittest


'test script for a few new invalid token catches'


# --- test body ---
"""A continuation at the end of input must be an error; bpo2180."""
expect = 'unexpected EOF while parsing (<string>, line 1)'
try:
    exec('ä = 5\\')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expect

assert cm.exception.text == 'ä = 5\\\n'

assert cm.exception.offset == 7
try:
    exec('ä = 5\\'.encode())
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expect

assert cm.exception.text == 'ä = 5\\\n'

assert cm.exception.offset == 7
try:
    exec('# coding:latin1\nä = 5\\'.encode('latin1'))
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == 'unexpected EOF while parsing (<string>, line 2)'

assert cm.exception.text == 'ä = 5\\\n'

assert cm.exception.offset == 7
try:
    exec(BOM_UTF8 + 'ä = 5\\'.encode())
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expect

assert cm.exception.text == 'ä = 5\\\n'

assert cm.exception.offset == 7
try:
    exec('\\')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == expect
print("EOFTestCase::test_line_continuation_EOF: ok")
