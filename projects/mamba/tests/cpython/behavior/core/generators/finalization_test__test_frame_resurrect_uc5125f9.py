# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "finalization_test__test_frame_resurrect_uc5125f9"
# subject = "cpython.test_generators.FinalizationTest.test_frame_resurrect"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_generators.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_generators
_suite = unittest.defaultTestLoader.loadTestsFromName("FinalizationTest.test_frame_resurrect", test_generators)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FinalizationTest.test_frame_resurrect did not pass"
print("FinalizationTest::test_frame_resurrect: ok")
