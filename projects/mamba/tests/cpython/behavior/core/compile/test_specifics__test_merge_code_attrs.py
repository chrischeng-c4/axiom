# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_merge_code_attrs"
# subject = "cpython.test_compile.TestSpecifics.test_merge_code_attrs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_merge_code_attrs
"""Auto-ported test: TestSpecifics::test_merge_code_attrs (CPython 3.12 oracle)."""


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
f1 = lambda x: x.y.z
f2 = lambda a: a.b.c

assert f1.__code__.co_linetable is f2.__code__.co_linetable
print("TestSpecifics::test_merge_code_attrs: ok")
