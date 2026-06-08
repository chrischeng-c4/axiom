# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "offset_validation_tests__test_invalid_offset_header_validations"
# subject = "cpython.test_tarfile.OffsetValidationTests.test_invalid_offset_header_validations"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("OffsetValidationTests.test_invalid_offset_header_validations", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OffsetValidationTests.test_invalid_offset_header_validations did not pass"
print("OffsetValidationTests::test_invalid_offset_header_validations: ok")
