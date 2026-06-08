# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_tests__test_attributes_in_errorcode"
# subject = "cpython.test_errno.ErrorcodeTests.test_attributes_in_errorcode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_errno.py::ErrorcodeTests::test_attributes_in_errorcode
"""Auto-ported test: ErrorcodeTests::test_attributes_in_errorcode (CPython 3.12 oracle)."""


import errno
import unittest


'Test the errno module\n   Roger E. Masse\n'

std_c_errors = frozenset(['EDOM', 'ERANGE'])


# --- test body ---
for attribute in errno.__dict__.keys():
    if attribute.isupper():

        assert getattr(errno, attribute) in errno.errorcode
print("ErrorcodeTests::test_attributes_in_errorcode: ok")
