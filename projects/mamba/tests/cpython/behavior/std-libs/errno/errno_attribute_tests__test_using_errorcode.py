# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errno_attribute_tests__test_using_errorcode"
# subject = "cpython.test_errno.ErrnoAttributeTests.test_using_errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_errno.py::ErrnoAttributeTests::test_using_errorcode
"""Auto-ported test: ErrnoAttributeTests::test_using_errorcode (CPython 3.12 oracle)."""


import errno
import unittest


'Test the errno module\n   Roger E. Masse\n'

std_c_errors = frozenset(['EDOM', 'ERANGE'])


# --- test body ---
for value in errno.errorcode.values():

    assert hasattr(errno, value)
print("ErrnoAttributeTests::test_using_errorcode: ok")
