# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "deleting_zips_test__test_read_text_does_not_keep_open_uc18bd82"
# subject = "cpython.test_resource.DeletingZipsTest.test_read_text_does_not_keep_open"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_resource
_suite = unittest.defaultTestLoader.loadTestsFromName("DeletingZipsTest.test_read_text_does_not_keep_open", test_resource)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DeletingZipsTest.test_read_text_does_not_keep_open did not pass"
print("DeletingZipsTest::test_read_text_does_not_keep_open: ok")
