# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "test_old_py_time__test_object_to_timespec_uc3ce7bf"
# subject = "cpython.test_time.TestOldPyTime.test_object_to_timespec"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_time
_suite = unittest.defaultTestLoader.loadTestsFromName("TestOldPyTime.test_object_to_timespec", test_time)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestOldPyTime.test_object_to_timespec did not pass"
print("TestOldPyTime::test_object_to_timespec: ok")
