# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fcntl"
# dimension = "behavior"
# case = "test_fcntl__test_fcntl_f_pipesize_uc7ce64f"
# subject = "cpython.test_fcntl.TestFcntl.test_fcntl_f_pipesize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fcntl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fcntl
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFcntl.test_fcntl_f_pipesize", test_fcntl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFcntl.test_fcntl_f_pipesize did not pass"
print("TestFcntl::test_fcntl_f_pipesize: ok")
