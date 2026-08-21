# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "slots_shallow_shares_deep_copies"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: a __slots__ instance shares its slot value under shallow copy and gets an independent slot value under deepcopy"""
import copy


class Slotted:
    __slots__ = ["foo"]


sl = Slotted()
sl.foo = [42]
assert copy.copy(sl).foo is sl.foo, "slots shallow shares the slot value"
sl_d = copy.deepcopy(sl)
assert sl_d.foo == sl.foo and sl_d.foo is not sl.foo, "slots deepcopy copies the slot value"

print("slots_shallow_shares_deep_copies OK")
