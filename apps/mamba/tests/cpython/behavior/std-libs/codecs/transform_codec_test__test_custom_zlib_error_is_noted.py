# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "transform_codec_test__test_custom_zlib_error_is_noted"
# subject = "cpython.test_codecs.TransformCodecTest.test_custom_zlib_error_is_noted"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("TransformCodecTest.test_custom_zlib_error_is_noted", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TransformCodecTest.test_custom_zlib_error_is_noted did not pass"
print("TransformCodecTest::test_custom_zlib_error_is_noted: ok")
