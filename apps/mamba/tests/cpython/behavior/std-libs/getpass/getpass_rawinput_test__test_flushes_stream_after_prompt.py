# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "getpass_rawinput_test__test_flushes_stream_after_prompt"
# subject = "cpython.test_getpass.GetpassRawinputTest.test_flushes_stream_after_prompt"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpass.py::GetpassRawinputTest::test_flushes_stream_after_prompt
"""Auto-ported test: GetpassRawinputTest::test_flushes_stream_after_prompt (CPython 3.12 oracle)."""


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
stream = mock.Mock(spec=StringIO)
input = StringIO('input_string')
getpass._raw_input('some_prompt', stream, input=input)
stream.flush.assert_called_once_with()
print("GetpassRawinputTest::test_flushes_stream_after_prompt: ok")
