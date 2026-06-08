# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_level_as_none"
# subject = "cpython.test_ast.ASTHelpers_Test.test_level_as_none"
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
body = [ast.ImportFrom(module='time', names=[ast.alias(name='sleep', lineno=0, col_offset=0)], level=None, lineno=0, col_offset=0)]
mod = ast.Module(body, [])
code = compile(mod, 'test', 'exec')
ns = {}
exec(code, ns)
assert 'sleep' in ns

print("ASTHelpers_Test::test_level_as_none: ok")
