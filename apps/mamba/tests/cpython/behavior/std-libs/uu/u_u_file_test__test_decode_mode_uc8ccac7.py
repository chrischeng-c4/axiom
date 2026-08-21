# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uu"
# dimension = "behavior"
# case = "u_u_file_test__test_decode_mode_uc8ccac7"
# subject = "cpython.test_uu.UUFileTest.test_decode_mode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uu.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_uu
_suite = unittest.defaultTestLoader.loadTestsFromName("UUFileTest.test_decode_mode", test_uu)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UUFileTest.test_decode_mode did not pass"
print("UUFileTest::test_decode_mode: ok")
