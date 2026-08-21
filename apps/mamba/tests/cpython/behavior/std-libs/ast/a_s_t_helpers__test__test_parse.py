# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_parse"
# subject = "cpython.test_ast.ASTHelpers_Test.test_parse"
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
a = ast.parse('foo(1 + 1)')
b = compile('foo(1 + 1)', '<unknown>', 'exec', ast.PyCF_ONLY_AST)
assert ast.dump(a) == ast.dump(b)

print("ASTHelpers_Test::test_parse: ok")
