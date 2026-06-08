# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_dbm_test_case__test_line_endings"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_line_endings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_dumb.py::DumbDBMTestCase::test_line_endings
"""Auto-ported test: DumbDBMTestCase::test_line_endings (CPython 3.12 oracle)."""


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
with contextlib.closing(dumbdbm.open(_fname)) as f:
    f[b'1'] = b'hello'
    f[b'2'] = b'hello2'
with io.open(_fname + '.dir', 'rb') as file:
    data = file.read()
if os.linesep == '\n':
    data = data.replace(b'\n', b'\r\n')
else:
    data = data.replace(b'\r\n', b'\n')
with io.open(_fname + '.dir', 'wb') as file:
    file.write(data)
f = dumbdbm.open(_fname)

assert f[b'1'] == b'hello'

assert f[b'2'] == b'hello2'
print("DumbDBMTestCase::test_line_endings: ok")
