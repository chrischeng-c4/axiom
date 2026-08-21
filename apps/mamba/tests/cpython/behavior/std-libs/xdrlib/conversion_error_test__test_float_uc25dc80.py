# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "behavior"
# case = "conversion_error_test__test_float_uc25dc80"
# subject = "cpython.test_xdrlib.ConversionErrorTest.test_float"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xdrlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xdrlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ConversionErrorTest.test_float", test_xdrlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConversionErrorTest.test_float did not pass"
print("ConversionErrorTest::test_float: ok")
