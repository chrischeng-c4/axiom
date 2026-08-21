# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_match_args_with_unnamed_fields"
# subject = "cpython.test_structseq.StructSeqTest.test_match_args_with_unnamed_fields"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_match_args_with_unnamed_fields
"""Auto-ported test: StructSeqTest::test_match_args_with_unnamed_fields (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
expected_args = ('st_mode', 'st_ino', 'st_dev', 'st_nlink', 'st_uid', 'st_gid', 'st_size')

assert os.stat_result.n_unnamed_fields == 3

assert os.stat_result.__match_args__ == expected_args
print("StructSeqTest::test_match_args_with_unnamed_fields: ok")
