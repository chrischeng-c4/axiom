# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "metadata_api"
# dimension = "behavior"
# case = "a_p_i_tests__test_name_normalization_uc96224e"
# subject = "cpython.test_metadata_api.APITests.test_name_normalization"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_metadata_api.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_metadata_api
_suite = unittest.defaultTestLoader.loadTestsFromName("APITests.test_name_normalization", test_metadata_api)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython APITests.test_name_normalization did not pass"
print("APITests::test_name_normalization: ok")
