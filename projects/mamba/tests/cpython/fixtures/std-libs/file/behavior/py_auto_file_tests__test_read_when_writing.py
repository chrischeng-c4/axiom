# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_auto_file_tests__test_read_when_writing"
# subject = "cpython.test_file.PyAutoFileTests.testReadWhenWriting"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::PyAutoFileTests::testReadWhenWriting
"""Auto-ported test: PyAutoFileTests::testReadWhenWriting (CPython 3.12 oracle)."""


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
    self_f.read()
    raise AssertionError('expected OSError')
except OSError:
    pass
print("PyAutoFileTests::testReadWhenWriting: ok")
