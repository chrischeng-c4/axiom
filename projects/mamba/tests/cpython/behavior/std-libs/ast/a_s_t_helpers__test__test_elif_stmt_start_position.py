# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_elif_stmt_start_position"
# subject = "cpython.test_ast.ASTHelpers_Test.test_elif_stmt_start_position"
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
node = ast.parse('if a:\n    pass\nelif b:\n    pass\n')
elif_stmt = node.body[0].orelse[0]
assert elif_stmt.lineno == 3
assert elif_stmt.col_offset == 0

print("ASTHelpers_Test::test_elif_stmt_start_position: ok")
