# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_readinto"
# subject = "cpython.test_file.CAutoFileTests.testReadinto"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testReadinto
"""Auto-ported test: CAutoFileTests::testReadinto (CPython 3.12 oracle)."""


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
self_f.write(b'12')
self_f.close()
a = array('b', b'x' * 10)
self_f = open(TESTFN, 'rb')
n = self_f.readinto(a)

assert b'12' == a.tobytes()[:n]
print("CAutoFileTests::testReadinto: ok")
