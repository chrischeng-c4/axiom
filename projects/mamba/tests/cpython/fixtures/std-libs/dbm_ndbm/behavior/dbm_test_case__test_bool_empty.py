# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_ndbm"
# dimension = "behavior"
# case = "dbm_test_case__test_bool_empty"
# subject = "cpython.test_dbm_ndbm.DbmTestCase.test_bool_empty"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_ndbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_ndbm.py::DbmTestCase::test_bool_empty
"""Auto-ported test: DbmTestCase::test_bool_empty (CPython 3.12 oracle)."""


from test.support import import_helper
from test.support import os_helper
import os
import unittest
import dbm.ndbm
from dbm.ndbm import error


import_helper.import_module('dbm.ndbm')


# --- test body ---
self_filename = os_helper.TESTFN
self_d = dbm.ndbm.open(self_filename, 'c')
self_d.close()
with dbm.ndbm.open(self_filename, 'c') as db:

    assert not bool(db)
print("DbmTestCase::test_bool_empty: ok")
