# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "child_return_value_configuration"
# subject = "unittest.mock.Mock.return_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.return_value: configuring a deep child's return_value (m.child.method.return_value) makes the corresponding nested call return that value"""
from unittest.mock import MagicMock

m = MagicMock()
m.child.method.return_value = "z"
assert m.child.method() == "z"
print("child_return_value_configuration OK")
