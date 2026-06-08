# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_tuple"
# subject = "cpython.test_structseq.StructSeqTest.test_tuple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_tuple
"""Auto-ported test: StructSeqTest::test_tuple (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t = time.gmtime()

assert isinstance(t, tuple)
astuple = tuple(t)

assert len(t) == len(astuple)

assert t == astuple
for i in range(-len(t), len(t)):

    assert t[i:] == astuple[i:]
    for j in range(-len(t), len(t)):

        assert t[i:j] == astuple[i:j]
for j in range(-len(t), len(t)):

    assert t[:j] == astuple[:j]

try:
    t.__getitem__(-len(t) - 1)
    raise AssertionError('expected IndexError')
except IndexError:
    pass

try:
    t.__getitem__(len(t))
    raise AssertionError('expected IndexError')
except IndexError:
    pass
for i in range(-len(t), len(t) - 1):

    assert t[i] == astuple[i]
print("StructSeqTest::test_tuple: ok")
