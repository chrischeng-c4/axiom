# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm_dumb"
# dimension = "behavior"
# case = "dumb_dbm_test_case__test_random"
# subject = "cpython.test_dbm_dumb.DumbDBMTestCase.test_random"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dbm_dumb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dbm_dumb.py::DumbDBMTestCase::test_random
"""Auto-ported test: DumbDBMTestCase::test_random (CPython 3.12 oracle)."""


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
import random
d = {}
for dummy in range(5):
    with contextlib.closing(dumbdbm.open(_fname)) as f:
        for dummy in range(100):
            k = random.choice('abcdefghijklm')
            if random.random() < 0.2:
                if k in d:
                    del d[k]
                    del f[k]
            else:
                v = random.choice((b'a', b'b', b'c')) * random.randrange(10000)
                d[k] = v
                f[k] = v

                assert f[k] == v
    with contextlib.closing(dumbdbm.open(_fname)) as f:
        expected = sorted(((k.encode('latin-1'), v) for k, v in d.items()))
        got = sorted(f.items())

        assert expected == got
print("DumbDBMTestCase::test_random: ok")
