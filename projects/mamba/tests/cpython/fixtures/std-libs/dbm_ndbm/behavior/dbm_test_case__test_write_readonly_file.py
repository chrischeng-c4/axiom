# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_ndbm"
# dimension = "behavior"
# case = "dbm_test_case__test_write_readonly_file"
# subject = "cpython.test_dbm_ndbm.DbmTestCase.test_write_readonly_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_ndbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_ndbm.py::DbmTestCase::test_write_readonly_file
"""Auto-ported test: DbmTestCase::test_write_readonly_file (CPython 3.12 oracle)."""


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
    db[b'bytes key'] = b'bytes value'
with dbm.ndbm.open(self_filename, 'r') as db:
    try:
        del db[b'not exist key']
        raise AssertionError('expected error')
    except error:
        pass
    try:
        del db[b'bytes key']
        raise AssertionError('expected error')
    except error:
        pass
    try:
        db[b'not exist key'] = b'not exist value'
        raise AssertionError('expected error')
    except error:
        pass
print("DbmTestCase::test_write_readonly_file: ok")
