# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "base_xy_test_case__test_error_heritage"
# subject = "cpython.test_base64.BaseXYTestCase.test_ErrorHeritage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_base64.py::BaseXYTestCase::test_ErrorHeritage
"""Auto-ported test: BaseXYTestCase::test_ErrorHeritage (CPython 3.12 oracle)."""


import unittest
import base64
import binascii
import os
from array import array
from test.support import os_helper
from test.support import script_helper


# --- test body ---

assert issubclass(binascii.Error, ValueError)
print("BaseXYTestCase::test_ErrorHeritage: ok")
