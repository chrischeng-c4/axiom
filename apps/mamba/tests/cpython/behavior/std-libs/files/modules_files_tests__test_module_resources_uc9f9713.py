# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "files"
# dimension = "behavior"
# case = "modules_files_tests__test_module_resources_uc9f9713"
# subject = "cpython.test_files.ModulesFilesTests.test_module_resources"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_files.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_files
_suite = unittest.defaultTestLoader.loadTestsFromName("ModulesFilesTests.test_module_resources", test_files)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ModulesFilesTests.test_module_resources did not pass"
print("ModulesFilesTests::test_module_resources: ok")
