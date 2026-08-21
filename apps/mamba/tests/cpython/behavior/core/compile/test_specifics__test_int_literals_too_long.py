# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_int_literals_too_long"
# subject = "cpython.test_compile.TestSpecifics.test_int_literals_too_long"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_int_literals_too_long
"""Auto-ported test: TestSpecifics::test_int_literals_too_long (CPython 3.12 oracle)."""


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
n = 3000
source = f"a = 1\nb = 2\nc = {'3' * n}\nd = 4"
with support.adjust_int_max_str_digits(n):
    compile(source, '<long_int_pass>', 'exec')
with support.adjust_int_max_str_digits(n - 1):
    try:
        compile(source, '<long_int_fail>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import types as _types_aR
        err_ctx = _types_aR.SimpleNamespace(exception=_aR_e)
    exc = err_ctx.exception

    assert exc.lineno == 3

    assert 'Exceeds the limit ' in str(exc)

    assert ' Consider hexadecimal ' in str(exc)
print("TestSpecifics::test_int_literals_too_long: ok")
