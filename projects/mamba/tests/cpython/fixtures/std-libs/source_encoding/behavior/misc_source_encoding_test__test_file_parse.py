# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "misc_source_encoding_test__test_file_parse"
# subject = "cpython.test_source_encoding.MiscSourceEncodingTest.test_file_parse"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::MiscSourceEncodingTest::test_file_parse
"""Auto-ported test: MiscSourceEncodingTest::test_file_parse (CPython 3.12 oracle)."""


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
unload(TESTFN)
filename = TESTFN + '.py'
f = open(filename, 'w', encoding='cp1252')
sys.path.insert(0, os.curdir)
try:
    with f:
        f.write('# -*- coding: cp1252 -*-\n')
        f.write("'''A short string\n")
        f.write("'''\n")
        f.write("'A very long string %s'\n" % ('X' * 1000))
    importlib.invalidate_caches()
    __import__(TESTFN)
finally:
    del sys.path[0]
    unlink(filename)
    unlink(filename + 'c')
    unlink(filename + 'o')
    unload(TESTFN)
    rmtree('__pycache__')
print("MiscSourceEncodingTest::test_file_parse: ok")
