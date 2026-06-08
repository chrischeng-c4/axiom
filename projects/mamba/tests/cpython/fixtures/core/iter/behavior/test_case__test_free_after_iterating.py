# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_free_after_iterating"
# subject = "cpython.test_iter.TestCase.test_free_after_iterating"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestCase::test_free_after_iterating (CPython 3.12 oracle)."""

import gc


class SequenceClass:
    def __init__(self, n):
        self.n = n

    def __getitem__(self, index):
        if 0 <= index < self.n:
            return index
        raise IndexError


freed = []
iterator_box = []


class FreeAfterIteratingSequence(SequenceClass):
    def __del__(self):
        freed.append(True)
        try:
            next(iterator_box[0])
        except StopIteration:
            pass


iterator = iter(FreeAfterIteratingSequence(0))
iterator_box.append(iterator)

try:
    next(iterator)
except StopIteration:
    pass
else:
    raise AssertionError("empty sequence iterator should be exhausted")

for _ in range(3):
    gc.collect()

assert freed == [True], freed

print("TestCase::test_free_after_iterating: ok")
