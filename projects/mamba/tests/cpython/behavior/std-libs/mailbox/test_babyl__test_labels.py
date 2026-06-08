# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "behavior"
# case = "test_babyl__test_labels"
# subject = "cpython.test_mailbox.TestBabyl.test_labels"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailbox.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mailbox
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBabyl.test_labels", test_mailbox)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBabyl.test_labels did not pass"
print("TestBabyl::test_labels: ok")
