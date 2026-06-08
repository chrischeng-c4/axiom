# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_errors"
# subject = "cpython.test_file.CAutoFileTests.testErrors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testErrors
"""Auto-ported test: CAutoFileTests::testErrors (CPython 3.12 oracle)."""


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
f = self_f

assert f.name == TESTFN

assert not f.isatty()

assert not f.closed
if hasattr(f, 'readinto'):

    try:
        f.readinto('')
        raise AssertionError('expected (OSError, TypeError)')
    except (OSError, TypeError):
        pass
f.close()

assert f.closed
print("CAutoFileTests::testErrors: ok")
