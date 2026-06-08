# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "side_effect_iterable_yields_in_order"
# subject = "unittest.mock.Mock.side_effect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.side_effect: a side_effect set to an iterable returns its elements one per call in order"""
from unittest.mock import MagicMock

m = MagicMock(side_effect=[1, 2, 3])
assert m() == 1
assert m() == 2
assert m() == 3
print("side_effect_iterable_yields_in_order OK")
