# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "behavior"
# case = "test_f_t_p_class__test_set_pasv"
# subject = "cpython.test_ftplib.TestFTPClass.test_set_pasv"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ftplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ftplib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFTPClass.test_set_pasv", test_ftplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFTPClass.test_set_pasv did not pass"
print("TestFTPClass::test_set_pasv: ok")
