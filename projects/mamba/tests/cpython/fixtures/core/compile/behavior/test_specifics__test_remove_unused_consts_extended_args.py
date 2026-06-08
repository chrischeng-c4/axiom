# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_remove_unused_consts_extended_args"
# subject = "cpython.test_compile.TestSpecifics.test_remove_unused_consts_extended_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_remove_unused_consts_extended_args
"""Auto-ported test: TestSpecifics::test_remove_unused_consts_extended_args (CPython 3.12 oracle)."""


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
N = 1000
code = ['def f():\n']
code.append("\ts = ''\n")
code.append('\tfor i in range(1):\n')
for i in range(N):
    code.append(f"\t\tif True: s += 't{i}'\n")
    code.append(f"\t\tif False: s += 'f{i}'\n")
code.append('\treturn s\n')
code = ''.join(code)
g = {}
eval(compile(code, 'file.py', 'exec'), g)
exec(code, g)
f = g['f']
expected = tuple([None, '', 1] + [f't{i}' for i in range(N)])

assert f.__code__.co_consts == expected
expected = ''.join(expected[3:])

assert expected == f()
print("TestSpecifics::test_remove_unused_consts_extended_args: ok")
