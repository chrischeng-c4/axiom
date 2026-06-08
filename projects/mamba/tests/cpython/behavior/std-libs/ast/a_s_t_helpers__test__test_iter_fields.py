# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_iter_fields"
# subject = "cpython.test_ast.ASTHelpers_Test.test_iter_fields"
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
node = ast.parse('foo()', mode='eval')
d = dict(ast.iter_fields(node.body))
assert d.pop('func').id == 'foo'
assert d == {'keywords': [], 'args': []}

print("ASTHelpers_Test::test_iter_fields: ok")
