# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_cmp"
# subject = "cpython.test_structseq.StructSeqTest.test_cmp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_cmp
"""Auto-ported test: StructSeqTest::test_cmp (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t1 = time.gmtime()
t2 = type(t1)(t1)

assert t1 == t2

assert not t1 < t2

assert t1 <= t2

assert not t1 > t2

assert t1 >= t2

assert not t1 != t2
print("StructSeqTest::test_cmp: ok")
