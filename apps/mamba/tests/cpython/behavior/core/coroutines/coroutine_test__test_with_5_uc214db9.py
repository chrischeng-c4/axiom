# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "coroutines"
# dimension = "behavior"
# case = "coroutine_test__test_with_5_uc214db9"
# subject = "cpython.test_coroutines.CoroutineTest.test_with_5"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_coroutines.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_coroutines
_suite = unittest.defaultTestLoader.loadTestsFromName("CoroutineTest.test_with_5", test_coroutines)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CoroutineTest.test_with_5 did not pass"
print("CoroutineTest::test_with_5: ok")
