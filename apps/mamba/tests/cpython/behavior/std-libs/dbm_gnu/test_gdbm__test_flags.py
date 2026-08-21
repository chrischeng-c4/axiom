# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_gnu"
# dimension = "behavior"
# case = "test_gdbm__test_flags"
# subject = "cpython.test_dbm_gnu.TestGdbm.test_flags"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_gnu.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_gnu.py::TestGdbm::test_flags
"""Auto-ported test: TestGdbm::test_flags (CPython 3.12 oracle)."""


import os
import unittest
from test import support
from test.support import cpython_only, import_helper
from test.support.os_helper import TESTFN, TESTFN_NONASCII, FakePath, create_empty_file, temp_dir, unlink


gdbm = import_helper.import_module('dbm.gnu')

filename = TESTFN


# --- test body ---
self_g = None
all = set(gdbm.open_flags)
modes = all - set('fsu')
for mode in sorted(modes):
    self_g = gdbm.open(filename, mode)
    self_g.close()
flags = all - set('crwn')
for mode in modes:
    for flag in flags:
        self_g = gdbm.open(filename, mode + flag)
        self_g.close()
print("TestGdbm::test_flags: ok")
