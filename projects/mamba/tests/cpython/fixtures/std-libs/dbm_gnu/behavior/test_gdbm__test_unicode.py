# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_unicode"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_unicode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_unicode
"""Auto-ported test: TestGdbm::test_unicode (CPython 3.12 oracle)."""


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
    db['Unicode key 🐍'] = 'Unicode value 🐍'
with gdbm.open(filename, 'r') as db:

    assert list(db.keys()) == ['Unicode key 🐍'.encode()]

    assert 'Unicode key 🐍'.encode() in db

    assert 'Unicode key 🐍' in db

    assert db['Unicode key 🐍'.encode()] == 'Unicode value 🐍'.encode()

    assert db['Unicode key 🐍'] == 'Unicode value 🐍'.encode()
print("TestGdbm::test_unicode: ok")
