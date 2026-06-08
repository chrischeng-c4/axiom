# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_writelines_integers_user_list"
# subject = "cpython.test_file.CAutoFileTests.testWritelinesIntegersUserList"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testWritelinesIntegersUserList
"""Auto-ported test: CAutoFileTests::testWritelinesIntegersUserList (CPython 3.12 oracle)."""


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
open = io.open
self_f = open(TESTFN, 'wb')
l = UserList([1, 2, 3])

try:
    self_f.writelines(l)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("CAutoFileTests::testWritelinesIntegersUserList: ok")
