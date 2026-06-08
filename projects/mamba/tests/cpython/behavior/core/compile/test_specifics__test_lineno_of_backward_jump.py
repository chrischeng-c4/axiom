# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_lineno_of_backward_jump"
# subject = "cpython.test_compile.TestSpecifics.test_lineno_of_backward_jump"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_lineno_of_backward_jump
"""Auto-ported test: TestSpecifics::test_lineno_of_backward_jump (CPython 3.12 oracle)."""


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
def f():
    for i in x:
        if y:
            pass
linenos = list((inst.positions.lineno for inst in dis.get_instructions(f.__code__) if inst.opname == 'JUMP_BACKWARD'))

assert len(linenos) > 0

assert all((l is not None for l in linenos))
print("TestSpecifics::test_lineno_of_backward_jump: ok")
