# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_interesting_edge_cases__test_close_and_throw_return_uc7e2338"
# subject = "cpython.test_yield_from.TestInterestingEdgeCases.test_close_and_throw_return"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_yield_from
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInterestingEdgeCases.test_close_and_throw_return", test_yield_from)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInterestingEdgeCases.test_close_and_throw_return did not pass"
print("TestInterestingEdgeCases::test_close_and_throw_return: ok")
