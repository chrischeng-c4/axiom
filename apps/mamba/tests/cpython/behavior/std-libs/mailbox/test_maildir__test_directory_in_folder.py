# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "behavior"
# case = "test_maildir__test_directory_in_folder"
# subject = "cpython.test_mailbox.TestMaildir.test_directory_in_folder"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailbox.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mailbox
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMaildir.test_directory_in_folder", test_mailbox)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMaildir.test_directory_in_folder did not pass"
print("TestMaildir::test_directory_in_folder: ok")
