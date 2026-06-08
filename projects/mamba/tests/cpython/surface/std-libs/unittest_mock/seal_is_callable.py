# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "seal_is_callable"
# subject = "unittest.mock.seal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.seal: seal_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.seal)
print("seal_is_callable OK")
