# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "behavior"
# case = "test_generic_test__test_filetime"
# subject = "cpython.test_genericpath.TestGenericTest.test_filetime"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericpath.py::TestGenericTest::test_filetime
"""Auto-ported test: TestGenericTest::test_filetime (CPython 3.12 oracle)."""


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

def _test_samefile_on_link_func(func):
    test_fn1 = os_helper.TESTFN
    test_fn2 = os_helper.TESTFN + '2'
    pass
    pass
    create_file(test_fn1)
    func(test_fn1, test_fn2)

    assert pathmodule.samefile(test_fn1, test_fn2)
    os.remove(test_fn2)
    create_file(test_fn2)

    assert not pathmodule.samefile(test_fn1, test_fn2)

def _test_samestat_on_link_func(func):
    test_fn1 = os_helper.TESTFN + '1'
    test_fn2 = os_helper.TESTFN + '2'
    pass
    pass
    create_file(test_fn1)
    func(test_fn1, test_fn2)

    assert pathmodule.samestat(os.stat(test_fn1), os.stat(test_fn2))
    os.remove(test_fn2)
    create_file(test_fn2)

    assert not pathmodule.samestat(os.stat(test_fn1), os.stat(test_fn2))
filename = os_helper.TESTFN
pass
create_file(filename, b'foo')
with open(filename, 'ab', 0) as f:
    f.write(b'bar')
with open(filename, 'rb', 0) as f:
    data = f.read()

assert data == b'foobar'

assert pathmodule.getctime(filename) <= pathmodule.getmtime(filename)
print("TestGenericTest::test_filetime: ok")
