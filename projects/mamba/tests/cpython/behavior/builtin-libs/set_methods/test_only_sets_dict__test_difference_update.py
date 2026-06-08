# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "set_methods"
# dimension = "behavior"
# case = "test_only_sets_dict__test_difference_update"
# subject = "cpython.test_set.TestOnlySetsDict.test_difference_update"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_set.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_set.py::TestOnlySetsDict::test_difference_update
"""Auto-ported test: TestOnlySetsDict::test_difference_update (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import warnings_helper
import gc
import weakref
import operator
import copy
import pickle
from random import randrange, shuffle
import warnings
import collections
import collections.abc
import itertools
from itertools import chain


class PassThru(Exception):
    pass

def check_pass_thru():
    raise PassThru
    yield 1

class BadCmp:

    def __hash__(self):
        return 1

    def __eq__(self, other):
        raise RuntimeError

class ReprWrapper:
    """Used to test self-referential repr() calls"""

    def __repr__(self):
        return repr(self.value)

class HashCountingInt(int):
    """int-like object that counts the number of times __hash__ is called"""

    def __init__(self, *args):
        self.hash_count = 0

    def __hash__(self):
        self.hash_count += 1
        return int.__hash__(self)

class SetSubclass(set):
    pass

class FrozenSetSubclass(frozenset):
    pass

class SetSubclassWithSlots(set):
    __slots__ = ('x', 'y', '__dict__')

class FrozenSetSubclassWithSlots(frozenset):
    __slots__ = ('x', 'y', '__dict__')

empty_set = set()

def baditer():
    raise TypeError
    yield True

def gooditer():
    yield True

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

class bad_eq:

    def __eq__(self, other):
        if be_bad:
            set2.clear()
            raise ZeroDivisionError
        return self is other

    def __hash__(self):
        return 0

class bad_dict_clear:

    def __eq__(self, other):
        if be_bad:
            dict2.clear()
        return self is other

    def __hash__(self):
        return 0

def powerset(U):
    """Generates all subsets of a set or sequence U."""
    U = iter(U)
    try:
        x = frozenset([next(U)])
        for S in powerset(U):
            yield S
            yield (S | x)
    except StopIteration:
        yield frozenset()

def cube(n):
    """Graph of n-dimensional hypercube."""
    singletons = [frozenset([x]) for x in range(n)]
    return dict([(x, frozenset([x ^ s for s in singletons])) for x in powerset(range(n))])

def linegraph(G):
    """Graph, the vertices of which are edges of G,
    with two vertices being adjacent iff the corresponding
    edges share a vertex."""
    L = {}
    for x in G:
        for y in G[x]:
            nx = [frozenset([x, z]) for z in G[x] if z != y]
            ny = [frozenset([y, z]) for z in G[y] if z != x]
            L[frozenset([x, y])] = frozenset(nx + ny)
    return L

def faces(G):
    """Return a set of faces in G.  Where a face is a set of vertices on that face"""
    f = set()
    for v1, edges in G.items():
        for v2 in edges:
            for v3 in G[v2]:
                if v1 == v3:
                    continue
                if v1 in G[v3]:
                    f.add(frozenset([v1, v2, v3]))
                else:
                    for v4 in G[v3]:
                        if v4 == v2:
                            continue
                        if v1 in G[v4]:
                            f.add(frozenset([v1, v2, v3, v4]))
                        else:
                            for v5 in G[v4]:
                                if v5 == v3 or v5 == v2:
                                    continue
                                if v1 in G[v5]:
                                    f.add(frozenset([v1, v2, v3, v4, v5]))
    return f


# --- test body ---
self_set = set((1, 2, 3))
self_other = {1: 2, 3: 4}
self_otherIsIterable = True
if self_otherIsIterable:
    self_set.difference_update(self_other)
else:

    try:
        self_set.difference_update(self_other)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestOnlySetsDict::test_difference_update: ok")
