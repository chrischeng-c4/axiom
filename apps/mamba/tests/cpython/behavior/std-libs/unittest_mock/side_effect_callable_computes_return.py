# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "side_effect_callable_computes_return"
# subject = "unittest.mock.Mock.side_effect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.side_effect: a callable side_effect is invoked with the call arguments and its result becomes the mock's return value"""
from unittest.mock import MagicMock

m = MagicMock(side_effect=lambda x: x * 10)
assert m(5) == 50
print("side_effect_callable_computes_return OK")
