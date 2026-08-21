# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_deque_reversed__test_invariant"
# subject = "cpython.test_iterlen.TestDequeReversed.test_invariant"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iterlen.py::TestDequeReversed::test_invariant
"""Auto-ported test: TestDequeReversed::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
d = deque(range(n))
self_it = reversed(d)
self_mutate = d.pop
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestDequeReversed::test_invariant: ok")
