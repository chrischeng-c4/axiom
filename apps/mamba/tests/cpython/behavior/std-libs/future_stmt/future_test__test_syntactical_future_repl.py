# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_syntactical_future_repl"
# subject = "cpython.test.test_future_stmt.test_future.FutureTest.test_syntactical_future_repl"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future.py::FutureTest::test_syntactical_future_repl
"""Auto-ported test: FutureTest::test_syntactical_future_repl (CPython 3.12 oracle)."""


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
p = spawn_python('-i')
p.stdin.write(b'from __future__ import barry_as_FLUFL\n')
p.stdin.write(b'2 <> 3\n')
out = kill_python(p)

assert b'SyntaxError: invalid syntax' not in out
print("FutureTest::test_syntactical_future_repl: ok")
