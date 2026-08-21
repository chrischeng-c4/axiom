# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "link_emulation_test__test_hardlink_extraction1"
# subject = "cpython.test_tarfile.LinkEmulationTest.test_hardlink_extraction1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("LinkEmulationTest.test_hardlink_extraction1", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LinkEmulationTest.test_hardlink_extraction1 did not pass"
print("LinkEmulationTest::test_hardlink_extraction1: ok")
