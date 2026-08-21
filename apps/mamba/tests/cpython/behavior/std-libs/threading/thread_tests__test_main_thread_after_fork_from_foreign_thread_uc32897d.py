# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_tests__test_main_thread_after_fork_from_foreign_thread_uc32897d"
# subject = "cpython.test_threading.ThreadTests.test_main_thread_after_fork_from_foreign_thread"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threading
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadTests.test_main_thread_after_fork_from_foreign_thread", test_threading)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadTests.test_main_thread_after_fork_from_foreign_thread did not pass"
print("ThreadTests::test_main_thread_after_fork_from_foreign_thread: ok")
