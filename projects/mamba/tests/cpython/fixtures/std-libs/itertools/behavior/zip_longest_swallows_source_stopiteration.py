# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "zip_longest_swallows_source_stopiteration"
# subject = "itertools.zip_longest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest treats a source's StopIteration as exhaustion and pads with fillvalue (bug 7244)"""
import itertools

class Repeater:
    """Yields `o` exactly `t` times, then raises `e`."""

    def __init__(self, o, t, e):
        self.o = o
        self.t = t
        self.e = e

    def __iter__(self):
        return self

    def __next__(self):
        if self.t > 0:
            self.t -= 1
            return self.o
        raise self.e

r1 = Repeater(1, 3, StopIteration)
r2 = Repeater(2, 4, StopIteration)
got = list(itertools.zip_longest(r1, r2, fillvalue=0))
assert got == [(1, 2), (1, 2), (1, 2), (0, 2)], f"zip_longest fill = {got!r}"

print("zip_longest_swallows_source_stopiteration OK")
