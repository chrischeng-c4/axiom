# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_consts_in_conditionals"
# subject = "cpython.test_compile.TestSpecifics.test_consts_in_conditionals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_consts_in_conditionals
"""Auto-ported test: TestSpecifics::test_consts_in_conditionals (CPython 3.12 oracle)."""


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

def and_true(x):
    return True and x

def and_false(x):
    return False and x

def or_true(x):
    return True or x

def or_false(x):
    return False or x
funcs = [and_true, and_false, or_true, or_false]
for func in funcs:
    opcodes = list(dis.get_instructions(func))

    assert len(opcodes) <= 3

    assert 'LOAD_' in opcodes[-2].opname

    assert 'RETURN_VALUE' == opcodes[-1].opname
print("TestSpecifics::test_consts_in_conditionals: ok")
