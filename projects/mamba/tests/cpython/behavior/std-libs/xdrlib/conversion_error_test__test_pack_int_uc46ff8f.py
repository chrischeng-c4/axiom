# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "behavior"
# case = "conversion_error_test__test_pack_int_uc46ff8f"
# subject = "cpython.test_xdrlib.ConversionErrorTest.test_pack_int"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xdrlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xdrlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ConversionErrorTest.test_pack_int", test_xdrlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConversionErrorTest.test_pack_int did not pass"
print("ConversionErrorTest::test_pack_int: ok")
