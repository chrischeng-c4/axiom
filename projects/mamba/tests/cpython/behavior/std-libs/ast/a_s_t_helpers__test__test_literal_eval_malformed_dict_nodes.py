# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_literal_eval_malformed_dict_nodes"
# subject = "cpython.test_ast.ASTHelpers_Test.test_literal_eval_malformed_dict_nodes"
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
malformed = ast.Dict(keys=[ast.Constant(1), ast.Constant(2)], values=[ast.Constant(3)])
try:
    ast.literal_eval(malformed)
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass
malformed = ast.Dict(keys=[ast.Constant(1)], values=[ast.Constant(2), ast.Constant(3)])
try:
    ast.literal_eval(malformed)
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("ASTHelpers_Test::test_literal_eval_malformed_dict_nodes: ok")
