# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_future_multiple_imports"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_future_multiple_imports"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_future_multiple_imports
"""Auto-ported test: FutureTest::test_future_multiple_imports (CPython 3.12 oracle)."""


import __future__
import ast
import unittest
from test.support import import_helper
from test.support.script_helper import spawn_python, kill_python
from textwrap import dedent
import os
import re
import sys


rx = re.compile('\\((\\S+).py, line (\\d+)')

def get_error_location(msg):
    mo = rx.search(str(msg))
    return mo.group(1, 2)


# --- test body ---
with import_helper.CleanImport('test.test_future_stmt.test_future_multiple_imports'):
    from test.test_future_stmt import test_future_multiple_imports
print("FutureTest::test_future_multiple_imports: ok")
