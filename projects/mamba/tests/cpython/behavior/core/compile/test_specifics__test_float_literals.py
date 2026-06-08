# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_float_literals"
# subject = "cpython.test_compile.TestSpecifics.test_float_literals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_float_literals
"""Auto-ported test: TestSpecifics::test_float_literals (CPython 3.12 oracle)."""


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
    eval('2e')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    eval('2.0e+')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    eval('1e-')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    eval('3-4e/21')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("TestSpecifics::test_float_literals: ok")
