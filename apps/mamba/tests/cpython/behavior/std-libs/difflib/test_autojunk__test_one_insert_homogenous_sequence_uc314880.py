# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_autojunk__test_one_insert_homogenous_sequence_uc314880"
# subject = "cpython.test_difflib.TestAutojunk.test_one_insert_homogenous_sequence"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_difflib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAutojunk.test_one_insert_homogenous_sequence", test_difflib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAutojunk.test_one_insert_homogenous_sequence did not pass"
print("TestAutojunk::test_one_insert_homogenous_sequence: ok")
