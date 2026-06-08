# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "attribute_access_autocreates_child_mock"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: accessing an undefined attribute auto-creates a child mock, the same object is returned on repeated access, and child calls are recorded on the child's call_args_list"""
from unittest.mock import MagicMock, call

m = MagicMock()
child = m.foo
assert m.foo is child  # same child returned on repeated access
m.foo.bar(1)
m.foo.bar(2)
assert m.foo.bar.call_args_list == [call(1), call(2)]
print("attribute_access_autocreates_child_mock OK")
