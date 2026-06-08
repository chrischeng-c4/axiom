# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userdict"
# dimension = "behavior"
# case = "user_dict_test__test_all"
# subject = "cpython.test_userdict.UserDictTest.test_all"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_userdict.py::UserDictTest::test_all
"""Auto-ported test: UserDictTest::test_all (CPython 3.12 oracle)."""


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
u = collections.UserDict()
u0 = collections.UserDict(d0)
u1 = collections.UserDict(d1)
u2 = collections.UserDict(d2)
uu = collections.UserDict(u)
uu0 = collections.UserDict(u0)
uu1 = collections.UserDict(u1)
uu2 = collections.UserDict(u2)

assert collections.UserDict(one=1, two=2) == d2

assert collections.UserDict([('one', 1), ('two', 2)]) == d2

assert collections.UserDict(dict=[('one', 1), ('two', 2)]) == {'dict': [('one', 1), ('two', 2)]}

assert collections.UserDict([('one', 1), ('two', 2)], two=3, three=5) == d3

assert collections.UserDict.fromkeys('one two'.split()) == d4

assert collections.UserDict().fromkeys('one two'.split()) == d4

assert collections.UserDict.fromkeys('one two'.split(), 1) == d5

assert collections.UserDict().fromkeys('one two'.split(), 1) == d5

assert u1.fromkeys('one two'.split()) is not u1

assert isinstance(u1.fromkeys('one two'.split()), collections.UserDict)

assert isinstance(u2.fromkeys('one two'.split()), collections.UserDict)

assert str(u0) == str(d0)

assert repr(u1) == repr(d1)

assert repr(u2) in ("{'one': 1, 'two': 2}", "{'two': 2, 'one': 1}")
all = [d0, d1, d2, u, u0, u1, u2, uu, uu0, uu1, uu2]
for a in all:
    for b in all:

        assert (a == b) == (len(a) == len(b))

assert u2['one'] == 1

try:
    u1.__getitem__('two')
    raise AssertionError('expected KeyError')
except KeyError:
    pass
u3 = collections.UserDict(u2)
u3['two'] = 2
u3['three'] = 3
del u3['three']

try:
    u3.__delitem__('three')
    raise AssertionError('expected KeyError')
except KeyError:
    pass
u3.clear()

assert u3 == {}
u2a = u2.copy()

assert u2a == u2
u2b = collections.UserDict(x=42, y=23)
u2c = u2b.copy()

assert u2b == u2c

class MyUserDict(collections.UserDict):

    def display(self):
        print(self)
m2 = MyUserDict(u2)
m2a = m2.copy()

assert m2a == m2
m2['foo'] = 'bar'

assert m2a != m2

assert sorted(u2.keys()) == sorted(d2.keys())

assert sorted(u2.items()) == sorted(d2.items())

assert sorted(u2.values()) == sorted(d2.values())
for i in u2.keys():

    assert i in u2

    assert (i in u1) == (i in d1)

    assert (i in u0) == (i in d0)
t = collections.UserDict()
t.update(u2)

assert t == u2
for i in u2.keys():

    assert u2.get(i) == u2[i]

    assert u1.get(i) == d1.get(i)

    assert u0.get(i) == d0.get(i)
for i in range(20):
    u2[i] = str(i)
ikeys = []
for k in u2:
    ikeys.append(k)
keys = u2.keys()

assert set(ikeys) == set(keys)
t = collections.UserDict()

assert t.setdefault('x', 42) == 42

assert 'x' in t

assert t.setdefault('x', 23) == 42
t = collections.UserDict(x=42)

assert t.pop('x') == 42

try:
    t.pop('x')
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert t.pop('x', 1) == 1
t['x'] = 42

assert t.pop('x', 1) == 42
t = collections.UserDict(x=42)

assert t.popitem() == ('x', 42)

try:
    t.popitem()
    raise AssertionError('expected KeyError')
except KeyError:
    pass
print("UserDictTest::test_all: ok")
