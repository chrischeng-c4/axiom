# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_except_star_split_semantics__test_match_single_type_partial_match"
# subject = "cpython.test_except_star.TestExceptStarSplitSemantics.test_match_single_type_partial_match"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_except_star
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExceptStarSplitSemantics.test_match_single_type_partial_match", test_except_star)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExceptStarSplitSemantics.test_match_single_type_partial_match did not pass"
print("TestExceptStarSplitSemantics::test_match_single_type_partial_match: ok")
