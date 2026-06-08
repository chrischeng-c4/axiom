# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_source_positions__test_weird_attribute_position_regressions"
# subject = "cpython.test_compile.TestSourcePositions.test_weird_attribute_position_regressions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSourcePositions::test_weird_attribute_position_regressions
"""Auto-ported test: TestSourcePositions::test_weird_attribute_position_regressions (CPython 3.12 oracle)."""


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
def f():
    bar.baz
    bar.baz()
    files().setdefault(0).setdefault(0)
for line, end_line, column, end_column in f.__code__.co_positions():

    assert line is not None

    assert end_line is not None

    assert column is not None

    assert end_column is not None

    assert (line, column) <= (end_line, end_column)
print("TestSourcePositions::test_weird_attribute_position_regressions: ok")
