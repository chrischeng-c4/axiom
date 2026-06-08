# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "patch_dict_is_callable"
# subject = "unittest.mock.patch.dict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.patch.dict: patch_dict_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.patch.dict)
print("patch_dict_is_callable OK")
