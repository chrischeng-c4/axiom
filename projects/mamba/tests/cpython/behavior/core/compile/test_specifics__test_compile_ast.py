# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_compile_ast"
# subject = "cpython.test_compile.TestSpecifics.test_compile_ast"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_compile_ast
"""Auto-ported test: TestSpecifics::test_compile_ast (CPython 3.12 oracle)."""


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
fname = __file__
if fname.lower().endswith('pyc'):
    fname = fname[:-1]
with open(fname, encoding='utf-8') as f:
    fcontents = f.read()
sample_code = [['<assign>', 'x = 5'], ['<ifblock>', 'if True:\n    pass\n'], ['<forblock>', 'for n in [1, 2, 3]:\n    print(n)\n'], ['<deffunc>', 'def foo():\n    pass\nfoo()\n'], [fname, fcontents]]
for fname, code in sample_code:
    co1 = compile(code, '%s1' % fname, 'exec')
    ast = compile(code, '%s2' % fname, 'exec', _ast.PyCF_ONLY_AST)

    assert type(ast) == _ast.Module
    co2 = compile(ast, '%s3' % fname, 'exec')

    assert co1 == co2

    assert co2.co_filename == '%s3' % fname
co1 = compile('print(1)', '<string>', 'exec', _ast.PyCF_ONLY_AST)

try:
    compile(co1, '<ast>', 'eval')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    compile(_ast.If(), '<ast>', 'exec')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
ast = _ast.Module()
ast.body = [_ast.BoolOp()]

try:
    compile(ast, '<ast>', 'exec')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestSpecifics::test_compile_ast: ok")
