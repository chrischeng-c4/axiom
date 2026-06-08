# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_with_unpacking"
# subject = "cpython.test_augassign.AugAssignTest.test_with_unpacking"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::test_with_unpacking
"""Auto-ported test: AugAssignTest::test_with_unpacking (CPython 3.12 oracle)."""


import unittest


# --- test body ---

try:
    compile('x, b += 3', '<test>', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("AugAssignTest::test_with_unpacking: ok")
