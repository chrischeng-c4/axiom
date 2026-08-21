# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uu"
# dimension = "behavior"
# case = "u_u_test__test_decode_uc650cc5"
# subject = "cpython.test_uu.UUTest.test_decode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uu.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_uu
_suite = unittest.defaultTestLoader.loadTestsFromName("UUTest.test_decode", test_uu)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UUTest.test_decode did not pass"
print("UUTest::test_decode: ok")
