# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "futures2"
# dimension = "behavior"
# case = "future_repr_tests__test_recursive_repr_for_pending_tasks_uc1abfb3"
# subject = "cpython.test_futures2.FutureReprTests.test_recursive_repr_for_pending_tasks"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_futures2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_futures2
_suite = unittest.defaultTestLoader.loadTestsFromName("FutureReprTests.test_recursive_repr_for_pending_tasks", test_futures2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FutureReprTests.test_recursive_repr_for_pending_tasks did not pass"
print("FutureReprTests::test_recursive_repr_for_pending_tasks: ok")
