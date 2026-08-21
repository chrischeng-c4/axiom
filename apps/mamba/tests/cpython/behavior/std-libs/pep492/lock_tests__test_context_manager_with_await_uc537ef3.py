# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pep492"
# dimension = "behavior"
# case = "lock_tests__test_context_manager_with_await_uc537ef3"
# subject = "cpython.test_pep492.LockTests.test_context_manager_with_await"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_pep492.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_pep492
_suite = unittest.defaultTestLoader.loadTestsFromName("LockTests.test_context_manager_with_await", test_pep492)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LockTests.test_context_manager_with_await did not pass"
print("LockTests::test_context_manager_with_await: ok")
