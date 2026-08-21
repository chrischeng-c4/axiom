# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "structseq"
# dimension = "behavior"
# case = "struct_seq_test__test_constructor"
# subject = "cpython.test_structseq.StructSeqTest.test_constructor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_structseq.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_structseq.py::StructSeqTest::test_constructor
"""Auto-ported test: StructSeqTest::test_constructor (CPython 3.12 oracle)."""


import copy
import os
import pickle
import textwrap
import time
import unittest
from test.support import script_helper


# --- test body ---
t = time.struct_time

try:
    t()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    t(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    t('123')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    t('123', dict={})
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    t('123456789', dict=None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
s = '123456789'

assert ''.join(t(s)) == s
print("StructSeqTest::test_constructor: ok")
