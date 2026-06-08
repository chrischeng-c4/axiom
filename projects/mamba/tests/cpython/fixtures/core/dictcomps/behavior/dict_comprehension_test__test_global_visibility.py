# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_global_visibility"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_global_visibility"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_global_visibility
"""Auto-ported test: DictComprehensionTest::test_global_visibility (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {0: 'Global variable', 1: 'Global variable', 2: 'Global variable', 3: 'Global variable', 4: 'Global variable', 5: 'Global variable', 6: 'Global variable', 7: 'Global variable', 8: 'Global variable', 9: 'Global variable'}
actual = {k: g for k in range(10)}

assert actual == expected
print("DictComprehensionTest::test_global_visibility: ok")
