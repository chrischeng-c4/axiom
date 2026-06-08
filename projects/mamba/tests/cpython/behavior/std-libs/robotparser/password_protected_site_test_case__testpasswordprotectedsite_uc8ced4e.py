# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "robotparser"
# dimension = "behavior"
# case = "password_protected_site_test_case__testpasswordprotectedsite_uc8ced4e"
# subject = "cpython.test_robotparser.PasswordProtectedSiteTestCase.testPasswordProtectedSite"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_robotparser
_suite = unittest.defaultTestLoader.loadTestsFromName("PasswordProtectedSiteTestCase.testPasswordProtectedSite", test_robotparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PasswordProtectedSiteTestCase.testPasswordProtectedSite did not pass"
print("PasswordProtectedSiteTestCase::testPasswordProtectedSite: ok")
