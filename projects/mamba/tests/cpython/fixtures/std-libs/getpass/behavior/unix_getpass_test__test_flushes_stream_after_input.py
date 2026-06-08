# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "unix_getpass_test__test_flushes_stream_after_input"
# subject = "cpython.test_getpass.UnixGetpassTest.test_flushes_stream_after_input"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpass.py::UnixGetpassTest::test_flushes_stream_after_input
"""Auto-ported test: UnixGetpassTest::test_flushes_stream_after_input (CPython 3.12 oracle)."""


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
with mock.patch('os.open') as open, mock.patch('io.FileIO'), mock.patch('io.TextIOWrapper'), mock.patch('termios.tcgetattr'), mock.patch('termios.tcsetattr'):
    open.return_value = 3
    mock_stream = mock.Mock(spec=StringIO)
    getpass.unix_getpass(stream=mock_stream)
    mock_stream.flush.assert_called_with()
print("UnixGetpassTest::test_flushes_stream_after_input: ok")
