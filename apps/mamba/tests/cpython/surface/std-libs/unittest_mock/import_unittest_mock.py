# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "import_unittest_mock"
# subject = "unittest.mock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock: import_unittest_mock (surface)."""
import unittest.mock

assert hasattr(unittest.mock, "MagicMock")
print("import_unittest_mock OK")
