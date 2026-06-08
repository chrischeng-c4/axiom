# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_compile_invalid_namedexpr"
# subject = "cpython.test_compile.TestSpecifics.test_compile_invalid_namedexpr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_compile_invalid_namedexpr
"""Auto-ported test: TestSpecifics::test_compile_invalid_namedexpr (CPython 3.12 oracle)."""


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
m = ast.Module(body=[ast.Expr(value=ast.ListComp(elt=ast.NamedExpr(target=ast.Constant(value=1), value=ast.Constant(value=3)), generators=[ast.comprehension(target=ast.Name(id='x', ctx=ast.Store()), iter=ast.Name(id='y', ctx=ast.Load()), ifs=[], is_async=0)]))], type_ignores=[])
try:
    compile(ast.fix_missing_locations(m), '<file>', 'exec')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('NamedExpr target must be a Name', str(_aR_e))
print("TestSpecifics::test_compile_invalid_namedexpr: ok")
