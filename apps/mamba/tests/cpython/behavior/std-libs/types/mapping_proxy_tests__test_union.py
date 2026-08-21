# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "mapping_proxy_tests__test_union"
# subject = "cpython.test_types.MappingProxyTests.test_union"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("MappingProxyTests.test_union", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MappingProxyTests.test_union did not pass"
print("MappingProxyTests::test_union: ok")
