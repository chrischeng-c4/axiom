# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "locale_codec_test__test_decode_surrogateescape"
# subject = "cpython.test_codecs.LocaleCodecTest.test_decode_surrogateescape"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaleCodecTest.test_decode_surrogateescape", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaleCodecTest.test_decode_surrogateescape did not pass"
print("LocaleCodecTest::test_decode_surrogateescape: ok")
