# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "file_source_encoding_test__test_second_coding_line"
# subject = "cpython.test_source_encoding.FileSourceEncodingTest.test_second_coding_line"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::FileSourceEncodingTest::test_second_coding_line
"""Auto-ported test: FileSourceEncodingTest::test_second_coding_line (CPython 3.12 oracle)."""


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
    with tempfile.TemporaryDirectory() as tmpd:
        fn = os.path.join(tmpd, 'test.py')
        with open(fn, 'wb') as fp:
            fp.write(src)
        res = script_helper.assert_python_ok(fn)

    assert res.out.rstrip() == expected
src = b'#\n#coding:iso8859-15\nprint(ascii("\xc3\xa4"))\n'
check_script_output(src, b"'\\xc3\\u20ac'")
print("FileSourceEncodingTest::test_second_coding_line: ok")
