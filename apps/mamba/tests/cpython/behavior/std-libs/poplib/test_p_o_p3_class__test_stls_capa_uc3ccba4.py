# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "behavior"
# case = "test_p_o_p3_class__test_stls_capa_uc3ccba4"
# subject = "cpython.test_poplib.TestPOP3Class.test_stls_capa"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_poplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_poplib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestPOP3Class.test_stls_capa", test_poplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestPOP3Class.test_stls_capa did not pass"
print("TestPOP3Class::test_stls_capa: ok")
