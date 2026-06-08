# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "behavior"
# case = "send_reader_n_n_t_pv2_tests__test_we_are_in_reader_mode_after_connect"
# subject = "cpython.test_nntplib.SendReaderNNTPv2Tests.test_we_are_in_reader_mode_after_connect"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nntplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_nntplib
_suite = unittest.defaultTestLoader.loadTestsFromName("SendReaderNNTPv2Tests.test_we_are_in_reader_mode_after_connect", test_nntplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SendReaderNNTPv2Tests.test_we_are_in_reader_mode_after_connect did not pass"
print("SendReaderNNTPv2Tests::test_we_are_in_reader_mode_after_connect: ok")
