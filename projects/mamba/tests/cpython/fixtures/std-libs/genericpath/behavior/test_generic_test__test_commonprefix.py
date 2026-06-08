# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "behavior"
# case = "test_generic_test__test_commonprefix"
# subject = "cpython.test_genericpath.TestGenericTest.test_commonprefix"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericpath.py::TestGenericTest::test_commonprefix
"""Auto-ported test: TestGenericTest::test_commonprefix (CPython 3.12 oracle)."""


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
commonprefix = pathmodule.commonprefix

assert commonprefix([]) == ''

assert commonprefix(['/home/swenson/spam', '/home/swen/spam']) == '/home/swen'

assert commonprefix(['/home/swen/spam', '/home/swen/eggs']) == '/home/swen/'

assert commonprefix(['/home/swen/spam', '/home/swen/spam']) == '/home/swen/spam'

assert commonprefix(['home:swenson:spam', 'home:swen:spam']) == 'home:swen'

assert commonprefix([':home:swen:spam', ':home:swen:eggs']) == ':home:swen:'

assert commonprefix([':home:swen:spam', ':home:swen:spam']) == ':home:swen:spam'

assert commonprefix([b'/home/swenson/spam', b'/home/swen/spam']) == b'/home/swen'

assert commonprefix([b'/home/swen/spam', b'/home/swen/eggs']) == b'/home/swen/'

assert commonprefix([b'/home/swen/spam', b'/home/swen/spam']) == b'/home/swen/spam'

assert commonprefix([b'home:swenson:spam', b'home:swen:spam']) == b'home:swen'

assert commonprefix([b':home:swen:spam', b':home:swen:eggs']) == b':home:swen:'

assert commonprefix([b':home:swen:spam', b':home:swen:spam']) == b':home:swen:spam'
testlist = ['', 'abc', 'Xbcd', 'Xb', 'XY', 'abcd', 'aXc', 'abd', 'ab', 'aX', 'abcX']
for s1 in testlist:
    for s2 in testlist:
        p = commonprefix([s1, s2])

        assert s1.startswith(p)

        assert s2.startswith(p)
        if s1 != s2:
            n = len(p)

            assert s1[n:n + 1] != s2[n:n + 1]
print("TestGenericTest::test_commonprefix: ok")
