# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pep3118"
# dimension = "behavior"
# case = "test__test_endian_types_uc8341f5"
# subject = "cpython.test_pep3118.Test.test_endian_types"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pep3118.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_pep3118
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_endian_types", test_pep3118)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_endian_types did not pass"
print("Test::test_endian_types: ok")
