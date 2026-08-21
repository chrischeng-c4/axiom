# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "headerregistry"
# dimension = "behavior"
# case = "test_header_registry__test_override_default_class_only_overrides_default"
# subject = "cpython.test_headerregistry.TestHeaderRegistry.test_override_default_class_only_overrides_default"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_headerregistry.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_email import test_headerregistry
_suite = unittest.defaultTestLoader.loadTestsFromName("TestHeaderRegistry.test_override_default_class_only_overrides_default", test_headerregistry)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestHeaderRegistry.test_override_default_class_only_overrides_default did not pass"
print("TestHeaderRegistry::test_override_default_class_only_overrides_default: ok")
