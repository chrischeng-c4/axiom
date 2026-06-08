# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_eviltuple"
# subject = "cpython.test_structseq.StructSeqTest.test_eviltuple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_eviltuple
"""Auto-ported test: StructSeqTest::test_eviltuple (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
class Exc(Exception):
    pass

class C:

    def __getitem__(self, i):
        raise Exc

    def __len__(self):
        return 9

try:
    time.struct_time(C())
    raise AssertionError('expected Exc')
except Exc:
    pass
print("StructSeqTest::test_eviltuple: ok")
