# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_tests__test_start_new_thread_failed_ucd08ea9"
# subject = "cpython.test_threading.ThreadTests.test_start_new_thread_failed"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threading
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadTests.test_start_new_thread_failed", test_threading)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadTests.test_start_new_thread_failed did not pass"
print("ThreadTests::test_start_new_thread_failed: ok")
