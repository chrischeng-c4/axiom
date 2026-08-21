# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "any_matches_any_argument"
# subject = "unittest.mock.ANY"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.ANY: mock.ANY compares equal to any value so assert_called_with(ANY, ANY) accepts any positional arguments"""
from unittest.mock import MagicMock, ANY

m = MagicMock()
m(123, "hi")
m.assert_called_with(ANY, ANY)
assert (ANY == 1) and (ANY == "x") and (ANY == object())
print("any_matches_any_argument OK")
