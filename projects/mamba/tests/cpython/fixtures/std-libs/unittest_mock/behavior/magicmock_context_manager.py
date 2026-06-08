# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "magicmock_context_manager"
# subject = "unittest.mock.MagicMock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testwith.py"
# status = "filled"
# ///
"""unittest.mock.MagicMock: MagicMock works as a context manager: configuring __enter__.return_value makes `with mock as x` bind that value"""
from unittest.mock import MagicMock

m = MagicMock()
m.__enter__.return_value = "ctx"
with m as c:
    assert c == "ctx"
print("magicmock_context_manager OK")
