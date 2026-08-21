# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mutable_sequence_mixin_completeness"
# subject = "collections.abc.MutableSequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableSequence: nominal subclasses inherit the full mutable sequence mixin surface"""
import collections.abc as abc


class MyMutableSeq(abc.MutableSequence):
    def __init__(self, values=()):
        self.items = list(values)

    def __getitem__(self, index):
        return self.items[index]

    def __setitem__(self, index, value):
        self.items[index] = value

    def __delitem__(self, index):
        del self.items[index]

    def __len__(self):
        return len(self.items)

    def insert(self, index, value):
        self.items.insert(index, value)


seq = MyMutableSeq([1, 2])

seq.extend([3, 4])
assert seq.items == [1, 2, 3, 4], "extend appends every item through insert"
assert list(seq) == [1, 2, 3, 4], "__iter__ yields indexed values"
assert 3 in seq, "__contains__ finds present values"
assert 9 not in seq, "__contains__ rejects missing values"
assert seq.index(3) == 2, "index returns the first matching position"

assert seq.pop() == 4, "pop defaults to the last item"
assert seq.items == [1, 2, 3], "pop removes the default item"
assert seq.pop(0) == 1, "pop accepts an explicit index"
assert seq.items == [2, 3], "indexed pop removes from that position"

seq.remove(2)
assert seq.items == [3], "remove deletes the first matching value"

same = seq
seq += (5, 6)
assert seq is same, "__iadd__ returns self"
assert seq.items == [3, 5, 6], "__iadd__ extends from an iterable"

seq.reverse()
assert seq.items == [6, 5, 3], "reverse remains available with the complete mixin set"
seq.append(7)
assert seq.items == [6, 5, 3, 7], "append remains available with the complete mixin set"

try:
    seq.index(99)
    raise AssertionError("index should reject missing values")
except ValueError:
    pass


class ReadOnlySeq(abc.Sequence):
    def __init__(self, values):
        self.values = list(values)

    def __getitem__(self, index):
        return self.values[index]

    def __len__(self):
        return len(self.values)


readonly = ReadOnlySeq([1, 2, 3])
assert list(readonly) == [1, 2, 3], "unrelated Sequence protocol still iterates"
assert not hasattr(readonly, "append"), "Sequence subclass does not gain MutableSequence append"
assert not hasattr(readonly, "remove"), "Sequence subclass does not gain MutableSequence remove"

native = [1]
native.extend([2])
native += [3]
assert native.pop() == 3, "native list pop behavior remains intact"
native.remove(1)
assert native == [2], "native list remove behavior remains intact"

print("mutable_sequence_mixin_completeness OK")
