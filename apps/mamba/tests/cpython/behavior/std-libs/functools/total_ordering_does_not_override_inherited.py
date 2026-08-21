# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "total_ordering_does_not_override_inherited"
# subject = "functools.total_ordering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: total_ordering does not overwrite rich comparisons inherited from a base (an int subclass keeps int's ordering)"""
import functools


# total_ordering does not overwrite ordering ops already inherited from a
# base; subclassing int keeps int's own rich comparisons.
@functools.total_ordering
class MyInt(int):
    pass


assert MyInt(1) < MyInt(2), "int subclass lt"
assert MyInt(2) > MyInt(1), "int subclass gt"
assert MyInt(2) >= MyInt(2), "int subclass ge equal"
assert MyInt(1) <= MyInt(1), "int subclass le equal"

# The inherited comparison methods are int's, not synthesized stand-ins.
assert MyInt.__lt__ is int.__lt__, "lt not overwritten"
assert MyInt.__gt__ is int.__gt__, "gt not overwritten"

print("total_ordering_does_not_override_inherited OK")
