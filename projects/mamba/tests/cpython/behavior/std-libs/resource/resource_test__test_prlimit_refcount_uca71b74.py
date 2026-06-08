# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_test__test_prlimit_refcount_uca71b74"
# subject = "cpython.test_resource.ResourceTest.test_prlimit_refcount"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_resource
_suite = unittest.defaultTestLoader.loadTestsFromName("ResourceTest.test_prlimit_refcount", test_resource)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ResourceTest.test_prlimit_refcount did not pass"
print("ResourceTest::test_prlimit_refcount: ok")
