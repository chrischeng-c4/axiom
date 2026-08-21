# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "misc_source_encoding_test__test_error_from_string"
# subject = "cpython.test_source_encoding.MiscSourceEncodingTest.test_error_from_string"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::MiscSourceEncodingTest::test_error_from_string
"""Auto-ported test: MiscSourceEncodingTest::test_error_from_string (CPython 3.12 oracle)."""


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
input = '# coding: ascii\n☃'.encode('utf-8')
try:
    compile(input, '<string>', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import types as _types_aR
    c = _types_aR.SimpleNamespace(exception=_aR_e)
expected = "'ascii' codec can't decode byte 0xe2 in position 16: ordinal not in range(128)"

assert c.exception.args[0].startswith(expected)
print("MiscSourceEncodingTest::test_error_from_string: ok")
