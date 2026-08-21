# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "behavior"
# case = "test_t_l_s__f_t_p_class__test_ccc"
# subject = "cpython.test_ftplib.TestTLS_FTPClass.test_ccc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ftplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ftplib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTLS_FTPClass.test_ccc", test_ftplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTLS_FTPClass.test_ccc did not pass"
print("TestTLS_FTPClass::test_ccc: ok")
