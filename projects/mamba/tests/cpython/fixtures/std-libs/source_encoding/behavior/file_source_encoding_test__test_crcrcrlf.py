# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "file_source_encoding_test__test_crcrcrlf"
# subject = "cpython.test_source_encoding.FileSourceEncodingTest.test_crcrcrlf"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::FileSourceEncodingTest::test_crcrcrlf
"""Auto-ported test: FileSourceEncodingTest::test_crcrcrlf (CPython 3.12 oracle)."""


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
src = b'print(ascii("""\r\r\r\n"""))\n'
out = check_script_output(src, b"'\\n\\n\\n'")
print("FileSourceEncodingTest::test_crcrcrlf: ok")
