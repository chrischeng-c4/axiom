# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "misc_source_encoding_test__test_file_parse_error_multiline"
# subject = "cpython.test_source_encoding.MiscSourceEncodingTest.test_file_parse_error_multiline"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::MiscSourceEncodingTest::test_file_parse_error_multiline
"""Auto-ported test: MiscSourceEncodingTest::test_file_parse_error_multiline (CPython 3.12 oracle)."""


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
def verify_bad_module(module_name):

    try:
        __import__('test.tokenizedata.' + module_name)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
    path = os.path.dirname(__file__)
    filename = os.path.join(path, 'tokenizedata', module_name + '.py')
    with open(filename, 'rb') as fp:
        bytes = fp.read()

    try:
        compile(bytes, filename, 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
with open(TESTFN, 'wb') as fd:
    fd.write(b'print("""\n\xb1""")\n')
try:
    retcode, stdout, stderr = script_helper.assert_python_failure(TESTFN)

    assert retcode > 0

    assert b"Non-UTF-8 code starting with '\\xb1'" in stderr
finally:
    os.unlink(TESTFN)
print("MiscSourceEncodingTest::test_file_parse_error_multiline: ok")
