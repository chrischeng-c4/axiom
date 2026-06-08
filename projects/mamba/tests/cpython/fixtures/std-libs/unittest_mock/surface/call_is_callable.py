# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "call_is_callable"
# subject = "unittest.mock.call"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.call: call_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.call)
print("call_is_callable OK")
