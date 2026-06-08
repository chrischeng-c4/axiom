# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_encode_maxlinelen_too_small"
# subject = "cpython.test_email.TestQuopri.test_encode_maxlinelen_too_small"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_email
_suite = unittest.defaultTestLoader.loadTestsFromName("TestQuopri.test_encode_maxlinelen_too_small", test_email)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestQuopri.test_encode_maxlinelen_too_small did not pass"
print("TestQuopri::test_encode_maxlinelen_too_small: ok")
