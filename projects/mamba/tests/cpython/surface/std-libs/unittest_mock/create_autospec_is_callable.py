# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "create_autospec_is_callable"
# subject = "unittest.mock.create_autospec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.create_autospec: create_autospec_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.create_autospec)
print("create_autospec_is_callable OK")
