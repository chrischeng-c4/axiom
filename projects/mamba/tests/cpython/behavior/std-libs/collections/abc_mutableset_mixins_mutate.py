# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_mutableset_mixins_mutate"
# subject = "collections.abc.MutableSet"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.MutableSet: a MutableSet subclass adding add/discard gets pop and the in-place operators ^=, |=, -= mixed in, mutating and returning the same object"""
from collections.abc import MutableSet

class MutableSetSubclass(MutableSet):
    def __init__(self, items=None):
        self._s = set(items or [])

    def __contains__(self, v):
        return v in self._s

    def __iter__(self):
        return iter(self._s)

    def __len__(self):
        return len(self._s)

    def add(self, v):
        self._s.add(v)

    def discard(self, v):
        self._s.discard(v)


m = MutableSetSubclass([5, 43, 2, 1])
popped = m.pop()
assert len(m) == 3 and popped not in m and popped in {5, 43, 2, 1}, "pop returns and removes a member"

m2 = MutableSetSubclass([1, 2, 3])
m2 ^= [3, 4]
assert set(m2) == {1, 2, 4}, "ixor"
m2 |= [10]
assert 10 in m2, "ior"
m2 -= [1]
assert 1 not in m2, "isub"

print("abc_mutableset_mixins_mutate OK")
