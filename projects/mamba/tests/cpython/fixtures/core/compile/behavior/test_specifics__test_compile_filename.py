# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_compile_filename"
# subject = "cpython.test_compile.TestSpecifics.test_compile_filename"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_compile_filename
"""Auto-ported test: TestSpecifics::test_compile_filename (CPython 3.12 oracle)."""


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
for filename in ('file.py', b'file.py'):
    code = compile('pass', filename, 'exec')

    assert code.co_filename == 'file.py'
for filename in (bytearray(b'file.py'), memoryview(b'file.py')):
    try:
        compile('pass', filename, 'exec')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

try:
    compile('pass', list(b'file.py'), 'exec')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestSpecifics::test_compile_filename: ok")
