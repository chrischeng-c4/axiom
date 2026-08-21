# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "test_plain_flag_class__test_none_member"
# subject = "cpython.test_enum.TestPlainFlagClass.test_none_member"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enum
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPlainFlagClass.test_none_member", test_enum)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPlainFlagClass.test_none_member did not pass"
print("TestPlainFlagClass::test_none_member: ok")
