# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pep492"
# dimension = "behavior"
# case = "stream_reader_tests__test_readline_uc9732ea"
# subject = "cpython.test_pep492.StreamReaderTests.test_readline"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_pep492.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_pep492
_suite = unittest.defaultTestLoader.loadTestsFromName("StreamReaderTests.test_readline", test_pep492)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StreamReaderTests.test_readline did not pass"
print("StreamReaderTests::test_readline: ok")
