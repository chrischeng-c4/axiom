# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "patch_object_is_callable"
# subject = "unittest.mock.patch.object"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.patch.object: patch_object_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.patch.object)
print("patch_object_is_callable OK")
