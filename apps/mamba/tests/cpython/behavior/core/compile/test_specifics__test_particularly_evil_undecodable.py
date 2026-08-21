# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_specifics__test_particularly_evil_undecodable"
# subject = "cpython.test_compile.TestSpecifics.test_particularly_evil_undecodable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestSpecifics::test_particularly_evil_undecodable
"""Auto-ported test: TestSpecifics::test_particularly_evil_undecodable (CPython 3.12 oracle)."""


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
src = b'0000\x00\n00000000000\n\x00\n\x9e\n'
with tempfile.TemporaryDirectory() as tmpd:
    fn = os.path.join(tmpd, 'bad.py')
    with open(fn, 'wb') as fp:
        fp.write(src)
    res = script_helper.run_python_until_end(fn)[0]

assert b'source code cannot contain null bytes' in res.err
print("TestSpecifics::test_particularly_evil_undecodable: ok")
