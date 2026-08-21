# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_same_filename_used"
# subject = "cpython.test_compile.TestSpecifics.test_same_filename_used"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_same_filename_used
"""Auto-ported test: TestSpecifics::test_same_filename_used (CPython 3.12 oracle)."""


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
s = 'def f(): pass\ndef g(): pass'
c = compile(s, 'myfile', 'exec')
for obj in c.co_consts:
    if isinstance(obj, types.CodeType):

        assert obj.co_filename is c.co_filename
print("TestSpecifics::test_same_filename_used: ok")
