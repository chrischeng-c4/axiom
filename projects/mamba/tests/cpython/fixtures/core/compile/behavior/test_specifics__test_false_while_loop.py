# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_false_while_loop"
# subject = "cpython.test_compile.TestSpecifics.test_false_while_loop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_false_while_loop
"""Auto-ported test: TestSpecifics::test_false_while_loop (CPython 3.12 oracle)."""


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
def break_in_while():
    while False:
        break

def continue_in_while():
    while False:
        continue
funcs = [break_in_while, continue_in_while]
for func in funcs:
    opcodes = list(dis.get_instructions(func))

    assert 2 == len(opcodes)

    assert 'RETURN_CONST' == opcodes[1].opname

    assert None == opcodes[1].argval
print("TestSpecifics::test_false_while_loop: ok")
