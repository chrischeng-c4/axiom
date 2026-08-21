# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_dbm_test_case__test_invalid_flag"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_invalid_flag"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_dumb.py::DumbDBMTestCase::test_invalid_flag
"""Auto-ported test: DumbDBMTestCase::test_invalid_flag (CPython 3.12 oracle)."""


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
for flag in ('x', 'rf', None):
    try:
        dumbdbm.open(_fname, flag)
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search("Flag must be one of 'r', 'w', 'c', or 'n'", str(_aR_e))
print("DumbDBMTestCase::test_invalid_flag: ok")
