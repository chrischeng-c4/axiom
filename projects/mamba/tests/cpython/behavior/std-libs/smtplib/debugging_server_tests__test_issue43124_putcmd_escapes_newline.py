# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "debugging_server_tests__test_issue43124_putcmd_escapes_newline"
# subject = "cpython.test_smtplib.DebuggingServerTests.test_issue43124_putcmd_escapes_newline"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_smtplib
_suite = unittest.defaultTestLoader.loadTestsFromName("DebuggingServerTests.test_issue43124_putcmd_escapes_newline", test_smtplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DebuggingServerTests.test_issue43124_putcmd_escapes_newline did not pass"
print("DebuggingServerTests::test_issue43124_putcmd_escapes_newline: ok")
