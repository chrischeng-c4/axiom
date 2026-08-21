# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_expressions_with_triple_quoted_strings"
# subject = "cpython.test_fstring.TestCase.test_expressions_with_triple_quoted_strings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_expressions_with_triple_quoted_strings
"""Auto-ported test: TestCase::test_expressions_with_triple_quoted_strings (CPython 3.12 oracle)."""


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

assert f"{'x'}" == 'x'

assert f"""{"eric's"}""" == "eric's"

assert f"""{'xeric"sy'}""" == 'xeric"sy'

assert f"""{'xeric"s'}""" == 'xeric"s'

assert f"""{'eric"sy'}""" == 'eric"sy'

assert f"""{'xeric"sy'}""" == 'xeric"sy'

assert f"""{'xeric"sy'}""" == 'xeric"sy'

assert f"""{'xeric"sy'}""" == 'xeric"sy'
print("TestCase::test_expressions_with_triple_quoted_strings: ok")
