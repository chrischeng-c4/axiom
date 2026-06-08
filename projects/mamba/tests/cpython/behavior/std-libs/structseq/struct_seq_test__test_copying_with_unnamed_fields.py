# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_copying_with_unnamed_fields"
# subject = "cpython.test_structseq.StructSeqTest.test_copying_with_unnamed_fields"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_copying_with_unnamed_fields
"""Auto-ported test: StructSeqTest::test_copying_with_unnamed_fields (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
assert os.stat_result.n_unnamed_fields > 0
n_sequence_fields = os.stat_result.n_sequence_fields
r = os.stat_result([[i] for i in range(n_sequence_fields)], {'st_atime': [1.0], 'st_atime_ns': [2.0]})
r2 = copy.copy(r)

assert r2.__class__ == r.__class__

assert r2 == r

assert r2.st_mode == r.st_mode

assert r2.st_atime == r.st_atime

assert r2.st_atime_ns == r.st_atime_ns

assert r2[0] is r[0]

assert r2.st_mode is r.st_mode

assert r2.st_atime is r.st_atime

assert r2.st_atime_ns is r.st_atime_ns
r3 = copy.deepcopy(r)

assert r3.__class__ == r.__class__

assert r3 == r

assert r3.st_mode == r.st_mode

assert r3.st_atime == r.st_atime

assert r3.st_atime_ns == r.st_atime_ns

assert r3[0] is not r[0]

assert r3.st_mode is not r.st_mode

assert r3.st_atime is not r.st_atime

assert r3.st_atime_ns is not r.st_atime_ns
print("StructSeqTest::test_copying_with_unnamed_fields: ok")
