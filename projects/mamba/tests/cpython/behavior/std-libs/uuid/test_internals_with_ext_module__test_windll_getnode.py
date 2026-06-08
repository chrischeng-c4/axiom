# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "test_internals_with_ext_module__test_windll_getnode"
# subject = "cpython.test_uuid.TestInternalsWithExtModule.test_windll_getnode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_uuid
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInternalsWithExtModule.test_windll_getnode", test_uuid)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInternalsWithExtModule.test_windll_getnode did not pass"
print("TestInternalsWithExtModule::test_windll_getnode: ok")
