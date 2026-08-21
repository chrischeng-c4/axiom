# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading_local"
# dimension = "behavior"
# case = "thread_local_test__test_dict_attribute"
# subject = "cpython.test_threading_local.ThreadLocalTest.test_dict_attribute"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_threading_local.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_threading_local.py::ThreadLocalTest::test_dict_attribute
"""Auto-ported test: ThreadLocalTest::test_dict_attribute (CPython 3.12 oracle)."""


import _thread


obj = _thread._local()
obj.x = 5
assert obj.__dict__ == {"x": 5}, obj.__dict__

try:
    obj.__dict__ = {}
except AttributeError:
    pass
else:
    raise AssertionError("_thread._local.__dict__ assignment did not raise AttributeError")

try:
    del obj.__dict__
except AttributeError:
    pass
else:
    raise AssertionError("_thread._local.__dict__ deletion did not raise AttributeError")

print("ThreadLocalTest::test_dict_attribute: ok")
