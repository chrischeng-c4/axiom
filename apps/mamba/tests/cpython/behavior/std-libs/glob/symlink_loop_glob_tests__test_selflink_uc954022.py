# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "symlink_loop_glob_tests__test_selflink_uc954022"
# subject = "cpython.test_glob.SymlinkLoopGlobTests.test_selflink"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_glob
_suite = unittest.defaultTestLoader.loadTestsFromName("SymlinkLoopGlobTests.test_selflink", test_glob)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SymlinkLoopGlobTests.test_selflink did not pass"
print("SymlinkLoopGlobTests::test_selflink: ok")
