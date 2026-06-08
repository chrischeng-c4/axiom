# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_imported_load_method"
# subject = "cpython.test_compile.TestSpecifics.test_imported_load_method"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_imported_load_method
"""Auto-ported test: TestSpecifics::test_imported_load_method (CPython 3.12 oracle)."""


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
sources = ['            import os\n            def foo():\n                return os.uname()\n            ', '            import os as operating_system\n            def foo():\n                return operating_system.uname()\n            ', '            from os import path\n            def foo(x):\n                return path.join(x)\n            ', '            from os import path as os_path\n            def foo(x):\n                return os_path.join(x)\n            ']
for source in sources:
    namespace = {}
    exec(textwrap.dedent(source), namespace)
    func = namespace['foo']
    opcodes = list(dis.get_instructions(func))
    instructions = [opcode.opname for opcode in opcodes]

    assert 'LOAD_METHOD' not in instructions

    assert 'LOAD_ATTR' in instructions

    assert 'CALL' in instructions
print("TestSpecifics::test_imported_load_method: ok")
