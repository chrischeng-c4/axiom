# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "transform_codec_test__test_binary_to_text_denylists_text_transforms"
# subject = "cpython.test_codecs.TransformCodecTest.test_binary_to_text_denylists_text_transforms"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("TransformCodecTest.test_binary_to_text_denylists_text_transforms", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TransformCodecTest.test_binary_to_text_denylists_text_transforms did not pass"
print("TransformCodecTest::test_binary_to_text_denylists_text_transforms: ok")
