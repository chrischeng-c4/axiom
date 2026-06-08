# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_pickling_with_unnamed_fields"
# subject = "cpython.test_structseq.StructSeqTest.test_pickling_with_unnamed_fields"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_pickling_with_unnamed_fields
"""Auto-ported test: StructSeqTest::test_pickling_with_unnamed_fields (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
assert os.stat_result.n_unnamed_fields > 0
r = os.stat_result(range(os.stat_result.n_sequence_fields), {'st_atime': 1.0, 'st_atime_ns': 2.0})
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    p = pickle.dumps(r, proto)
    r2 = pickle.loads(p)

    assert r2.__class__ == r.__class__

    assert r2 == r

    assert r2.st_mode == r.st_mode

    assert r2.st_atime == r.st_atime

    assert r2.st_atime_ns == r.st_atime_ns
print("StructSeqTest::test_pickling_with_unnamed_fields: ok")
