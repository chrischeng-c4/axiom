# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_match_args"
# subject = "cpython.test_structseq.StructSeqTest.test_match_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_match_args
"""Auto-ported test: StructSeqTest::test_match_args (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
expected_args = ('tm_year', 'tm_mon', 'tm_mday', 'tm_hour', 'tm_min', 'tm_sec', 'tm_wday', 'tm_yday', 'tm_isdst')

assert time.struct_time.__match_args__ == expected_args
print("StructSeqTest::test_match_args: ok")
