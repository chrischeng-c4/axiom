# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userdict"
# dimension = "behavior"
# case = "user_dict_test__test_missing"
# subject = "cpython.test_userdict.UserDictTest.test_missing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_userdict.py::UserDictTest::test_missing
"""Auto-ported test: UserDictTest::test_missing (CPython 3.12 oracle)."""


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

assert hasattr(collections.UserDict, '__missing__') == False

class D(collections.UserDict):

    def __missing__(self, key):
        return 42
d = D({1: 2, 3: 4})

assert d[1] == 2

assert d[3] == 4

assert 2 not in d

assert 2 not in d.keys()

assert d[2] == 42

class E(collections.UserDict):

    def __missing__(self, key):
        raise RuntimeError(key)
e = E()
try:
    e[42]
except RuntimeError as err:

    assert err.args == (42,)
else:

    raise AssertionError("e[42] didn't raise RuntimeError")

class F(collections.UserDict):

    def __init__(self):
        self.__missing__ = lambda key: None
        collections.UserDict.__init__(self)
f = F()
try:
    f[42]
except KeyError as err:

    assert err.args == (42,)
else:

    raise AssertionError("f[42] didn't raise KeyError")

class G(collections.UserDict):
    pass
g = G()
try:
    g[42]
except KeyError as err:

    assert err.args == (42,)
else:

    raise AssertionError("g[42] didn't raise KeyError")
print("UserDictTest::test_missing: ok")
