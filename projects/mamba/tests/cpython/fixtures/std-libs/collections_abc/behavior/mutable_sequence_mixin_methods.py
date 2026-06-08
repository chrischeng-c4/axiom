# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mutable_sequence_mixin_methods"
# subject = "collections.abc.MutableSequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableSequence: subclassing MutableSequence with the five abstract methods yields working mixin methods append() and reverse()"""
import collections.abc as abc


class MyMutableSeq(abc.MutableSequence):
    def __init__(self):
        self._data = []

    def __getitem__(self, i):
        return self._data[i]

    def __setitem__(self, i, v):
        self._data[i] = v

    def __delitem__(self, i):
        del self._data[i]

    def __len__(self):
        return len(self._data)

    def insert(self, i, v):
        self._data.insert(i, v)


ms = MyMutableSeq()
# append() is a mixin method provided by MutableSequence.
ms.append(10)
ms.append(20)
assert len(ms) == 2, f"MutableSeq len = {len(ms)!r}"
assert ms[0] == 10, f"MutableSeq[0] = {ms[0]!r}"
# reverse() is a mixin method provided by MutableSequence.
ms.reverse()
assert ms[0] == 20, f"after reverse[0] = {ms[0]!r}"
print("mutable_sequence_mixin_methods OK")
