# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_methods"
# subject = "cpython.test_file.CAutoFileTests.testMethods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testMethods
"""Auto-ported test: CAutoFileTests::testMethods (CPython 3.12 oracle)."""


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
methods = [('fileno', ()), ('flush', ()), ('isatty', ()), ('__next__', ()), ('read', ()), ('write', (b'',)), ('readline', ()), ('readlines', ()), ('seek', (0,)), ('tell', ()), ('write', (b'',)), ('writelines', ([],)), ('__iter__', ())]
methods.append(('truncate', ()))
self_f.__exit__(None, None, None)

assert self_f.closed
for methodname, args in methods:
    method = getattr(self_f, methodname)

    try:
        method(*args)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

assert self_f.__exit__(None, None, None) == None
try:
    1 / 0
except:

    assert self_f.__exit__(*sys.exc_info()) == None
print("CAutoFileTests::testMethods: ok")
