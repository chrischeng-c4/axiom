# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "simplesubclasses"
# dimension = "behavior"
# case = "test__test_int_callback_uc87b243"
# subject = "cpython.test_simplesubclasses.Test.test_int_callback"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_simplesubclasses.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_simplesubclasses
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_int_callback", test_simplesubclasses)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_int_callback did not pass"
print("Test::test_int_callback: ok")
