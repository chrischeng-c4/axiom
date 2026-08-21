# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_context_manager"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_context_manager"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_context_manager
"""Auto-ported test: TestGdbm::test_context_manager (CPython 3.12 oracle)."""


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
    db['gdbm context manager'] = 'context manager'
with gdbm.open(filename, 'r') as db:

    assert list(db.keys()) == [b'gdbm context manager']
try:
    db.keys()
    raise AssertionError('expected gdbm.error')
except gdbm.error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == 'GDBM object has already been closed'
print("TestGdbm::test_context_manager: ok")
