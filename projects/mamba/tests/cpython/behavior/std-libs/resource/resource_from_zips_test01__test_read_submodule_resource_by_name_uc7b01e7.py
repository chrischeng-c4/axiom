# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_from_zips_test01__test_read_submodule_resource_by_name_uc7b01e7"
# subject = "cpython.test_resource.ResourceFromZipsTest01.test_read_submodule_resource_by_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_resource
_suite = unittest.defaultTestLoader.loadTestsFromName("ResourceFromZipsTest01.test_read_submodule_resource_by_name", test_resource)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ResourceFromZipsTest01.test_read_submodule_resource_by_name did not pass"
print("ResourceFromZipsTest01::test_read_submodule_resource_by_name: ok")
