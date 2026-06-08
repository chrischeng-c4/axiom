# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_extended_getslice"
# subject = "cpython.test_structseq.StructSeqTest.test_extended_getslice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_extended_getslice
"""Auto-ported test: StructSeqTest::test_extended_getslice (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t = time.gmtime()
L = list(t)
indices = (0, None, 1, 3, 19, 300, -1, -2, -31, -300)
for start in indices:
    for stop in indices:
        for step in indices[1:]:

            assert list(t[start:stop:step]) == L[start:stop:step]
print("StructSeqTest::test_extended_getslice: ok")
