# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pep492"
# dimension = "behavior"
# case = "coroutine_tests__test_debug_mode_manages_coroutine_origin_tracking_uc3e188b"
# subject = "cpython.test_pep492.CoroutineTests.test_debug_mode_manages_coroutine_origin_tracking"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_pep492.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_pep492
_suite = unittest.defaultTestLoader.loadTestsFromName("CoroutineTests.test_debug_mode_manages_coroutine_origin_tracking", test_pep492)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CoroutineTests.test_debug_mode_manages_coroutine_origin_tracking did not pass"
print("CoroutineTests::test_debug_mode_manages_coroutine_origin_tracking: ok")
