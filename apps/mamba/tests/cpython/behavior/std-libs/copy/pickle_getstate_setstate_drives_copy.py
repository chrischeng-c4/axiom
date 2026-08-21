# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "pickle_getstate_setstate_drives_copy"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: with no __copy__/__deepcopy__, copy falls back to the pickle protocol so __getstate__/__setstate__ drive the copied state and deepcopy makes it independent"""
import copy


# Plain instance: shallow keeps the same state object, deepcopy rebuilds it.
class Vanilla:
    def __init__(self, foo):
        self.foo = foo

    def __eq__(self, other):
        return self.foo == other.foo


v = Vanilla([42])
assert copy.copy(v) == v, "vanilla shallow equal"
vd = copy.deepcopy(v)
assert vd == v and vd.foo is not v.foo, "vanilla deepcopy independent state"


# __getstate__ + __setstate__ drive the copied state.
class StatePair:
    def __init__(self, foo):
        self.foo = foo

    def __getstate__(self):
        return self.foo

    def __setstate__(self, state):
        self.foo = state

    def __eq__(self, other):
        return self.foo == other.foo


sp = StatePair([42])
spd = copy.deepcopy(sp)
assert spd == sp and spd.foo is not sp.foo, "getstate/setstate deepcopy independent"

print("pickle_getstate_setstate_drives_copy OK")
