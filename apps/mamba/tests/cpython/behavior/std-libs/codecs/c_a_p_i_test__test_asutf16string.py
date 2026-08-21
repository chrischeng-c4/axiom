# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "c_a_p_i_test__test_asutf16string"
# subject = "cpython.test_codecs.CAPITest.test_asutf16string"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_asutf16string", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_asutf16string did not pass"
print("CAPITest::test_asutf16string: ok")
