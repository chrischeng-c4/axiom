# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threads"
# dimension = "behavior"
# case = "to_thread_tests__test_to_thread_once_uc7cb700"
# subject = "cpython.test_threads.ToThreadTests.test_to_thread_once"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_threads.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_threads
_suite = unittest.defaultTestLoader.loadTestsFromName("ToThreadTests.test_to_thread_once", test_threads)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ToThreadTests.test_to_thread_once did not pass"
print("ToThreadTests::test_to_thread_once: ok")
