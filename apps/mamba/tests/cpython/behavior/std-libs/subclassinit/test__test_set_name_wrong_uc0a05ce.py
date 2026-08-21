# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_set_name_wrong_uc0a05ce"
# subject = "cpython.test_subclassinit.Test.test_set_name_wrong"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subclassinit
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_set_name_wrong", test_subclassinit)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_set_name_wrong did not pass"
print("Test::test_set_name_wrong: ok")
