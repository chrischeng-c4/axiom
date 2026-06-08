# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "condition_as_r_lock_tests__test_recursion_count_ucb307e2"
# subject = "cpython.test_threading.ConditionAsRLockTests.test_recursion_count"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threading
_suite = unittest.defaultTestLoader.loadTestsFromName("ConditionAsRLockTests.test_recursion_count", test_threading)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConditionAsRLockTests.test_recursion_count did not pass"
print("ConditionAsRLockTests::test_recursion_count: ok")
