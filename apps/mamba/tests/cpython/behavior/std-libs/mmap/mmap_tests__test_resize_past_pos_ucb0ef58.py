# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "behavior"
# case = "mmap_tests__test_resize_past_pos_ucb0ef58"
# subject = "cpython.test_mmap.MmapTests.test_resize_past_pos"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mmap.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mmap
_suite = unittest.defaultTestLoader.loadTestsFromName("MmapTests.test_resize_past_pos", test_mmap)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MmapTests.test_resize_past_pos did not pass"
print("MmapTests::test_resize_past_pos: ok")
