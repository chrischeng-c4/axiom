# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "waitfor"
# dimension = "behavior"
# case = "wait_for_shield_tests__test_shielded_timeout_uc12e1a0"
# subject = "cpython.test_waitfor.WaitForShieldTests.test_shielded_timeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_waitfor.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_waitfor
_suite = unittest.defaultTestLoader.loadTestsFromName("WaitForShieldTests.test_shielded_timeout", test_waitfor)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WaitForShieldTests.test_shielded_timeout did not pass"
print("WaitForShieldTests::test_shielded_timeout: ok")
