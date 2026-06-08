# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_ndbm"
# dimension = "behavior"
# case = "dbm_test_case__test_nonexisting_file"
# subject = "cpython.test_dbm_ndbm.DbmTestCase.test_nonexisting_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_ndbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_ndbm.py::DbmTestCase::test_nonexisting_file
"""Auto-ported test: DbmTestCase::test_nonexisting_file (CPython 3.12 oracle)."""


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
nonexisting_file = 'nonexisting-file'
try:
    dbm.ndbm.open(nonexisting_file)
    raise AssertionError('expected dbm.ndbm.error')
except dbm.ndbm.error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert nonexisting_file in str(cm.exception)

assert cm.exception.filename == nonexisting_file
print("DbmTestCase::test_nonexisting_file: ok")
