# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_none_assignment"
# subject = "cpython.test_compile.TestSpecifics.test_none_assignment"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_none_assignment
"""Auto-ported test: TestSpecifics::test_none_assignment (CPython 3.12 oracle)."""


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
stmts = ['None = 0', 'None += 0', '__builtins__.None = 0', 'def None(): pass', 'class None: pass', '(a, None) = 0, 0', 'for None in range(10): pass', 'def f(None): pass', 'import None', 'import x as None', 'from x import None', 'from x import y as None']
for stmt in stmts:
    stmt += '\n'

    try:
        compile(stmt, 'tmp', 'single')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass

    try:
        compile(stmt, 'tmp', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
print("TestSpecifics::test_none_assignment: ok")
