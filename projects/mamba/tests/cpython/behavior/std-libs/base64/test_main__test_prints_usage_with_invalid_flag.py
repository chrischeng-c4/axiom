# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "test_main__test_prints_usage_with_invalid_flag"
# subject = "cpython.test_base64.TestMain.test_prints_usage_with_invalid_flag"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base64.py::TestMain::test_prints_usage_with_invalid_flag
"""Auto-ported test: TestMain::test_prints_usage_with_invalid_flag (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---
output = script_helper.assert_python_failure('-m', 'base64', '-x').err

assert b'usage: ' in output

assert b'-d, -u: decode' in output
print("TestMain::test_prints_usage_with_invalid_flag: ok")
