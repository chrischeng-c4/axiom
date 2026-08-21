# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_pickling"
# subject = "cpython.test_structseq.StructSeqTest.test_pickling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_pickling
"""Auto-ported test: StructSeqTest::test_pickling (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t = time.gmtime()
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    p = pickle.dumps(t, proto)
    t2 = pickle.loads(p)

    assert t2.__class__ == t.__class__

    assert t2 == t

    assert t2.tm_year == t.tm_year

    assert t2.tm_zone == t.tm_zone
print("StructSeqTest::test_pickling: ok")
