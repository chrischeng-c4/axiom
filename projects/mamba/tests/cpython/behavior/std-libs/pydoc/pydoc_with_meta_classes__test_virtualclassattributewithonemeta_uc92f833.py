# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_with_meta_classes__test_virtualclassattributewithonemeta_uc92f833"
# subject = "cpython.test_pydoc.PydocWithMetaClasses.test_virtualClassAttributeWithOneMeta"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("PydocWithMetaClasses.test_virtualClassAttributeWithOneMeta", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PydocWithMetaClasses.test_virtualClassAttributeWithOneMeta did not pass"
print("PydocWithMetaClasses::test_virtualClassAttributeWithOneMeta: ok")
