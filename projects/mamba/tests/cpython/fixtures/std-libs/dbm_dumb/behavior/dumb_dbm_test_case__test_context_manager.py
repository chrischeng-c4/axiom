# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_dbm_test_case__test_context_manager"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_context_manager"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_dumb.py::DumbDBMTestCase::test_context_manager
"""Auto-ported test: DumbDBMTestCase::test_context_manager (CPython 3.12 oracle)."""


import contextlib
import io
import operator
import os
import stat
import unittest
import dbm.dumb as dumbdbm
from test import support
from test.support import os_helper
from functools import partial


'Test script for the dumbdbm module\n   Original by Roger E. Masse\n'

_fname = os_helper.TESTFN

def _delete_files():
    for ext in ['.dir', '.dat', '.bak']:
        try:
            os.unlink(_fname + ext)
        except OSError:
            pass


# --- test body ---
_delete_files()
with dumbdbm.open(_fname, 'c') as db:
    db['dumbdbm context manager'] = 'context manager'
with dumbdbm.open(_fname, 'r') as db:

    assert list(db.keys()) == [b'dumbdbm context manager']
try:
    db.keys()
    raise AssertionError('expected dumbdbm.error')
except dumbdbm.error:
    pass
print("DumbDBMTestCase::test_context_manager: ok")
