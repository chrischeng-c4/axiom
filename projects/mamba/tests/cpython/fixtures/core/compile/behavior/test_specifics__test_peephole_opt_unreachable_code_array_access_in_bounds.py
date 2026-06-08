# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_peephole_opt_unreachable_code_array_access_in_bounds"
# subject = "cpython.test_compile.TestSpecifics.test_peephole_opt_unreachable_code_array_access_in_bounds"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_peephole_opt_unreachable_code_array_access_in_bounds
"""Auto-ported test: TestSpecifics::test_peephole_opt_unreachable_code_array_access_in_bounds (CPython 3.12 oracle)."""


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
"""Regression test for issue35193 when run under clang msan."""

def unused_code_at_end():
    return 3
    raise RuntimeError('unreachable')

assert 'RETURN_CONST' == list(dis.get_instructions(unused_code_at_end))[-1].opname
print("TestSpecifics::test_peephole_opt_unreachable_code_array_access_in_bounds: ok")
