# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "generator_throw_test__test_exception_context_with_yield_from_with_context_cycle_uc1d4496"
# subject = "cpython.test_generators.GeneratorThrowTest.test_exception_context_with_yield_from_with_context_cycle"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_generators.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_generators
_suite = unittest.defaultTestLoader.loadTestsFromName("GeneratorThrowTest.test_exception_context_with_yield_from_with_context_cycle", test_generators)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GeneratorThrowTest.test_exception_context_with_yield_from_with_context_cycle did not pass"
print("GeneratorThrowTest::test_exception_context_with_yield_from_with_context_cycle: ok")
