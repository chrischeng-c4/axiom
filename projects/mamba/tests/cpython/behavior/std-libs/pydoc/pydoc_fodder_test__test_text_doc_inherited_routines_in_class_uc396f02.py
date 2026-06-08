# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pydoc"
# dimension = "behavior"
# case = "pydoc_fodder_test__test_text_doc_inherited_routines_in_class_uc396f02"
# subject = "cpython.test_pydoc.PydocFodderTest.test_text_doc_inherited_routines_in_class"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pydoc/test_pydoc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_pydoc import test_pydoc
_suite = unittest.defaultTestLoader.loadTestsFromName("PydocFodderTest.test_text_doc_inherited_routines_in_class", test_pydoc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PydocFodderTest.test_text_doc_inherited_routines_in_class did not pass"
print("PydocFodderTest::test_text_doc_inherited_routines_in_class: ok")
