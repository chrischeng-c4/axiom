# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_tests__test_various_ops_large_stack_uc6797d7"
# subject = "cpython.test_threading.ThreadTests.test_various_ops_large_stack"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_threading
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadTests.test_various_ops_large_stack", test_threading)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadTests.test_various_ops_large_stack did not pass"
print("ThreadTests::test_various_ops_large_stack: ok")
