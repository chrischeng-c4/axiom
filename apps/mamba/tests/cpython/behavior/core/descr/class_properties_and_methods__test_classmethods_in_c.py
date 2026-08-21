# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_classmethods_in_c"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_classmethods_in_c"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_descr
_suite = unittest.defaultTestLoader.loadTestsFromName("ClassPropertiesAndMethods.test_classmethods_in_c", test_descr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ClassPropertiesAndMethods.test_classmethods_in_c did not pass"
print("ClassPropertiesAndMethods::test_classmethods_in_c: ok")
