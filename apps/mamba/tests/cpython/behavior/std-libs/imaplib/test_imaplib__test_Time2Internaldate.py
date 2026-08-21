# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "behavior"
# case = "test_imaplib__test_Time2Internaldate"
# subject = "cpython.test_imaplib.TestImaplib.test_Time2Internaldate"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imaplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_imaplib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestImaplib.test_Time2Internaldate", test_imaplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestImaplib.test_Time2Internaldate did not pass"
print("TestImaplib::test_Time2Internaldate: ok")
