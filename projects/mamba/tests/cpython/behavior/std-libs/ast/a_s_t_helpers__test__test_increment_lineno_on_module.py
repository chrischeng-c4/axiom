# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_increment_lineno_on_module"
# subject = "cpython.test_ast.ASTHelpers_Test.test_increment_lineno_on_module"
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
src = ast.parse(dedent('        a = 1\n        b = 2 # type: ignore\n        c = 3\n        d = 4 # type: ignore@tag\n        '), type_comments=True)
ast.increment_lineno(src, n=5)
assert src.type_ignores[0].lineno == 7
assert src.type_ignores[1].lineno == 9
assert src.type_ignores[1].tag == '@tag'

print("ASTHelpers_Test::test_increment_lineno_on_module: ok")
