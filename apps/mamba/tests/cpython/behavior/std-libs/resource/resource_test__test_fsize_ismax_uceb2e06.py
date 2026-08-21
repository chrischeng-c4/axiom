# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_fsize_ismax_uceb2e06"
# subject = "cpython.test_resource.ResourceTest.test_fsize_ismax"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_resource
_suite = unittest.defaultTestLoader.loadTestsFromName("ResourceTest.test_fsize_ismax", test_resource)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ResourceTest.test_fsize_ismax did not pass"
print("ResourceTest::test_fsize_ismax: ok")
