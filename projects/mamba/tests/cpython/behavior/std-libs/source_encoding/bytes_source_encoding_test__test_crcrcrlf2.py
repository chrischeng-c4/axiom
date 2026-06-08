# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "bytes_source_encoding_test__test_crcrcrlf2"
# subject = "cpython.test_source_encoding.BytesSourceEncodingTest.test_crcrcrlf2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::BytesSourceEncodingTest::test_crcrcrlf2
"""Auto-ported test: BytesSourceEncodingTest::test_crcrcrlf2 (CPython 3.12 oracle)."""


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
src = b'#coding:iso-8859-1\nprint(ascii("""\r\r\r\n"""))\n'
out = check_script_output(src, b"'\\n\\n\\n'")
print("BytesSourceEncodingTest::test_crcrcrlf2: ok")
