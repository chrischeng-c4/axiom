# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_weak_refs"
# subject = "cpython.test_file.CAutoFileTests.testWeakRefs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testWeakRefs
"""Auto-ported test: CAutoFileTests::testWeakRefs (CPython 3.12 oracle)."""


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
p = proxy(self_f)
p.write(b'teststring')

assert self_f.tell() == p.tell()
self_f.close()
self_f = None
gc_collect()

try:
    getattr(p, 'tell')
    raise AssertionError('expected ReferenceError')
except ReferenceError:
    pass
print("CAutoFileTests::testWeakRefs: ok")
