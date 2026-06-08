# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval_complex"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval_complex"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
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
assert ast.literal_eval('6j') == 6j
assert ast.literal_eval('-6j') == -6j
assert ast.literal_eval('6.75j') == 6.75j
assert ast.literal_eval('-6.75j') == -6.75j
assert ast.literal_eval('3+6j') == 3 + 6j
assert ast.literal_eval('-3+6j') == -3 + 6j
assert ast.literal_eval('3-6j') == 3 - 6j
assert ast.literal_eval('-3-6j') == -3 - 6j
assert ast.literal_eval('3.25+6.75j') == 3.25 + 6.75j
assert ast.literal_eval('-3.25+6.75j') == -3.25 + 6.75j
assert ast.literal_eval('3.25-6.75j') == 3.25 - 6.75j
assert ast.literal_eval('-3.25-6.75j') == -3.25 - 6.75j
assert ast.literal_eval('(3+6j)') == 3 + 6j
try:
    ast.literal_eval('-6j+3')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('-6j+3j')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('3+-6j')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('3+(0+6j)')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
try:
    ast.literal_eval('-(3+6j)')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("ASTHelpers_Test::test_literal_eval_complex: ok")
