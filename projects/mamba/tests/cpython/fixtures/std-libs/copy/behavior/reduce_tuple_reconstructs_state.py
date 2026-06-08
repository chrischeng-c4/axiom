# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "reduce_tuple_reconstructs_state"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: a __reduce__ 3-tuple (callable, args, state) reconstructs the instance, applying the state dict for copy and an independent copy of it for deepcopy"""
import copy


# __reduce__ returning (callable, args, state): copy applies the state dict,
# deepcopy applies an independent copy of it.
class Reconstruct:
    def __reduce__(self):
        return (Reconstruct, (), self.__dict__)

    def __eq__(self, other):
        return self.__dict__ == other.__dict__


r = Reconstruct()
r.foo = [42]
assert copy.copy(r) == r, "reduce 3-tuple shallow equal"
rd = copy.deepcopy(r)
assert rd == r and rd.foo is not r.foo, "reduce 3-tuple deepcopy independent state"


# A __reduce__ 2-tuple (no state) still reconstructs the class; an attribute set
# after construction is not reproduced.
class NoState:
    def __reduce__(self):
        return (NoState, ())


ns = NoState()
ns.foo = 42
assert copy.copy(ns).__class__ is ns.__class__, "reduce 2-tuple copy class"
assert copy.deepcopy(ns).__class__ is ns.__class__, "reduce 2-tuple deepcopy class"

print("reduce_tuple_reconstructs_state OK")
