# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_free_after_iterating"
# subject = "cpython.test_dict.DictTest.test_free_after_iterating"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
"""Auto-ported test: DictTest::test_free_after_iterating (CPython 3.12 oracle)."""

import unittest
from test import support


case = unittest.TestCase()
support.check_free_after_iterating(case, iter, dict)
support.check_free_after_iterating(case, lambda d: iter(d.keys()), dict)
support.check_free_after_iterating(case, lambda d: iter(d.values()), dict)
support.check_free_after_iterating(case, lambda d: iter(d.items()), dict)

print("DictTest::test_free_after_iterating: ok")
