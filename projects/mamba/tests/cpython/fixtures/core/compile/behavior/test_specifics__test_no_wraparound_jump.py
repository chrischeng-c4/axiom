# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_no_wraparound_jump"
# subject = "cpython.test_compile.TestSpecifics.test_no_wraparound_jump"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_no_wraparound_jump
"""Auto-ported test: TestSpecifics::test_no_wraparound_jump (CPython 3.12 oracle)."""


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
def while_not_chained(a, b, c):
    while not a < b < c:
        pass
for instr in dis.Bytecode(while_not_chained):

    assert instr.opname != 'EXTENDED_ARG'
print("TestSpecifics::test_no_wraparound_jump: ok")
