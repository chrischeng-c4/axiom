# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "global"
# dimension = "behavior"
# case = "global_tests__test3_uc128857"
# subject = "cpython.test_global.GlobalTests.test3"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_global.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_global
_suite = unittest.defaultTestLoader.loadTestsFromName("GlobalTests.test3", test_global)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GlobalTests.test3 did not pass"
print("GlobalTests::test3: ok")
