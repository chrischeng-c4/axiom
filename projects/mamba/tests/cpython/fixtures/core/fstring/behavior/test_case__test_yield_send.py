# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_yield_send"
# subject = "cpython.test_fstring.TestCase.test_yield_send"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_yield_send
"""Auto-ported test: TestCase::test_yield_send (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
def fn(x):
    yield f'x:{(yield (lambda i: x * i))}'
g = fn(10)
the_lambda = next(g)

assert the_lambda(4) == 40

assert g.send('string') == 'x:string'
print("TestCase::test_yield_send: ok")
