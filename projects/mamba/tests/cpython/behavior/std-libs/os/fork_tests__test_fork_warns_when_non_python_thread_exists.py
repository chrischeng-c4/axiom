# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "fork_tests__test_fork_warns_when_non_python_thread_exists"
# subject = "cpython.test_os.ForkTests.test_fork_warns_when_non_python_thread_exists"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("ForkTests.test_fork_warns_when_non_python_thread_exists", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ForkTests.test_fork_warns_when_non_python_thread_exists did not pass"
print("ForkTests::test_fork_warns_when_non_python_thread_exists: ok")
