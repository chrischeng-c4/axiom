# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "getpass_rawinput_test__test_uses_stderr_as_default"
# subject = "cpython.test_getpass.GetpassRawinputTest.test_uses_stderr_as_default"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpass.py::GetpassRawinputTest::test_uses_stderr_as_default
"""Auto-ported test: GetpassRawinputTest::test_uses_stderr_as_default (CPython 3.12 oracle)."""


import getpass
import os
import unittest
from io import BytesIO, StringIO, TextIOWrapper
from unittest import mock
from test import support


try:
    import termios
except ImportError:
    termios = None

try:
    import pwd
except ImportError:
    pwd = None


# --- test body ---
input = StringIO('input_string')
prompt = 'some_prompt'
with mock.patch('sys.stderr') as stderr:
    getpass._raw_input(prompt, input=input)
    stderr.write.assert_called_once_with(prompt)
print("GetpassRawinputTest::test_uses_stderr_as_default: ok")
