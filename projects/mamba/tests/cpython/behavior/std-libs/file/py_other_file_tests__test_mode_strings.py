# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_other_file_tests__test_mode_strings"
# subject = "cpython.test_file.PyOtherFileTests.testModeStrings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::PyOtherFileTests::testModeStrings
"""Auto-ported test: PyOtherFileTests::testModeStrings (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = staticmethod(pyio.open)

def _checkBufferSize(s):
    try:
        f = open(TESTFN, 'wb', s)
        f.write(str(s).encode('ascii'))
        f.close()
        f.close()
        f = open(TESTFN, 'rb', s)
        d = int(f.read().decode('ascii'))
        f.close()
        f.close()
    except OSError as msg:

        raise AssertionError('error setting buffer size %d: %s' % (s, str(msg)))

    assert d == s
open(TESTFN, 'wb').close()
for mode in ('', 'aU', 'wU+', 'U+', '+U', 'rU+'):
    try:
        f = open(TESTFN, mode)
    except ValueError:
        pass
    else:
        f.close()

        raise AssertionError('%r is an invalid file mode' % mode)
print("PyOtherFileTests::testModeStrings: ok")
