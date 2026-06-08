# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "utf8_mode"
# dimension = "behavior"
# case = "u_t_f8_mode_tests__test_device_encoding_uc656cd2"
# subject = "cpython.test_utf8_mode.UTF8ModeTests.test_device_encoding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_utf8_mode.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_utf8_mode
_suite = unittest.defaultTestLoader.loadTestsFromName("UTF8ModeTests.test_device_encoding", test_utf8_mode)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UTF8ModeTests.test_device_encoding did not pass"
print("UTF8ModeTests::test_device_encoding: ok")
