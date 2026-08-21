# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_star_expression"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_star_expression"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_star_expression
"""Auto-ported test: DictComprehensionTest::test_star_expression (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {0: 0, 1: 1, 2: 4, 3: 9}

assert {i: i * i for i in [*range(4)]} == expected

assert {i: i * i for i in (*range(4),)} == expected
print("DictComprehensionTest::test_star_expression: ok")
