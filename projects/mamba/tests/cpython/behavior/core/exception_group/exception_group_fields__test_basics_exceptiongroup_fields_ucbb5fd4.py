# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "exception_group_fields__test_basics_exceptiongroup_fields_ucbb5fd4"
# subject = "cpython.test_exception_group.ExceptionGroupFields.test_basics_ExceptionGroup_fields"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exception_group
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionGroupFields.test_basics_ExceptionGroup_fields", test_exception_group)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionGroupFields.test_basics_ExceptionGroup_fields did not pass"
print("ExceptionGroupFields::test_basics_ExceptionGroup_fields: ok")
