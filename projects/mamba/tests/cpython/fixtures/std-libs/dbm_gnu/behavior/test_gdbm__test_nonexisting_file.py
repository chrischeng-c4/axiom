# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_nonexisting_file"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_nonexisting_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_nonexisting_file
"""Auto-ported test: TestGdbm::test_nonexisting_file (CPython 3.12 oracle)."""


import os
import unittest
from test import support
from test.support import cpython_only, import_helper
from test.support.os_helper import TESTFN, TESTFN_NONASCII, FakePath, create_empty_file, temp_dir, unlink


gdbm = import_helper.import_module('dbm.gnu')

filename = TESTFN


# --- test body ---
self_g = None
nonexisting_file = 'nonexisting-file'
try:
    gdbm.open(nonexisting_file)
    raise AssertionError('expected gdbm.error')
except gdbm.error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert nonexisting_file in str(cm.exception)

assert cm.exception.filename == nonexisting_file
print("TestGdbm::test_nonexisting_file: ok")
