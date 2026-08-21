# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getargs"
# dimension = "behavior"
# case = "parse_tuple_and_keywords__test__test_parse_tuple_and_keywords"
# subject = "cpython.test_getargs.ParseTupleAndKeywords_Test.test_parse_tuple_and_keywords"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_getargs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_getargs
_suite = unittest.defaultTestLoader.loadTestsFromName("ParseTupleAndKeywords_Test.test_parse_tuple_and_keywords", test_getargs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ParseTupleAndKeywords_Test.test_parse_tuple_and_keywords did not pass"
print("ParseTupleAndKeywords_Test::test_parse_tuple_and_keywords: ok")
