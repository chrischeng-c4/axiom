# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "behavior"
# case = "helper_functions_tests__test_addsitedir_hidden_file_attribute_uc681f86"
# subject = "cpython.test_site.HelperFunctionsTests.test_addsitedir_hidden_file_attribute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_site.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_site
_suite = unittest.defaultTestLoader.loadTestsFromName("HelperFunctionsTests.test_addsitedir_hidden_file_attribute", test_site)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HelperFunctionsTests.test_addsitedir_hidden_file_attribute did not pass"
print("HelperFunctionsTests::test_addsitedir_hidden_file_attribute: ok")
