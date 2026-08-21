# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asian_codecs"
# dimension = "behavior"
# case = "test_email_asian_codecs__test_japanese_codecs_ucfcf106"
# subject = "cpython.test_asian_codecs.TestEmailAsianCodecs.test_japanese_codecs"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_asian_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_asian_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEmailAsianCodecs.test_japanese_codecs", test_asian_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEmailAsianCodecs.test_japanese_codecs did not pass"
print("TestEmailAsianCodecs::test_japanese_codecs: ok")
