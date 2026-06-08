# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "propertymock_intercepts_attribute"
# subject = "unittest.mock.PropertyMock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.PropertyMock: a PropertyMock installed on a class intercepts attribute reads, returns its return_value, and records the access"""
from unittest.mock import PropertyMock

pm = PropertyMock(return_value="pv")


class O:
    x = pm


o = O()
assert o.x == "pv"
pm.assert_called_once_with()
print("propertymock_intercepts_attribute OK")
