# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "init"
# dimension = "behavior"
# case = "init_test__test_get_ucf1b5a5"
# subject = "cpython.test_init.InitTest.test_get"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_init.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_init
_suite = unittest.defaultTestLoader.loadTestsFromName("InitTest.test_get", test_init)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InitTest.test_get did not pass"
print("InitTest::test_get: ok")
