# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crashers"
# dimension = "behavior"
# case = "crasher_test__test_crashers_crash_uc7d09b1"
# subject = "cpython.test_crashers.CrasherTest.test_crashers_crash"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_crashers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_crashers
_suite = unittest.defaultTestLoader.loadTestsFromName("CrasherTest.test_crashers_crash", test_crashers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CrasherTest.test_crashers_crash did not pass"
print("CrasherTest::test_crashers_crash: ok")
