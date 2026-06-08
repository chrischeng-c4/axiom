# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "total_ordering_propagates_notimplemented"
# subject = "functools.total_ordering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: when the seed comparison returns NotImplemented, the derived operators also return NotImplemented rather than guessing"""
import functools


# When the seed op returns NotImplemented (here for a foreign type), the
# derived ops propagate NotImplemented instead of guessing a result.
@functools.total_ordering
class OnlyLt:
    def __init__(self, v):
        self.v = v

    def __eq__(self, other):
        return isinstance(other, OnlyLt) and self.v == other.v

    def __lt__(self, other):
        if isinstance(other, OnlyLt):
            return self.v < other.v
        return NotImplemented


obj = OnlyLt(1)
assert obj.__le__(1) is NotImplemented, "le passthrough NotImplemented"
assert obj.__gt__(1) is NotImplemented, "gt passthrough NotImplemented"
assert obj.__ge__(1) is NotImplemented, "ge passthrough NotImplemented"

# For same-type operands the derived ops still produce real results.
assert obj < OnlyLt(2), "lt same type"
assert obj <= OnlyLt(1), "le same type equal"

print("total_ordering_propagates_notimplemented OK")
