# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_write_readonly_file"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_write_readonly_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_write_readonly_file
"""Auto-ported test: TestGdbm::test_write_readonly_file (CPython 3.12 oracle)."""


import os
import unittest
from test import support
from test.support import cpython_only, import_helper
from test.support.os_helper import TESTFN, TESTFN_NONASCII, FakePath, create_empty_file, temp_dir, unlink


gdbm = import_helper.import_module('dbm.gnu')

filename = TESTFN


# --- test body ---
self_g = None
with gdbm.open(filename, 'c') as db:
    db[b'bytes key'] = b'bytes value'
with gdbm.open(filename, 'r') as db:
    try:
        del db[b'not exist key']
        raise AssertionError('expected gdbm.error')
    except gdbm.error:
        pass
    try:
        del db[b'bytes key']
        raise AssertionError('expected gdbm.error')
    except gdbm.error:
        pass
    try:
        db[b'not exist key'] = b'not exist value'
        raise AssertionError('expected gdbm.error')
    except gdbm.error:
        pass
print("TestGdbm::test_write_readonly_file: ok")
