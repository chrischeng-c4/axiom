# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_dbm_test_case__test_eval"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_eval"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_dumb.py::DumbDBMTestCase::test_eval
"""Auto-ported test: DumbDBMTestCase::test_eval (CPython 3.12 oracle)."""


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
with open(_fname + '.dir', 'w', encoding='utf-8') as stream:
    stream.write("str(print('Hacked!')), 0\n")
with support.captured_stdout() as stdout:
    try:
        with dumbdbm.open(_fname) as f:
            pass
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    assert stdout.getvalue() == ''
print("DumbDBMTestCase::test_eval: ok")
