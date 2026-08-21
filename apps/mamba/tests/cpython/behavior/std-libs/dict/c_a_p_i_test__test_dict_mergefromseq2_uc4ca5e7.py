# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dict"
# dimension = "behavior"
# case = "c_a_p_i_test__test_dict_mergefromseq2_uc4ca5e7"
# subject = "cpython.test_dict.CAPITest.test_dict_mergefromseq2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_dict
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPITest.test_dict_mergefromseq2", test_dict)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPITest.test_dict_mergefromseq2 did not pass"
print("CAPITest::test_dict_mergefromseq2: ok")
