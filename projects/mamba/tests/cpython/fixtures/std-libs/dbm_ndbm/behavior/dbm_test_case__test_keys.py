# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_ndbm"
# dimension = "behavior"
# case = "dbm_test_case__test_keys"
# subject = "cpython.test_dbm_ndbm.DbmTestCase.test_keys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_ndbm.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_ndbm.py::DbmTestCase::test_keys
"""Auto-ported test: DbmTestCase::test_keys (CPython 3.12 oracle)."""


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
self_d = dbm.ndbm.open(self_filename, 'c')

assert self_d.keys() == []
self_d['a'] = 'b'
self_d[b'bytes'] = b'data'
self_d['12345678910'] = '019237410982340912840198242'
self_d.keys()

assert 'a' in self_d

assert b'a' in self_d

assert self_d[b'bytes'] == b'data'

assert self_d.get(b'a') == b'b'

assert self_d.get(b'xxx') is None

assert self_d.get(b'xxx', b'foo') == b'foo'
try:
    self_d['xxx']
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert self_d.setdefault(b'xxx', b'foo') == b'foo'

assert self_d[b'xxx'] == b'foo'
self_d.close()
print("DbmTestCase::test_keys: ok")
