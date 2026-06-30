# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval_trailing_ws"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval_trailing_ws"
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
assert ast.literal_eval('    -1') == -1
assert ast.literal_eval('\t\t-1') == -1
assert ast.literal_eval(' \t -1') == -1
try:
    ast.literal_eval('\n -1')
    raise AssertionError('assertRaises: no raise')
except IndentationError:
    pass

print("ASTHelpers_Test::test_literal_eval_trailing_ws: ok")
