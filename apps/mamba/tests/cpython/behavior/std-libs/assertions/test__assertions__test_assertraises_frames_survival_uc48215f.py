# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "assertions"
# dimension = "behavior"
# case = "test__assertions__test_assertraises_frames_survival_uc48215f"
# subject = "cpython.test_assertions.Test_Assertions.test_assertRaises_frames_survival"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_assertions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_assertions
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_Assertions.test_assertRaises_frames_survival", test_assertions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_Assertions.test_assertRaises_frames_survival did not pass"
print("Test_Assertions::test_assertRaises_frames_survival: ok")
