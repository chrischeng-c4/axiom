# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "misc_source_encoding_test__test_issue3297"
# subject = "cpython.test_source_encoding.MiscSourceEncodingTest.test_issue3297"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::MiscSourceEncodingTest::test_issue3297
"""Auto-ported test: MiscSourceEncodingTest::test_issue3297 (CPython 3.12 oracle)."""


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
c = compile("a, b = '𐄏', '\\U0001010F'", 'dummy', 'exec')
d = {}
exec(c, d)

assert d['a'] == d['b']

assert len(d['a']) == len(d['b'])

assert ascii(d['a']) == ascii(d['b'])
print("MiscSourceEncodingTest::test_issue3297: ok")
