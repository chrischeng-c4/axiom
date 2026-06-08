# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "bytes_source_encoding_test__test_double_coding_line"
# subject = "cpython.test_source_encoding.BytesSourceEncodingTest.test_double_coding_line"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::BytesSourceEncodingTest::test_double_coding_line
"""Auto-ported test: BytesSourceEncodingTest::test_double_coding_line (CPython 3.12 oracle)."""


import unittest
from test.support import script_helper, captured_stdout, requires_subprocess, requires_resource
from test.support.os_helper import TESTFN, unlink, rmtree
from test.support.import_helper import unload
import importlib
import os
import sys
import subprocess
import tempfile


# --- test body ---
def check_script_output(src, expected):
    with captured_stdout() as stdout:
        exec(src)
    out = stdout.getvalue().encode('latin1')

    assert out.rstrip() == expected
src = b'#coding:iso8859-15\n#coding:latin1\nprint(ascii("\xc3\xa4"))\n'
check_script_output(src, b"'\\xc3\\u20ac'")
print("BytesSourceEncodingTest::test_double_coding_line: ok")
