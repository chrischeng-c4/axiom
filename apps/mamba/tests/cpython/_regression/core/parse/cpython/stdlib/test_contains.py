# RUN: parse
# Extracted from CPython Lib/test/test_contains.py — syntax constructs only.
from collections import deque


class base_set:
    def __init__(self, el):
        self.el = el

class myset(base_set):
    def __contains__(self, el):
        return self.el == el

class seq(base_set):
    def __getitem__(self, n):
        return [self.el][n]


class Deviant1:
    """Behaves strangely when compared"""
    aList = list(range(15))
    def __eq__(self, other):
        if other == 12:
            self.aList.remove(12)
            self.aList.remove(13)
            self.aList.remove(14)
        return 0


class ByContains(object):
    def __contains__(self, other):
        return False

class BlockContains(ByContains):
    """Blocks __contains__ fallback to iteration.

    This class is a perfectly good iterable, as well as inheriting
    from a perfectly good container, but __contains__ = None prevents
    the usual fallback to iteration in the container protocol.
    """
    def __iter__(self):
        while False:
            yield None
    __contains__ = None


# Membership expressions
a = base_set(1)
b = myset(1)
c = seq(1)
1 in b
0 not in b
1 in c
0 not in c

'c' in 'abc'
'd' not in 'abc'
'' in ''
'' in 'abc'

a = range(10)
for i in a:
    i in a

a = tuple(a)
for i in a:
    i in a

values = float('nan'), 1, None, 'abc'
constructors = list, tuple, dict.fromkeys, set, frozenset, deque
for constructor in constructors:
    container = constructor(values)
    for elem in container:
        elem in container

bc = BlockContains()
0 in list(bc)
