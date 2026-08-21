# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_scope_isolation_from_global"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_scope_isolation_from_global"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_scope_isolation_from_global
"""Auto-ported test: DictComprehensionTest::test_scope_isolation_from_global (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {0: None, 1: None, 2: None, 3: None, 4: None, 5: None, 6: None, 7: None, 8: None, 9: None}
actual = {g: None for g in range(10)}

assert actual == expected

assert g == 'Global variable'
expected = {9: 1, 18: 2, 19: 2, 27: 3, 28: 3, 29: 3, 36: 4, 37: 4, 38: 4, 39: 4, 45: 5, 46: 5, 47: 5, 48: 5, 49: 5, 54: 6, 55: 6, 56: 6, 57: 6, 58: 6, 59: 6, 63: 7, 64: 7, 65: 7, 66: 7, 67: 7, 68: 7, 69: 7, 72: 8, 73: 8, 74: 8, 75: 8, 76: 8, 77: 8, 78: 8, 79: 8, 81: 9, 82: 9, 83: 9, 84: 9, 85: 9, 86: 9, 87: 9, 88: 9, 89: 9}
actual = {g: v for v in range(10) for g in range(v * 9, v * 10)}

assert g == 'Global variable'

assert actual == expected
print("DictComprehensionTest::test_scope_isolation_from_global: ok")
