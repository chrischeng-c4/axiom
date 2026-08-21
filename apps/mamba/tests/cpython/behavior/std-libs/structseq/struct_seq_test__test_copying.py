# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_copying"
# subject = "cpython.test_structseq.StructSeqTest.test_copying"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_copying
"""Auto-ported test: StructSeqTest::test_copying (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
n_fields = time.struct_time.n_fields
t = time.struct_time([[i] for i in range(n_fields)])
t2 = copy.copy(t)

assert t2.__class__ == t.__class__

assert t2 == t

assert t2.tm_year == t.tm_year

assert t2.tm_zone == t.tm_zone

assert t2[0] is t[0]

assert t2.tm_year is t.tm_year
t3 = copy.deepcopy(t)

assert t3.__class__ == t.__class__

assert t3 == t

assert t3.tm_year == t.tm_year

assert t3.tm_zone == t.tm_zone

assert t3[0] is not t[0]

assert t3.tm_year is not t.tm_year
print("StructSeqTest::test_copying: ok")
