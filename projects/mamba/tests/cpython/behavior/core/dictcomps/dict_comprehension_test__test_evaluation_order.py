# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "dictcomps"
# dimension = "behavior"
# case = "dict_comprehension_test__test_evaluation_order"
# subject = "cpython.test_dictcomps.DictComprehensionTest.test_evaluation_order"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictcomps.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictcomps.py::DictComprehensionTest::test_evaluation_order
"""Auto-ported test: DictComprehensionTest::test_evaluation_order (CPython 3.12 oracle)."""


import traceback
import unittest
from test.support import BrokenIter


g = 'Global variable'


# --- test body ---
expected = {'H': 'W', 'e': 'o', 'l': 'l', 'o': 'd'}
expected_calls = [('key', 'H'), ('value', 'W'), ('key', 'e'), ('value', 'o'), ('key', 'l'), ('value', 'r'), ('key', 'l'), ('value', 'l'), ('key', 'o'), ('value', 'd')]
actual_calls = []

def add_call(pos, value):
    actual_calls.append((pos, value))
    return value
actual = {add_call('key', k): add_call('value', v) for k, v in zip('Hello', 'World')}

assert actual == expected

assert actual_calls == expected_calls
print("DictComprehensionTest::test_evaluation_order: ok")
