# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zip"
# dimension = "behavior"
# case = "test_egg__test_normalized_name_uc34de3d"
# subject = "cpython.test_zip.TestEgg.test_normalized_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_zip.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_zip
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEgg.test_normalized_name", test_zip)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEgg.test_normalized_name did not pass"
print("TestEgg::test_normalized_name: ok")
