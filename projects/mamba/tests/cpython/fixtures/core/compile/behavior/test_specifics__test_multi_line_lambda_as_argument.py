# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_multi_line_lambda_as_argument"
# subject = "cpython.test_compile.TestSpecifics.test_multi_line_lambda_as_argument"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_multi_line_lambda_as_argument
"""Auto-ported test: TestSpecifics::test_multi_line_lambda_as_argument (CPython 3.12 oracle)."""


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
code = textwrap.dedent('\n            def foo(param, lambda_exp):\n                pass\n\n            foo(param=0,\n                lambda_exp=lambda:\n                1)\n        ')
compile(code, '<test>', 'exec')
print("TestSpecifics::test_multi_line_lambda_as_argument: ok")
