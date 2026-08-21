# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tasks"
# dimension = "behavior"
# case = "future_gather_tests__test_constructor_heterogenous_futures"
# subject = "cpython.test_tasks.FutureGatherTests.test_constructor_heterogenous_futures"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_tasks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_tasks
_suite = unittest.defaultTestLoader.loadTestsFromName("FutureGatherTests.test_constructor_heterogenous_futures", test_tasks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FutureGatherTests.test_constructor_heterogenous_futures did not pass"
print("FutureGatherTests::test_constructor_heterogenous_futures: ok")
