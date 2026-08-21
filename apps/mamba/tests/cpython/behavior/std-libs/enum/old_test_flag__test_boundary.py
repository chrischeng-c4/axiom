# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "old_test_flag__test_boundary"
# subject = "cpython.test_enum.OldTestFlag.test_boundary"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enum
_suite = unittest.defaultTestLoader.loadTestsFromName("OldTestFlag.test_boundary", test_enum)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OldTestFlag.test_boundary did not pass"
print("OldTestFlag::test_boundary: ok")
