# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_header_value_parser"
# dimension = "behavior"
# case = "test_parser__test_get_group_list_obs_group_list"
# subject = "cpython.test__header_value_parser.TestParser.test_get_group_list_obs_group_list"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test__header_value_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test__header_value_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestParser.test_get_group_list_obs_group_list", test__header_value_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestParser.test_get_group_list_obs_group_list did not pass"
print("TestParser::test_get_group_list_obs_group_list: ok")
