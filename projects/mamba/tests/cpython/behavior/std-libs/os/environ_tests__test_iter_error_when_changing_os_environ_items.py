# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "environ_tests__test_iter_error_when_changing_os_environ_items"
# subject = "cpython.test_os.EnvironTests.test_iter_error_when_changing_os_environ_items"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("EnvironTests.test_iter_error_when_changing_os_environ_items", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EnvironTests.test_iter_error_when_changing_os_environ_items did not pass"
print("EnvironTests::test_iter_error_when_changing_os_environ_items: ok")
