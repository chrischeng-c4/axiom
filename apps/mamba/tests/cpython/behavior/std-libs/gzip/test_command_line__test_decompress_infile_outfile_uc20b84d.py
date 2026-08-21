# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_command_line__test_decompress_infile_outfile_uc20b84d"
# subject = "cpython.test_gzip.TestCommandLine.test_decompress_infile_outfile"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gzip
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCommandLine.test_decompress_infile_outfile", test_gzip)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCommandLine.test_decompress_infile_outfile did not pass"
print("TestCommandLine::test_decompress_infile_outfile: ok")
