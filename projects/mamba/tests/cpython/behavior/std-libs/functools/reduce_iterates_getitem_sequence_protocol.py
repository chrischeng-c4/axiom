# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "reduce_iterates_getitem_sequence_protocol"
# subject = "functools.reduce"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: reduce drives an arbitrary __getitem__ sequence (IndexError stops), with and without an initial, including the empty custom sequence"""
import functools


class Squares:
    """A sequence-protocol object: __getitem__ raising IndexError to stop."""

    def __init__(self, count):
        self.count = count
        self.sofar = []

    def __getitem__(self, i):
        if not 0 <= i < self.count:
            raise IndexError
        while len(self.sofar) <= i:
            n = len(self.sofar)
            self.sofar.append(n * n)
        return self.sofar[i]


def _add(x, y):
    return x + y


# reduce iterates via __getitem__, not just over built-in lists.
assert functools.reduce(_add, Squares(10)) == 285, "reduce over custom seq"
assert functools.reduce(_add, Squares(10), 0) == 285, "reduce custom seq + initial"
assert functools.reduce(_add, Squares(0), 0) == 0, "reduce empty custom seq"

print("reduce_iterates_getitem_sequence_protocol OK")
