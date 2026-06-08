# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_badfuture9"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_badfuture9"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_badfuture9
"""Auto-ported test: FutureTest::test_badfuture9 (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
def check_syntax_error(err, basename, lineno, offset=1):

    assert '%s.py, line %d' % (basename, lineno) in str(err)

    assert os.path.basename(err.filename) == basename + '.py'

    assert err.lineno == lineno

    assert err.offset == offset
try:
    from test.test_future_stmt import badsyntax_future9
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
check_syntax_error(cm.exception, 'badsyntax_future9', 3, 39)
print("FutureTest::test_badfuture9: ok")
