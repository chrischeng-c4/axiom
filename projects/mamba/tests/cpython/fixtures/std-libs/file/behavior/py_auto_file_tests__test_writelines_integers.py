# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_auto_file_tests__test_writelines_integers"
# subject = "cpython.test_file.PyAutoFileTests.testWritelinesIntegers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::PyAutoFileTests::testWritelinesIntegers
"""Auto-ported test: PyAutoFileTests::testWritelinesIntegers (CPython 3.12 oracle)."""


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
self_f = open(TESTFN, 'wb')

try:
    self_f.writelines([1, 2, 3])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PyAutoFileTests::testWritelinesIntegers: ok")
