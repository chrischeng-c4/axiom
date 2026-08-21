# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runners"
# dimension = "behavior"
# case = "run_tests__test_asyncio_run_closes_gens_after_hanging_tasks_errors_ucf3cf14"
# subject = "cpython.test_runners.RunTests.test_asyncio_run_closes_gens_after_hanging_tasks_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_runners.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_runners
_suite = unittest.defaultTestLoader.loadTestsFromName("RunTests.test_asyncio_run_closes_gens_after_hanging_tasks_errors", test_runners)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RunTests.test_asyncio_run_closes_gens_after_hanging_tasks_errors did not pass"
print("RunTests::test_asyncio_run_closes_gens_after_hanging_tasks_errors: ok")
