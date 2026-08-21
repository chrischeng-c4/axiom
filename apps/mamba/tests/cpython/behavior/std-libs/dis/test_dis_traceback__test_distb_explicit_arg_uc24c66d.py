# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "behavior"
# case = "test_dis_traceback__test_distb_explicit_arg_uc24c66d"
# subject = "cpython.test_dis.TestDisTraceback.test_distb_explicit_arg"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dis.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dis
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDisTraceback.test_distb_explicit_arg", test_dis)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDisTraceback.test_distb_explicit_arg did not pass"
print("TestDisTraceback::test_distb_explicit_arg: ok")
