# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_dict"
# subject = "cpython.test_fstring.TestCase.test_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_dict
"""Auto-ported test: TestCase::test_dict (CPython 3.12 oracle)."""


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
d = {'"': 'dquote', "'": 'squote', 'foo': 'bar'}

assert f"""{d["'"]}""" == 'squote'

assert f"""{d['"']}""" == 'dquote'

assert f"{d['foo']}" == 'bar'

assert f"{d['foo']}" == 'bar'
print("TestCase::test_dict: ok")
