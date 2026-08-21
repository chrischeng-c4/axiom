# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "largefile"
# dimension = "behavior"
# case = "test_socket_sendfile__test_it_uc88202c"
# subject = "cpython.test_largefile.TestSocketSendfile.test_it"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_largefile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_largefile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSocketSendfile.test_it", test_largefile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSocketSendfile.test_it did not pass"
print("TestSocketSendfile::test_it: ok")
