# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_bad_single_statement"
# subject = "cpython.test_compile.TestSpecifics.test_bad_single_statement"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_bad_single_statement
"""Auto-ported test: TestSpecifics::test_bad_single_statement (CPython 3.12 oracle)."""


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
assertInvalidSingle('1\n2')
assertInvalidSingle('def f(): pass')
assertInvalidSingle('a = 13\nb = 187')
assertInvalidSingle('del x\ndel y')
assertInvalidSingle('f()\ng()')
assertInvalidSingle('f()\n# blah\nblah()')
assertInvalidSingle('f()\nxy # blah\nblah()')
assertInvalidSingle('x = 5 # comment\nx = 6\n')
assertInvalidSingle("c = '''\nd=1\n'''\na = 1\n\nb = 2\n")
print("TestSpecifics::test_bad_single_statement: ok")
