# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_args_args_kwargs_accessors"
# subject = "unittest.mock.Mock.call_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_args: call_args exposes .args (a tuple of positionals) and .kwargs (a dict of keywords) for the last call m(1, 2, k=3)"""
from unittest.mock import MagicMock

m = MagicMock()
m(1, 2, k=3)
assert m.call_args.args == (1, 2)
assert m.call_args.kwargs == {"k": 3}
print("call_args_args_kwargs_accessors OK")
