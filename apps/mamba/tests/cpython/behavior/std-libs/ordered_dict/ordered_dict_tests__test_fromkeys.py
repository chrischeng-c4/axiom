# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ordered_dict"
# dimension = "behavior"
# case = "ordered_dict_tests__test_fromkeys"
# subject = "cpython.test_ordered_dict.OrderedDictTests.test_fromkeys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ordered_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ordered_dict.py::OrderedDictTests::test_fromkeys
"""Auto-ported test: OrderedDictTests::test_fromkeys (CPython 3.12 oracle)."""


from collections import OrderedDict


od = OrderedDict.fromkeys("abc")
assert list(od.items()) == [(c, None) for c in "abc"], od

od = OrderedDict.fromkeys("abc", value=None)
assert list(od.items()) == [(c, None) for c in "abc"], od

od = OrderedDict.fromkeys("abc", value=0)
assert list(od.items()) == [(c, 0) for c in "abc"], od

print("OrderedDictTests::test_fromkeys: ok")
