# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_debug_assignment"
# subject = "cpython.test_compile.TestSpecifics.test_debug_assignment"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_debug_assignment
"""Auto-ported test: TestSpecifics::test_debug_assignment (CPython 3.12 oracle)."""


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
    compile('__debug__ = 1', '?', 'single')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
import builtins
prev = builtins.__debug__
setattr(builtins, '__debug__', 'sure')

assert __debug__ == prev
setattr(builtins, '__debug__', prev)
print("TestSpecifics::test_debug_assignment: ok")
