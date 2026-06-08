# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_fstring_format_spec_greedy_matching"
# subject = "cpython.test_fstring.TestCase.test_fstring_format_spec_greedy_matching"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_fstring_format_spec_greedy_matching
"""Auto-ported test: TestCase::test_fstring_format_spec_greedy_matching (CPython 3.12 oracle)."""


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

assert f'{1:}}}' == '1}'

assert f'{1:>3{5}}}}' == '                                  1}'
print("TestCase::test_fstring_format_spec_greedy_matching: ok")
