# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userdict"
# dimension = "behavior"
# case = "user_dict_test__test_update"
# subject = "cpython.test_userdict.UserDictTest.test_update"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_userdict.py::UserDictTest::test_update
"""Auto-ported test: UserDictTest::test_update (CPython 3.12 oracle)."""


from test import mapping_tests
import unittest
import collections


d0 = {}

d1 = {'one': 1}

d2 = {'one': 1, 'two': 2}

d3 = {'one': 1, 'two': 3, 'three': 5}

d4 = {'one': None, 'two': None}

d5 = {'one': 1, 'two': 1}


# --- test body ---
type2test = collections.UserDict
for kw in ('self', 'dict', 'other', 'iterable'):
    d = collections.UserDict()
    d.update(**{kw: 42})

    assert list(d.items()) == [(kw, 42)]

try:
    collections.UserDict().update(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    collections.UserDict().update({}, {})
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    collections.UserDict.update()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("UserDictTest::test_update: ok")
