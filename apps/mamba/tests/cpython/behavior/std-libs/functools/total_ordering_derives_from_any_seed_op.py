# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "total_ordering_derives_from_any_seed_op"
# subject = "functools.total_ordering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: total_ordering seeded from __lt__ or __ge__ derives the other three ordering operators consistently, including equal-value boundaries"""
import functools


# total_ordering can seed from any one ordering op; whichever is provided,
# the other three are derived consistently.
@functools.total_ordering
class FromLt:
    def __init__(self, v):
        self.v = v

    def __eq__(self, other):
        return self.v == other.v

    def __lt__(self, other):
        return self.v < other.v


@functools.total_ordering
class FromGe:
    def __init__(self, v):
        self.v = v

    def __eq__(self, other):
        return self.v == other.v

    def __ge__(self, other):
        return self.v >= other.v


for cls in (FromLt, FromGe):
    lo, hi = cls(1), cls(2)
    assert lo < hi, f"{cls.__name__}: lt"
    assert lo <= hi, f"{cls.__name__}: le"
    assert hi > lo, f"{cls.__name__}: gt"
    assert hi >= lo, f"{cls.__name__}: ge"
    assert lo <= cls(1), f"{cls.__name__}: le equal"
    assert hi >= cls(2), f"{cls.__name__}: ge equal"
    assert not (lo < cls(1)), f"{cls.__name__}: lt equal is False"
    assert not (hi > cls(2)), f"{cls.__name__}: gt equal is False"

print("total_ordering_derives_from_any_seed_op OK")
