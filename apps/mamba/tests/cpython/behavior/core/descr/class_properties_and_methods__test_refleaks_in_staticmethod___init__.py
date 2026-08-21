# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_refleaks_in_staticmethod___init__"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_refleaks_in_staticmethod___init__"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_descr
_suite = unittest.defaultTestLoader.loadTestsFromName("ClassPropertiesAndMethods.test_refleaks_in_staticmethod___init__", test_descr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ClassPropertiesAndMethods.test_refleaks_in_staticmethod___init__ did not pass"
print("ClassPropertiesAndMethods::test_refleaks_in_staticmethod___init__: ok")
