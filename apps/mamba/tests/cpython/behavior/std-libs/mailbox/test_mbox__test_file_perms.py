# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "behavior"
# case = "test_mbox__test_file_perms"
# subject = "cpython.test_mailbox.TestMbox.test_file_perms"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailbox.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mailbox
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMbox.test_file_perms", test_mailbox)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMbox.test_file_perms did not pass"
print("TestMbox::test_file_perms: ok")
