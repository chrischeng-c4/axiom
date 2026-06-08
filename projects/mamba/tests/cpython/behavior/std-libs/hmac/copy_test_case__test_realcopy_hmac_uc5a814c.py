# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "copy_test_case__test_realcopy_hmac_uc5a814c"
# subject = "cpython.test_hmac.CopyTestCase.test_realcopy_hmac"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hmac
_suite = unittest.defaultTestLoader.loadTestsFromName("CopyTestCase.test_realcopy_hmac", test_hmac)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CopyTestCase.test_realcopy_hmac did not pass"
print("CopyTestCase::test_realcopy_hmac: ok")
