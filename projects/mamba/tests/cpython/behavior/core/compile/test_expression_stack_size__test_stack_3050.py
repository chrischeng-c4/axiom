# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_expression_stack_size__test_stack_3050"
# subject = "cpython.test_compile.TestExpressionStackSize.test_stack_3050"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestExpressionStackSize::test_stack_3050
"""Auto-ported test: TestExpressionStackSize::test_stack_3050 (CPython 3.12 oracle)."""


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
N = 100
M = 3050
code = 'x,' * M + '=t'
compile(code, '<foo>', 'single')
print("TestExpressionStackSize::test_stack_3050: ok")
