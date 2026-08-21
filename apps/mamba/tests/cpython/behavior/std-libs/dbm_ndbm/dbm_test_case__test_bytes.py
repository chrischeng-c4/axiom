# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_ndbm"
# dimension = "behavior"
# case = "dbm_test_case__test_bytes"
# subject = "cpython.test_dbm_ndbm.DbmTestCase.test_bytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_ndbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_ndbm.py::DbmTestCase::test_bytes
"""Auto-ported test: DbmTestCase::test_bytes (CPython 3.12 oracle)."""


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
    db[b'bytes key \xbd'] = b'bytes value \xbd'
with dbm.ndbm.open(self_filename, 'r') as db:

    assert list(db.keys()) == [b'bytes key \xbd']

    assert b'bytes key \xbd' in db

    assert db[b'bytes key \xbd'] == b'bytes value \xbd'
print("DbmTestCase::test_bytes: ok")
