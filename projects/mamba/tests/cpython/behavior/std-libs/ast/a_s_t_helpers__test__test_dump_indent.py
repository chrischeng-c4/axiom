# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_dump_indent"
# subject = "cpython.test_ast.ASTHelpers_Test.test_dump_indent"
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
node = ast.parse('spam(eggs, "and cheese")')
assert ast.dump(node, indent=3) == "Module(\n   body=[\n      Expr(\n         value=Call(\n            func=Name(id='spam', ctx=Load()),\n            args=[\n               Name(id='eggs', ctx=Load()),\n               Constant(value='and cheese')],\n            keywords=[]))],\n   type_ignores=[])"
assert ast.dump(node, annotate_fields=False, indent='\t') == "Module(\n\t[\n\t\tExpr(\n\t\t\tCall(\n\t\t\t\tName('spam', Load()),\n\t\t\t\t[\n\t\t\t\t\tName('eggs', Load()),\n\t\t\t\t\tConstant('and cheese')],\n\t\t\t\t[]))],\n\t[])"
assert ast.dump(node, include_attributes=True, indent=3) == "Module(\n   body=[\n      Expr(\n         value=Call(\n            func=Name(\n               id='spam',\n               ctx=Load(),\n               lineno=1,\n               col_offset=0,\n               end_lineno=1,\n               end_col_offset=4),\n            args=[\n               Name(\n                  id='eggs',\n                  ctx=Load(),\n                  lineno=1,\n                  col_offset=5,\n                  end_lineno=1,\n                  end_col_offset=9),\n               Constant(\n                  value='and cheese',\n                  lineno=1,\n                  col_offset=11,\n                  end_lineno=1,\n                  end_col_offset=23)],\n            keywords=[],\n            lineno=1,\n            col_offset=0,\n            end_lineno=1,\n            end_col_offset=24),\n         lineno=1,\n         col_offset=0,\n         end_lineno=1,\n         end_col_offset=24)],\n   type_ignores=[])"

print("ASTHelpers_Test::test_dump_indent: ok")
