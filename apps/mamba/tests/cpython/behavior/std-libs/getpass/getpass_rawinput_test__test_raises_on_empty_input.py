# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "getpass_rawinput_test__test_raises_on_empty_input"
# subject = "cpython.test_getpass.GetpassRawinputTest.test_raises_on_empty_input"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpass.py::GetpassRawinputTest::test_raises_on_empty_input
"""Auto-ported test: GetpassRawinputTest::test_raises_on_empty_input (CPython 3.12 oracle)."""


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
input = StringIO('')

try:
    getpass._raw_input(input=input)
    raise AssertionError('expected EOFError')
except EOFError:
    pass
print("GetpassRawinputTest::test_raises_on_empty_input: ok")
