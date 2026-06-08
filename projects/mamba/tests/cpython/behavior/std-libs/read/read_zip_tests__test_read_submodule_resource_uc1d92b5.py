# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "read"
# dimension = "behavior"
# case = "read_zip_tests__test_read_submodule_resource_uc1d92b5"
# subject = "cpython.test_read.ReadZipTests.test_read_submodule_resource"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_read.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_read
_suite = unittest.defaultTestLoader.loadTestsFromName("ReadZipTests.test_read_submodule_resource", test_read)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReadZipTests.test_read_submodule_resource did not pass"
print("ReadZipTests::test_read_submodule_resource: ok")
