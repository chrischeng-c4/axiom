# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_mutablesequence_mixins"
# subject = "collections.abc.MutableSequence"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.MutableSequence: a MutableSequence subclass implementing the abstract methods gets append/extend/reverse/pop/remove/+= mixed in over a backing list"""
from collections.abc import MutableSequence

class Seq(MutableSequence):
    def __init__(self):
        self._lst = []

    def __getitem__(self, i):
        return self._lst[i]

    def __setitem__(self, i, v):
        self._lst[i] = v

    def __delitem__(self, i):
        del self._lst[i]

    def __len__(self):
        return len(self._lst)

    def insert(self, i, v):
        self._lst.insert(i, v)


seq = Seq()
seq.append(0)
seq.extend((1, 2, 3, 4))
assert len(seq) == 5 and seq[3] == 3, "append/extend mixins"
seq.reverse()
assert seq[0] == 4, "reverse mixin"
seq.pop()
seq.remove(3)
assert list(seq) == [4, 2, 1], f"pop+remove = {list(seq)!r}"
seq += (10, 20)
assert seq[-1] == 20, "+= mixin"

print("abc_mutablesequence_mixins OK")
