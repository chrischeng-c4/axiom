# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_localized_error"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_localized_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_localized_error
"""Auto-ported test: TestGdbm::test_localized_error (CPython 3.12 oracle)."""


import os
import unittest
from test import support
from test.support import cpython_only, import_helper
from test.support.os_helper import TESTFN, TESTFN_NONASCII, FakePath, create_empty_file, temp_dir, unlink


gdbm = import_helper.import_module('dbm.gnu')

filename = TESTFN


# --- test body ---
self_g = None
with temp_dir() as d:
    create_empty_file(os.path.join(d, 'test'))

    try:
        gdbm.open(filename, 'r')
        raise AssertionError('expected gdbm.error')
    except gdbm.error:
        pass
print("TestGdbm::test_localized_error: ok")
