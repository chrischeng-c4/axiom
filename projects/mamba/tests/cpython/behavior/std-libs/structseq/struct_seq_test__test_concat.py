# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_concat"
# subject = "cpython.test_structseq.StructSeqTest.test_concat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_concat
"""Auto-ported test: StructSeqTest::test_concat (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t1 = time.gmtime()
t2 = t1 + tuple(t1)
for i in range(len(t1)):

    assert t2[i] == t2[i + len(t1)]
print("StructSeqTest::test_concat: ok")
