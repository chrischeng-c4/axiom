# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_dbm_test_case__test_missing_index"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_missing_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_dumb.py::DumbDBMTestCase::test_missing_index
"""Auto-ported test: DumbDBMTestCase::test_missing_index (CPython 3.12 oracle)."""


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
with dumbdbm.open(_fname, 'n') as f:
    pass
os.unlink(_fname + '.dir')
for value in ('r', 'w'):
    try:
        dumbdbm.open(_fname, value)
        raise AssertionError('expected FileNotFoundError')
    except FileNotFoundError:
        pass

    assert not os.path.exists(_fname + '.dir')

    assert not os.path.exists(_fname + '.bak')
print("DumbDBMTestCase::test_missing_index: ok")
