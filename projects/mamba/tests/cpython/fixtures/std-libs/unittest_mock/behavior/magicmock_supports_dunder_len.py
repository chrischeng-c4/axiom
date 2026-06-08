# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "magicmock_supports_dunder_len"
# subject = "unittest.mock.MagicMock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmagicmethods.py"
# status = "filled"
# ///
"""unittest.mock.MagicMock: MagicMock supports magic methods: setting __len__.return_value makes len(mock) return that integer"""
from unittest.mock import MagicMock

m = MagicMock()
m.__len__.return_value = 3
assert len(m) == 3
print("magicmock_supports_dunder_len OK")
