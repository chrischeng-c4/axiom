# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "behavior"
# case = "threaded_networked_tests__test_enable_UTF8_True_append"
# subject = "cpython.test_imaplib.ThreadedNetworkedTests.test_enable_UTF8_True_append"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imaplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_imaplib
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadedNetworkedTests.test_enable_UTF8_True_append", test_imaplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadedNetworkedTests.test_enable_UTF8_True_append did not pass"
print("ThreadedNetworkedTests::test_enable_UTF8_True_append: ok")
