# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_assignment_idiom_in_comprehensions"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_assignment_idiom_in_comprehensions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_assignment_idiom_in_comprehensions
"""Auto-ported test: DictComprehensionTest::test_assignment_idiom_in_comprehensions (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {1: 1, 2: 4, 3: 9, 4: 16}
actual = {j: j * j for i in range(4) for j in [i + 1]}

assert actual == expected
expected = {3: 2, 5: 6, 7: 12, 9: 20}
actual = {j + k: j * k for i in range(4) for j in [i + 1] for k in [j + 1]}

assert actual == expected
expected = {3: 2, 5: 6, 7: 12, 9: 20}
actual = {j + k: j * k for i in range(4) for j, k in [(i + 1, i + 2)]}

assert actual == expected
print("DictComprehensionTest::test_assignment_idiom_in_comprehensions: ok")
