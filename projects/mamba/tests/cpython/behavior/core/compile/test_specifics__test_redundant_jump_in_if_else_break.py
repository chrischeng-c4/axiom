# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_redundant_jump_in_if_else_break"
# subject = "cpython.test_compile.TestSpecifics.test_redundant_jump_in_if_else_break"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_redundant_jump_in_if_else_break
"""Auto-ported test: TestSpecifics::test_redundant_jump_in_if_else_break (CPython 3.12 oracle)."""


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
def if_else_break():
    val = 1
    while True:
        if val > 0:
            val -= 1
        else:
            break
        val = -1
INSTR_SIZE = 2
HANDLED_JUMPS = ('POP_JUMP_IF_FALSE', 'POP_JUMP_IF_TRUE', 'JUMP_ABSOLUTE', 'JUMP_FORWARD')
for line, instr in enumerate(dis.Bytecode(if_else_break, show_caches=True)):
    if instr.opname == 'JUMP_FORWARD':

        assert instr.arg != 0
    elif instr.opname in HANDLED_JUMPS:

        assert instr.arg != (line + 1) * INSTR_SIZE
print("TestSpecifics::test_redundant_jump_in_if_else_break: ok")
