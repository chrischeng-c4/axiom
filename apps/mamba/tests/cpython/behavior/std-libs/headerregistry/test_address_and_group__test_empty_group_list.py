# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "headerregistry"
# dimension = "behavior"
# case = "test_address_and_group__test_empty_group_list"
# subject = "cpython.test_headerregistry.TestAddressAndGroup.test_empty_group_list"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_headerregistry.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_headerregistry
_suite = unittest.defaultTestLoader.loadTestsFromName("TestAddressAndGroup.test_empty_group_list", test_headerregistry)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestAddressAndGroup.test_empty_group_list did not pass"
print("TestAddressAndGroup::test_empty_group_list: ok")
