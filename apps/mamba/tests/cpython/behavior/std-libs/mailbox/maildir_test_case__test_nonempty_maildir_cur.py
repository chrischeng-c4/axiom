# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "behavior"
# case = "maildir_test_case__test_nonempty_maildir_cur"
# subject = "cpython.test_mailbox.MaildirTestCase.test_nonempty_maildir_cur"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailbox.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mailbox
_suite = unittest.defaultTestLoader.loadTestsFromName("MaildirTestCase.test_nonempty_maildir_cur", test_mailbox)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MaildirTestCase.test_nonempty_maildir_cur did not pass"
print("MaildirTestCase::test_nonempty_maildir_cur: ok")
