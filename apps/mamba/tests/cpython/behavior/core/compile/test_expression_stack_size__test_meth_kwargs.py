# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_expression_stack_size__test_meth_kwargs"
# subject = "cpython.test_compile.TestExpressionStackSize.test_meth_kwargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestExpressionStackSize::test_meth_kwargs
"""Auto-ported test: TestExpressionStackSize::test_meth_kwargs (CPython 3.12 oracle)."""


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

def check_stack_size(code):
    if isinstance(code, str):
        code = compile(code, '<foo>', 'single')
    max_size = math.ceil(math.log(len(code.co_code)))

    assert code.co_stacksize <= max_size
kwargs = (f'a{i}=x' for i in range(N))
check_stack_size('o.m(' + ', '.join(kwargs) + ')')
print("TestExpressionStackSize::test_meth_kwargs: ok")
