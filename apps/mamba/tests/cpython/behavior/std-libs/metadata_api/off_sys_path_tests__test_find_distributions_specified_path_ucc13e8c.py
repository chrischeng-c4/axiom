# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "metadata_api"
# dimension = "behavior"
# case = "off_sys_path_tests__test_find_distributions_specified_path_ucc13e8c"
# subject = "cpython.test_metadata_api.OffSysPathTests.test_find_distributions_specified_path"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_metadata_api.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_metadata_api
_suite = unittest.defaultTestLoader.loadTestsFromName("OffSysPathTests.test_find_distributions_specified_path", test_metadata_api)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OffSysPathTests.test_find_distributions_specified_path did not pass"
print("OffSysPathTests::test_find_distributions_specified_path: ok")
