# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_compare_positions"
# subject = "cpython.test_compile.TestSpecifics.test_compare_positions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_compare_positions
"""Auto-ported test: TestSpecifics::test_compare_positions (CPython 3.12 oracle)."""


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
def assertInvalidSingle(source):

    try:
        compile_single(source)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass

def check_constant(func, expected):
    for const in func.__code__.co_consts:
        if repr(const) == repr(expected):
            break
    else:

        raise AssertionError('unable to find constant %r in %r' % (expected, func.__code__.co_consts))

def compile_single(source):
    compile(source, '<single>', 'single')

def get_code_lines(code):
    last_line = -2
    res = []
    for _, _, line in code.co_lines():
        if line is not None and line != last_line:
            res.append(line - code.co_firstlineno)
            last_line = line
    return res
for opname_prefix, op in [('COMPARE_', '<'), ('COMPARE_', '<='), ('COMPARE_', '>'), ('COMPARE_', '>='), ('CONTAINS_OP', 'in'), ('CONTAINS_OP', 'not in'), ('IS_OP', 'is'), ('IS_OP', 'is not')]:
    expr = f'a {op} b {op} c'
    expected_positions = 2 * [(2, 2, 0, len(expr))]
    for source in [f'\\\n{expr}', f'if \\\n{expr}: x', f'x if \\\n{expr} else y']:
        code = compile(source, '<test>', 'exec')
        actual_positions = [instruction.positions for instruction in dis.get_instructions(code) if instruction.opname.startswith(opname_prefix)]

        assert actual_positions == expected_positions
print("TestSpecifics::test_compare_positions: ok")
