# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errno_attribute_tests__test_for_improper_attributes"
# subject = "cpython.test_errno.ErrnoAttributeTests.test_for_improper_attributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_errno.py::ErrnoAttributeTests::test_for_improper_attributes
"""Auto-ported test: ErrnoAttributeTests::test_for_improper_attributes (CPython 3.12 oracle)."""


import errno
import unittest


'Test the errno module\n   Roger E. Masse\n'

std_c_errors = frozenset(['EDOM', 'ERANGE'])


# --- test body ---
for error_code in std_c_errors:

    assert hasattr(errno, error_code)
print("ErrnoAttributeTests::test_for_improper_attributes: ok")
