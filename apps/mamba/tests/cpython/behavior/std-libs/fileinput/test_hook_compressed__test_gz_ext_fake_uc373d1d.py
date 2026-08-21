# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "behavior"
# case = "test_hook_compressed__test_gz_ext_fake_uc373d1d"
# subject = "cpython.test_fileinput.Test_hook_compressed.test_gz_ext_fake"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fileinput.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fileinput
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_hook_compressed.test_gz_ext_fake", test_fileinput)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_hook_compressed.test_gz_ext_fake did not pass"
print("Test_hook_compressed::test_gz_ext_fake: ok")
