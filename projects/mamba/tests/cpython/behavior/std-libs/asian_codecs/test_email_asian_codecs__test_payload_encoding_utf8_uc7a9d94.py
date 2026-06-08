# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asian_codecs"
# dimension = "behavior"
# case = "test_email_asian_codecs__test_payload_encoding_utf8_uc7a9d94"
# subject = "cpython.test_asian_codecs.TestEmailAsianCodecs.test_payload_encoding_utf8"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_asian_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_asian_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEmailAsianCodecs.test_payload_encoding_utf8", test_asian_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEmailAsianCodecs.test_payload_encoding_utf8 did not pass"
print("TestEmailAsianCodecs::test_payload_encoding_utf8: ok")
