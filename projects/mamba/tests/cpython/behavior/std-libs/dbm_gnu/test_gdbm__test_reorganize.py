# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_reorganize"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_reorganize"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_reorganize
"""Auto-ported test: TestGdbm::test_reorganize (CPython 3.12 oracle)."""


import os
import unittest
from test import support
from test.support import cpython_only, import_helper
from test.support.os_helper import TESTFN, TESTFN_NONASCII, FakePath, create_empty_file, temp_dir, unlink


gdbm = import_helper.import_module('dbm.gnu')

filename = TESTFN


# --- test body ---
self_g = None
self_g = gdbm.open(filename, 'c')
size0 = os.path.getsize(filename)
value_size = max(size0, 10000)
self_g['x'] = 'x' * value_size
size1 = os.path.getsize(filename)

assert size1 > size0
del self_g['x']

assert os.path.getsize(filename) == size1
self_g.reorganize()
size2 = os.path.getsize(filename)

assert size2 < size1

assert size2 >= size0
print("TestGdbm::test_reorganize: ok")
