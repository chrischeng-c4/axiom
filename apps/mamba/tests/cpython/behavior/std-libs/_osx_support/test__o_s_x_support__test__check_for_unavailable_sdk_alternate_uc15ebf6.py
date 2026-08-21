# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_osx_support"
# dimension = "behavior"
# case = "test__o_s_x_support__test__check_for_unavailable_sdk_alternate_uc15ebf6"
# subject = "cpython.test__osx_support.Test_OSXSupport.test__check_for_unavailable_sdk_alternate"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__osx_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__osx_support
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_OSXSupport.test__check_for_unavailable_sdk_alternate", test__osx_support)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_OSXSupport.test__check_for_unavailable_sdk_alternate did not pass"
print("Test_OSXSupport::test__check_for_unavailable_sdk_alternate: ok")
