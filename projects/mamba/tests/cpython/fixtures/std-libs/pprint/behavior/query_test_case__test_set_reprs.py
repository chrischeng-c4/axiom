# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "query_test_case__test_set_reprs"
# subject = "cpython.test_pprint.QueryTestCase.test_set_reprs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pprint.py::QueryTestCase::test_set_reprs
"""Auto-ported test: QueryTestCase::test_set_reprs (CPython 3.12 oracle)."""


import collections
import contextlib
import dataclasses
import io
import itertools
import pprint
import random
import test.support
import test.test_set
import types
import unittest


class list2(list):
    pass

class list3(list):

    def __repr__(self):
        return list.__repr__(self)

class list_custom_repr(list):

    def __repr__(self):
        return '*' * len(list.__repr__(self))

class tuple2(tuple):
    pass

class tuple3(tuple):

    def __repr__(self):
        return tuple.__repr__(self)

class tuple_custom_repr(tuple):

    def __repr__(self):
        return '*' * len(tuple.__repr__(self))

class set2(set):
    pass

class set3(set):

    def __repr__(self):
        return set.__repr__(self)

class set_custom_repr(set):

    def __repr__(self):
        return '*' * len(set.__repr__(self))

class frozenset2(frozenset):
    pass

class frozenset3(frozenset):

    def __repr__(self):
        return frozenset.__repr__(self)

class frozenset_custom_repr(frozenset):

    def __repr__(self):
        return '*' * len(frozenset.__repr__(self))

class dict2(dict):
    pass

class dict3(dict):

    def __repr__(self):
        return dict.__repr__(self)

class dict_custom_repr(dict):

    def __repr__(self):
        return '*' * len(dict.__repr__(self))

@dataclasses.dataclass
class dataclass1:
    field1: str
    field2: int
    field3: bool = False
    field4: int = dataclasses.field(default=1, repr=False)

@dataclasses.dataclass
class dataclass2:
    a: int = 1

    def __repr__(self):
        return "custom repr that doesn't fit within pprint width"

@dataclasses.dataclass(repr=False)
class dataclass3:
    a: int = 1

@dataclasses.dataclass
class dataclass4:
    a: 'dataclass4'
    b: int = 1

@dataclasses.dataclass
class dataclass5:
    a: 'dataclass6'
    b: int = 1

@dataclasses.dataclass
class dataclass6:
    c: 'dataclass5'
    d: int = 1

class Unorderable:

    def __repr__(self):
        return str(id(self))

class Orderable:

    def __init__(self, hash):
        self._hash = hash

    def __lt__(self, other):
        return False

    def __gt__(self, other):
        return self != other

    def __le__(self, other):
        return self == other

    def __ge__(self, other):
        return True

    def __eq__(self, other):
        return self is other

    def __ne__(self, other):
        return self is not other

    def __hash__(self):
        return self._hash

class DottedPrettyPrinter(pprint.PrettyPrinter):

    def format(self, object, context, maxlevels, level):
        if isinstance(object, str):
            if ' ' in object:
                return (repr(object), 1, 0)
            else:
                return (object, 0, 0)
        else:
            return pprint.PrettyPrinter.format(self, object, context, maxlevels, level)


# --- test body ---
self_a = list(range(100))
self_b = list(range(200))
self_a[-12] = self_b

assert pprint.pformat(set()) == 'set()'

assert pprint.pformat(set(range(3))) == '{0, 1, 2}'

assert pprint.pformat(set(range(7)), width=20) == '{0,\n 1,\n 2,\n 3,\n 4,\n 5,\n 6}'

assert pprint.pformat(set2(range(7)), width=20) == 'set2({0,\n      1,\n      2,\n      3,\n      4,\n      5,\n      6})'

assert pprint.pformat(set3(range(7)), width=20) == 'set3({0, 1, 2, 3, 4, 5, 6})'

assert pprint.pformat(frozenset()) == 'frozenset()'

assert pprint.pformat(frozenset(range(3))) == 'frozenset({0, 1, 2})'

assert pprint.pformat(frozenset(range(7)), width=20) == 'frozenset({0,\n           1,\n           2,\n           3,\n           4,\n           5,\n           6})'

assert pprint.pformat(frozenset2(range(7)), width=20) == 'frozenset2({0,\n            1,\n            2,\n            3,\n            4,\n            5,\n            6})'

assert pprint.pformat(frozenset3(range(7)), width=20) == 'frozenset3({0, 1, 2, 3, 4, 5, 6})'
print("QueryTestCase::test_set_reprs: ok")
