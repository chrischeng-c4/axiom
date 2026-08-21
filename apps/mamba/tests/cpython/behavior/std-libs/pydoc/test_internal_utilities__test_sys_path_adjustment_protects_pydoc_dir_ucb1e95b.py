# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "test_internal_utilities__test_sys_path_adjustment_protects_pydoc_dir_ucb1e95b"
# subject = "cpython.test_pydoc.TestInternalUtilities.test_sys_path_adjustment_protects_pydoc_dir"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInternalUtilities.test_sys_path_adjustment_protects_pydoc_dir", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInternalUtilities.test_sys_path_adjustment_protects_pydoc_dir did not pass"
print("TestInternalUtilities::test_sys_path_adjustment_protects_pydoc_dir: ok")
