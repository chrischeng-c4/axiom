# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_set_mixins_provide_algebra"
# subject = "collections.abc.Set"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.Set: a Set subclass implementing __contains__/__iter__/__len__ gets &, |, -, ^, ordering, equality, and isdisjoint mixed in, returning the same Set subclass via _from_iterable"""
from collections.abc import Set

class MySet(Set):
    def __init__(self, items):
        self._items = set(items)

    def __contains__(self, x):
        return x in self._items

    def __iter__(self):
        return iter(self._items)

    def __len__(self):
        return len(self._items)


s1 = MySet((1, 2, 3))
s2 = MySet((3, 4, 5))
assert (s1 & s2) == MySet((3,)), "intersection"
assert set(s1 | s2) == {1, 2, 3, 4, 5}, "union"
assert set(s1 - s2) == {1, 2}, "difference"
assert set(s1 ^ s2) == {1, 2, 4, 5}, "symmetric difference"
assert isinstance(s1 | s2, MySet), "binary op returns the same Set subclass via _from_iterable"
assert MySet((1,)) < MySet((1, 2)) and MySet((1, 2)) > MySet((1,)), "proper subset/superset"
assert not (MySet((1, 2)) <= MySet((1,))), "not subset"
assert s1.isdisjoint(MySet((4, 5, 6))) and not s1.isdisjoint(MySet((1, 9))), "isdisjoint"

print("abc_set_mixins_provide_algebra OK")
