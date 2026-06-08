# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_single_statement"
# subject = "cpython.test_compile.TestSpecifics.test_single_statement"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_single_statement
"""Auto-ported test: TestSpecifics::test_single_statement (CPython 3.12 oracle)."""


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
compile_single('1 + 2')
compile_single('\n1 + 2')
compile_single('1 + 2\n')
compile_single('1 + 2\n\n')
compile_single('1 + 2\t\t\n')
compile_single('1 + 2\t\t\n        ')
compile_single('1 + 2 # one plus two')
compile_single('1; 2')
compile_single('import sys; sys')
compile_single('def f():\n   pass')
compile_single('while False:\n   pass')
compile_single('if x:\n   f(x)')
compile_single('if x:\n   f(x)\nelse:\n   g(x)')
compile_single('class T:\n   pass')
compile_single("c = '''\na=1\nb=2\nc=3\n'''")
print("TestSpecifics::test_single_statement: ok")
