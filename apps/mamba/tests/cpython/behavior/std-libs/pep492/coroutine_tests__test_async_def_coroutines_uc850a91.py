# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pep492"
# dimension = "behavior"
# case = "coroutine_tests__test_async_def_coroutines_uc850a91"
# subject = "cpython.test_pep492.CoroutineTests.test_async_def_coroutines"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_pep492.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_pep492
_suite = unittest.defaultTestLoader.loadTestsFromName("CoroutineTests.test_async_def_coroutines", test_pep492)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CoroutineTests.test_async_def_coroutines did not pass"
print("CoroutineTests::test_async_def_coroutines: ok")
