# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "behavior"
# case = "test_generic_test__test_isfile"
# subject = "cpython.test_genericpath.TestGenericTest.test_isfile"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericpath.py::TestGenericTest::test_isfile
"""Auto-ported test: TestGenericTest::test_isfile (CPython 3.12 oracle)."""


import genericpath
import os
import sys
import unittest
import warnings
from test.support import is_emscripten
from test.support import os_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok
from test.support.os_helper import FakePath


'\nTests common to genericpath, ntpath and posixpath\n'

def create_file(filename, data=b'foo'):
    with open(filename, 'xb', 0) as fp:
        fp.write(data)


# --- test body ---
common_attributes = ['commonprefix', 'getsize', 'getatime', 'getctime', 'getmtime', 'exists', 'isdir', 'isfile']
attributes = []
pathmodule = genericpath
filename = os_helper.TESTFN
bfilename = os.fsencode(filename)

assert pathmodule.isfile(filename) is False

assert pathmodule.isfile(bfilename) is False

assert pathmodule.isfile(filename + '\udfff') is False

assert pathmodule.isfile(bfilename + b'\xff') is False

assert pathmodule.isfile(filename + '\x00') is False

assert pathmodule.isfile(bfilename + b'\x00') is False
try:
    create_file(filename)

    assert pathmodule.isfile(filename) is True

    assert pathmodule.isfile(bfilename) is True
finally:
    os_helper.unlink(filename)
try:
    os.mkdir(filename)

    assert pathmodule.isfile(filename) is False

    assert pathmodule.isfile(bfilename) is False
finally:
    os_helper.rmdir(filename)
print("TestGenericTest::test_isfile: ok")
