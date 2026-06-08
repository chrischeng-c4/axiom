# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "unix_getpass_test__test_resets_termios"
# subject = "cpython.test_getpass.UnixGetpassTest.test_resets_termios"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpass.py::UnixGetpassTest::test_resets_termios
"""Auto-ported test: UnixGetpassTest::test_resets_termios (CPython 3.12 oracle)."""


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
with mock.patch('os.open') as open, mock.patch('io.FileIO'), mock.patch('io.TextIOWrapper'), mock.patch('termios.tcgetattr') as tcgetattr, mock.patch('termios.tcsetattr') as tcsetattr:
    open.return_value = 3
    fake_attrs = [255, 255, 255, 255, 255]
    tcgetattr.return_value = list(fake_attrs)
    getpass.unix_getpass()
    tcsetattr.assert_called_with(3, mock.ANY, fake_attrs)
print("UnixGetpassTest::test_resets_termios: ok")
