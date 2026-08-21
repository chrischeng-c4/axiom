# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "patch_is_callable"
# subject = "unittest.mock.patch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.patch: patch_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.patch)
print("patch_is_callable OK")
