# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2"
# dimension = "behavior"
# case = "handler_tests__test_https_handler_global_debuglevel_ucd09f86"
# subject = "cpython.test_urllib2.HandlerTests.test_https_handler_global_debuglevel"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2
_suite = unittest.defaultTestLoader.loadTestsFromName("HandlerTests.test_https_handler_global_debuglevel", test_urllib2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HandlerTests.test_https_handler_global_debuglevel did not pass"
print("HandlerTests::test_https_handler_global_debuglevel: ok")
