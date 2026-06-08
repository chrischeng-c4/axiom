# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_repr"
# subject = "cpython.test_structseq.StructSeqTest.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_repr
"""Auto-ported test: StructSeqTest::test_repr (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t = time.gmtime()

assert repr(t)
t = time.gmtime(0)

assert repr(t) == 'time.struct_time(tm_year=1970, tm_mon=1, tm_mday=1, tm_hour=0, tm_min=0, tm_sec=0, tm_wday=3, tm_yday=1, tm_isdst=0)'
st = os.stat(__file__)
rep = repr(st)

assert rep.startswith('os.stat_result')

assert 'st_mode=' in rep

assert 'st_ino=' in rep

assert 'st_dev=' in rep
print("StructSeqTest::test_repr: ok")
