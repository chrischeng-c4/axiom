# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_keyword_args"
# subject = "cpython.test.test_bool.BoolTest.test_keyword_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_keyword_args
"""Auto-ported test: BoolTest::test_keyword_args (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
try:
    bool(x=10)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))
print("BoolTest::test_keyword_args: ok")
