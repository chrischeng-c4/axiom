# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_iterator_protocol"
# subject = "collections.abc.Iterator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Iterator: a class defining __iter__ and __next__ is an Iterator and iterates to exhaustion via list()"""
import collections.abc as abc


class MyIterator:
    def __init__(self):
        self._data = [10, 20, 30]
        self._idx = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self._idx >= len(self._data):
            raise StopIteration
        v = self._data[self._idx]
        self._idx += 1
        return v


it = MyIterator()
assert isinstance(it, abc.Iterator), "custom Iterator"
assert isinstance(it, abc.Iterable), "Iterator is Iterable"
assert list(it) == [10, 20, 30], "Iterator iteration"
print("custom_iterator_protocol OK")
