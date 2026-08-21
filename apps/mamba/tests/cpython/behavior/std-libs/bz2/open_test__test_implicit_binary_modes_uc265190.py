# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_test__test_implicit_binary_modes_uc265190"
# subject = "cpython.test_bz2.OpenTest.test_implicit_binary_modes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bz2
_suite = unittest.defaultTestLoader.loadTestsFromName("OpenTest.test_implicit_binary_modes", test_bz2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OpenTest.test_implicit_binary_modes did not pass"
print("OpenTest::test_implicit_binary_modes: ok")
