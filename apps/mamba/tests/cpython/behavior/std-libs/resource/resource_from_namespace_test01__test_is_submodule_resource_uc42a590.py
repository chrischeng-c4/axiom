# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_from_namespace_test01__test_is_submodule_resource_uc42a590"
# subject = "cpython.test_resource.ResourceFromNamespaceTest01.test_is_submodule_resource"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_resource
_suite = unittest.defaultTestLoader.loadTestsFromName("ResourceFromNamespaceTest01.test_is_submodule_resource", test_resource)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ResourceFromNamespaceTest01.test_is_submodule_resource did not pass"
print("ResourceFromNamespaceTest01::test_is_submodule_resource: ok")
