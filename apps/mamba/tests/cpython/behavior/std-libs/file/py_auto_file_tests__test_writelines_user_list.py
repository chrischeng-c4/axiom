# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_auto_file_tests__test_writelines_user_list"
# subject = "cpython.test_file.PyAutoFileTests.testWritelinesUserList"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::PyAutoFileTests::testWritelinesUserList
"""Auto-ported test: PyAutoFileTests::testWritelinesUserList (CPython 3.12 oracle)."""


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
l = UserList([b'1', b'2'])
self_f.writelines(l)
self_f.close()
self_f = open(TESTFN, 'rb')
buf = self_f.read()

assert buf == b'12'
print("PyAutoFileTests::testWritelinesUserList: ok")
