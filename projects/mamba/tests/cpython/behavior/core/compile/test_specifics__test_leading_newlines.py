# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_leading_newlines"
# subject = "cpython.test_compile.TestSpecifics.test_leading_newlines"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_leading_newlines
"""Auto-ported test: TestSpecifics::test_leading_newlines (CPython 3.12 oracle)."""


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
s256 = ''.join(['\n'] * 256 + ['spam'])
co = compile(s256, 'fn', 'exec')

assert co.co_firstlineno == 1
lines = [line for _, _, line in co.co_lines()]

assert lines == [0, 257]
print("TestSpecifics::test_leading_newlines: ok")
