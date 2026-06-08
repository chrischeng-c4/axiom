# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_null_terminated"
# subject = "cpython.test_compile.TestSpecifics.test_null_terminated"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_null_terminated
"""Auto-ported test: TestSpecifics::test_null_terminated (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
try:
    compile('123\x00', '<dummy>', 'eval')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot contain null', str(_aR_e))
try:
    compile(memoryview(b'123\x00'), '<dummy>', 'eval')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('cannot contain null', str(_aR_e))
code = compile(memoryview(b'123\x00')[1:-1], '<dummy>', 'eval')

assert eval(code) == 23
code = compile(memoryview(b'1234')[1:-1], '<dummy>', 'eval')

assert eval(code) == 23
code = compile(memoryview(b'$23$')[1:-1], '<dummy>', 'eval')

assert eval(code) == 23

assert eval(memoryview(b'1234')[1:-1]) == 23
namespace = dict()
exec(memoryview(b'ax = 123')[1:-1], namespace)

assert namespace['x'] == 12
print("TestSpecifics::test_null_terminated: ok")
