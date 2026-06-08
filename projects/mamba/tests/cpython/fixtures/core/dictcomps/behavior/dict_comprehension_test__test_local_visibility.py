# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_local_visibility"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_local_visibility"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_local_visibility
"""Auto-ported test: DictComprehensionTest::test_local_visibility (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
v = 'Local variable'
expected = {0: 'Local variable', 1: 'Local variable', 2: 'Local variable', 3: 'Local variable', 4: 'Local variable', 5: 'Local variable', 6: 'Local variable', 7: 'Local variable', 8: 'Local variable', 9: 'Local variable'}
actual = {k: v for k in range(10)}

assert actual == expected

assert v == 'Local variable'
print("DictComprehensionTest::test_local_visibility: ok")
