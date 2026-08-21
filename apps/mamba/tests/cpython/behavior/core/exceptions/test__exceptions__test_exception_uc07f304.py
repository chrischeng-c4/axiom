# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "test__exceptions__test_exception_uc07f304"
# subject = "cpython.test_exceptions.Test_Exceptions.test_exception"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_Exceptions.test_exception", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_Exceptions.test_exception did not pass"
print("Test_Exceptions::test_exception: ok")
