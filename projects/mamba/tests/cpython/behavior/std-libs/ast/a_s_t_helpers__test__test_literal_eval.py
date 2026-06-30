# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent
maxDiff = None
assert ast.literal_eval('[1, 2, 3]') == [1, 2, 3]
assert ast.literal_eval('{"foo": 42}') == {'foo': 42}
assert ast.literal_eval('(True, False, None)') == (True, False, None)
assert ast.literal_eval('{1, 2, 3}') == {1, 2, 3}
assert ast.literal_eval('b"hi"') == b'hi'
assert ast.literal_eval('set()') == set()
try:
    ast.literal_eval('foo()')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
assert ast.literal_eval('6') == 6
assert ast.literal_eval('+6') == 6
assert ast.literal_eval('-6') == -6
assert ast.literal_eval('3.25') == 3.25
assert ast.literal_eval('+3.25') == 3.25
assert ast.literal_eval('-3.25') == -3.25
assert repr(ast.literal_eval('-0.0')) == '-0.0'
try:
    ast.literal_eval('++6')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('+True')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('2+3')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("ASTHelpers_Test::test_literal_eval: ok")
