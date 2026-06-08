# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_fields"
# subject = "cpython.test_structseq.StructSeqTest.test_fields"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_fields
"""Auto-ported test: StructSeqTest::test_fields (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t = time.gmtime()

assert len(t) == t.n_sequence_fields

assert t.n_unnamed_fields == 0

assert t.n_fields == time._STRUCT_TM_ITEMS
print("StructSeqTest::test_fields: ok")
