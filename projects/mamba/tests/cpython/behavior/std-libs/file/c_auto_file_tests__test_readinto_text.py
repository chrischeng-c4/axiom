# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_readinto_text"
# subject = "cpython.test_file.CAutoFileTests.testReadinto_text"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testReadinto_text
"""Auto-ported test: CAutoFileTests::testReadinto_text (CPython 3.12 oracle)."""


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
a = array('b', b'x' * 10)
self_f.close()
self_f = open(TESTFN, encoding='utf-8')
if hasattr(self_f, 'readinto'):

    try:
        self_f.readinto(a)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("CAutoFileTests::testReadinto_text: ok")
