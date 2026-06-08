# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "custom_reduce_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim does not consult __reduce__ and serializes Instance objects to the 'N' sentinel (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a class defining __reduce__ controls its own reconstruction: the round-tripped instance equals the original via the reduce-provided constructor args"""
import pickle


class Custom:
    def __init__(self, val):
        self.val = val

    def __reduce__(self):
        return (Custom, (self.val,))

    def __eq__(self, other):
        return isinstance(other, Custom) and self.val == other.val


c = Custom(99)
rt = pickle.loads(pickle.dumps(c))
assert rt == c, f"custom __reduce__ round-trip = {rt.val!r}"

print("custom_reduce_roundtrip OK")
