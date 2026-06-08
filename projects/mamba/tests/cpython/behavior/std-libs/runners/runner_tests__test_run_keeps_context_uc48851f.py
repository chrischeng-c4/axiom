# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runners"
# dimension = "behavior"
# case = "runner_tests__test_run_keeps_context_uc48851f"
# subject = "cpython.test_runners.RunnerTests.test_run_keeps_context"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_runners.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_runners
_suite = unittest.defaultTestLoader.loadTestsFromName("RunnerTests.test_run_keeps_context", test_runners)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RunnerTests.test_run_keeps_context did not pass"
print("RunnerTests::test_run_keeps_context: ok")
