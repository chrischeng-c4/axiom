# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imghdr"
# dimension = "behavior"
# case = "test_imghdr__test_file_pos_uce67d3a"
# subject = "cpython.test_imghdr.TestImghdr.test_file_pos"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_imghdr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_imghdr
_suite = unittest.defaultTestLoader.loadTestsFromName("TestImghdr.test_file_pos", test_imghdr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestImghdr.test_file_pos did not pass"
print("TestImghdr::test_file_pos: ok")
