# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "metadata_api"
# dimension = "behavior"
# case = "a_p_i_tests__test_prefix_not_matched_uc56dcbe"
# subject = "cpython.test_metadata_api.APITests.test_prefix_not_matched"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_metadata_api.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_metadata_api
_suite = unittest.defaultTestLoader.loadTestsFromName("APITests.test_prefix_not_matched", test_metadata_api)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython APITests.test_prefix_not_matched did not pass"
print("APITests::test_prefix_not_matched: ok")
