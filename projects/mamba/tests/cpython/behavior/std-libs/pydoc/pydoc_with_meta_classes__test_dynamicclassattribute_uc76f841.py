# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_with_meta_classes__test_dynamicclassattribute_uc76f841"
# subject = "cpython.test_pydoc.PydocWithMetaClasses.test_DynamicClassAttribute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("PydocWithMetaClasses.test_DynamicClassAttribute", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PydocWithMetaClasses.test_DynamicClassAttribute did not pass"
print("PydocWithMetaClasses::test_DynamicClassAttribute: ok")
