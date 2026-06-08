# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_source_positions__test_multiline_assert_rewritten_as_method_call"
# subject = "cpython.test_compile.TestSourcePositions.test_multiline_assert_rewritten_as_method_call"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSourcePositions::test_multiline_assert_rewritten_as_method_call
"""Auto-ported test: TestSourcePositions::test_multiline_assert_rewritten_as_method_call (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
tree = ast.parse('assert (\n42\n)')
old_node = tree.body[0]
new_node = ast.Expr(ast.Call(ast.Attribute(ast.Name('spam', ast.Load()), 'eggs', ast.Load()), [], []))
ast.copy_location(new_node, old_node)
ast.fix_missing_locations(new_node)
tree.body[0] = new_node
compile(tree, '<test>', 'exec')
print("TestSourcePositions::test_multiline_assert_rewritten_as_method_call: ok")
