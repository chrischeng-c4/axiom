# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_key_methods"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_key_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_key_methods
"""Auto-ported test: TestGdbm::test_key_methods (CPython 3.12 oracle)."""


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

assert self_g.keys() == []
self_g['a'] = 'b'
self_g['12345678910'] = '019237410982340912840198242'
self_g[b'bytes'] = b'data'
key_set = set(self_g.keys())

assert key_set == set([b'a', b'bytes', b'12345678910'])

assert 'a' in self_g

assert b'a' in self_g

assert self_g[b'bytes'] == b'data'
key = self_g.firstkey()
while key:

    assert key in key_set
    key_set.remove(key)
    key = self_g.nextkey(key)

assert self_g.get(b'a') == b'b'

assert self_g.get(b'xxx') is None

assert self_g.get(b'xxx', b'foo') == b'foo'
try:
    self_g['xxx']
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert self_g.setdefault(b'xxx', b'foo') == b'foo'

assert self_g[b'xxx'] == b'foo'
print("TestGdbm::test_key_methods: ok")
