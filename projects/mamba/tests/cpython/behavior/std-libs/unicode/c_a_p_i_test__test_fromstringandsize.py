# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "c_a_p_i_test__test_fromstringandsize"
# subject = "cpython.test_unicode.CAPITest.test_fromstringandsize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_unicode
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_fromstringandsize", test_unicode)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_fromstringandsize did not pass"
print("CAPITest::test_fromstringandsize: ok")
