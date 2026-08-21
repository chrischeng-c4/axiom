# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_argument_handling"
# subject = "cpython.test_compile.TestSpecifics.test_argument_handling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_argument_handling
"""Auto-ported test: TestSpecifics::test_argument_handling (CPython 3.12 oracle)."""


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
    eval('lambda a,a:0')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    eval('lambda a,a=1:0')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    eval('lambda a=1,a=1:0')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    exec('def f(a, a): pass')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    exec('def f(a = 0, a = 1): pass')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    exec('def f(a): global a; a = 1')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("TestSpecifics::test_argument_handling: ok")
