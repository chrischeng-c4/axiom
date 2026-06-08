# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_sizeof_uc354ac7"
# subject = "cpython.test_mmap.MmapTests.test_sizeof"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mmap
_suite = unittest.defaultTestLoader.loadTestsFromName("MmapTests.test_sizeof", test_mmap)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MmapTests.test_sizeof did not pass"
print("MmapTests::test_sizeof: ok")
