# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_error_conditions"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_error_conditions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_error_conditions
"""Auto-ported test: TestGdbm::test_error_conditions (CPython 3.12 oracle)."""


import os
import unittest
from test import support
from test.support import cpython_only, import_helper
from test.support.os_helper import TESTFN, TESTFN_NONASCII, FakePath, create_empty_file, temp_dir, unlink


gdbm = import_helper.import_module('dbm.gnu')

filename = TESTFN


# --- test body ---
self_g = None
unlink(filename)

try:
    gdbm.open(filename, 'r')
    raise AssertionError('expected gdbm.error')
except gdbm.error:
    pass
self_g = gdbm.open(filename, 'c')
self_g.close()

try:
    (lambda: self_g['a'])()
    raise AssertionError('expected gdbm.error')
except gdbm.error:
    pass

try:
    (lambda: gdbm.open(filename, 'rx').close())()
    raise AssertionError('expected gdbm.error')
except gdbm.error:
    pass
print("TestGdbm::test_error_conditions: ok")
