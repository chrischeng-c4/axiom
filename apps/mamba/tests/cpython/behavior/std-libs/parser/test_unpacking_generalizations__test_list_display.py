# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parser"
# dimension = "behavior"
# case = "test_unpacking_generalizations__test_list_display"
# subject = "cpython.test_parser.TestUnpackingGeneralizations.test_list_display"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_parser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_parser
_suite = unittest.defaultTestLoader.loadTestsFromName("TestUnpackingGeneralizations.test_list_display", test_parser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestUnpackingGeneralizations.test_list_display did not pass"
print("TestUnpackingGeneralizations::test_list_display: ok")
