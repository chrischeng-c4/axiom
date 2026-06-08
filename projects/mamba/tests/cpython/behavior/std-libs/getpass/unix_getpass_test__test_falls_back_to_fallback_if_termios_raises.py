# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "unix_getpass_test__test_falls_back_to_fallback_if_termios_raises"
# subject = "cpython.test_getpass.UnixGetpassTest.test_falls_back_to_fallback_if_termios_raises"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpass.py::UnixGetpassTest::test_falls_back_to_fallback_if_termios_raises
"""Auto-ported test: UnixGetpassTest::test_falls_back_to_fallback_if_termios_raises (CPython 3.12 oracle)."""


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
with mock.patch('os.open') as open, mock.patch('io.FileIO') as fileio, mock.patch('io.TextIOWrapper') as textio, mock.patch('termios.tcgetattr'), mock.patch('termios.tcsetattr') as tcsetattr, mock.patch('getpass.fallback_getpass') as fallback:
    open.return_value = 3
    fileio.return_value = BytesIO()
    tcsetattr.side_effect = termios.error
    getpass.unix_getpass()
    fallback.assert_called_once_with('Password: ', textio.return_value)
print("UnixGetpassTest::test_falls_back_to_fallback_if_termios_raises: ok")
