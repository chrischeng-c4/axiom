# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtin"
# dimension = "behavior"
# case = "immortal_tests__test_tuple_repeat_respect_immortality_uc26bd8d"
# subject = "cpython.test_builtin.ImmortalTests.test_tuple_repeat_respect_immortality"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_builtin
_suite = unittest.defaultTestLoader.loadTestsFromName("ImmortalTests.test_tuple_repeat_respect_immortality", test_builtin)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImmortalTests.test_tuple_repeat_respect_immortality did not pass"
print("ImmortalTests::test_tuple_repeat_respect_immortality: ok")
