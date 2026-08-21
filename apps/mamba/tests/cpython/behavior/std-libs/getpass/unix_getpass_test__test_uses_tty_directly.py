# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "unix_getpass_test__test_uses_tty_directly"
# subject = "cpython.test_getpass.UnixGetpassTest.test_uses_tty_directly"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpass.py::UnixGetpassTest::test_uses_tty_directly
"""Auto-ported test: UnixGetpassTest::test_uses_tty_directly (CPython 3.12 oracle)."""


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
with mock.patch('os.open') as open, mock.patch('io.FileIO') as fileio, mock.patch('io.TextIOWrapper') as textio:
    open.return_value = None
    getpass.unix_getpass()
    open.assert_called_once_with('/dev/tty', os.O_RDWR | os.O_NOCTTY)
    fileio.assert_called_once_with(open.return_value, 'w+')
    textio.assert_called_once_with(fileio.return_value)
print("UnixGetpassTest::test_uses_tty_directly: ok")
