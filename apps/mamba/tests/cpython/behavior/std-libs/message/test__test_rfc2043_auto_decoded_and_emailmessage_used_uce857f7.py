# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "message"
# dimension = "behavior"
# case = "test__test_rfc2043_auto_decoded_and_emailmessage_used_uce857f7"
# subject = "cpython.test_message.Test.test_rfc2043_auto_decoded_and_emailmessage_used"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_message.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_message
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_rfc2043_auto_decoded_and_emailmessage_used", test_message)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_rfc2043_auto_decoded_and_emailmessage_used did not pass"
print("Test::test_rfc2043_auto_decoded_and_emailmessage_used: ok")
