# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_dbm_test_case__test_check_closed"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_check_closed"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_dumb.py::DumbDBMTestCase::test_check_closed
"""Auto-ported test: DumbDBMTestCase::test_check_closed (CPython 3.12 oracle)."""


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
f = dumbdbm.open(_fname, 'c')
f.close()
for meth in (partial(operator.delitem, f), partial(operator.setitem, f, 'b'), partial(operator.getitem, f), partial(operator.contains, f)):
    try:
        meth('test')
        raise AssertionError('expected dumbdbm.error')
    except dumbdbm.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert str(cm.exception) == 'DBM object has already been closed'
for meth in (operator.methodcaller('keys'), operator.methodcaller('iterkeys'), operator.methodcaller('items'), len):
    try:
        meth(f)
        raise AssertionError('expected dumbdbm.error')
    except dumbdbm.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert str(cm.exception) == 'DBM object has already been closed'
print("DumbDBMTestCase::test_check_closed: ok")
