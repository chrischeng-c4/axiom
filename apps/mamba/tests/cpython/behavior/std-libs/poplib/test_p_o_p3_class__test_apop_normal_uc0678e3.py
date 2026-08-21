# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "behavior"
# case = "test_p_o_p3_class__test_apop_normal_uc0678e3"
# subject = "cpython.test_poplib.TestPOP3Class.test_apop_normal"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_poplib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPOP3Class.test_apop_normal", test_poplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPOP3Class.test_apop_normal did not pass"
print("TestPOP3Class::test_apop_normal: ok")
