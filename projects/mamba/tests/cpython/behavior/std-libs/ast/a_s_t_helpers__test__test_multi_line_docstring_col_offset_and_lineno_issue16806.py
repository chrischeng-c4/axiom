# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_multi_line_docstring_col_offset_and_lineno_issue16806"
# subject = "cpython.test_ast.ASTHelpers_Test.test_multi_line_docstring_col_offset_and_lineno_issue16806"
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
node = ast.parse('"""line one\nline two"""\n\ndef foo():\n  """line one\n  line two"""\n\n  def bar():\n    """line one\n    line two"""\n  """line one\n  line two"""\n"""line one\nline two"""\n\n')
assert node.body[0].col_offset == 0
assert node.body[0].lineno == 1
assert node.body[1].body[0].col_offset == 2
assert node.body[1].body[0].lineno == 5
assert node.body[1].body[1].body[0].col_offset == 4
assert node.body[1].body[1].body[0].lineno == 9
assert node.body[1].body[2].col_offset == 2
assert node.body[1].body[2].lineno == 11
assert node.body[2].col_offset == 0
assert node.body[2].lineno == 13

print("ASTHelpers_Test::test_multi_line_docstring_col_offset_and_lineno_issue16806: ok")
