# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_command_line__test_compress_infile_outfile_default_uc446cb0"
# subject = "cpython.test_gzip.TestCommandLine.test_compress_infile_outfile_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gzip
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCommandLine.test_compress_infile_outfile_default", test_gzip)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCommandLine.test_compress_infile_outfile_default did not pass"
print("TestCommandLine::test_compress_infile_outfile_default: ok")
