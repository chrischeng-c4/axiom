# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "behavior"
# case = "code_info_tests__test_code_info_uc28b882"
# subject = "cpython.test_dis.CodeInfoTests.test_code_info"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dis.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dis
_suite = unittest.defaultTestLoader.loadTestsFromName("CodeInfoTests.test_code_info", test_dis)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CodeInfoTests.test_code_info did not pass"
print("CodeInfoTests::test_code_info: ok")
