# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "frozenset_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = "pickle shim has no set/frozenset serialization branch; sets serialize to the 'N' sentinel (src/runtime/stdlib/pickle_mod.rs:220)"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a frozenset round-trips through dumps+loads equal to the original frozenset"""
import pickle

fs = frozenset([1, 2, 3])
rt = pickle.loads(pickle.dumps(fs))
assert rt == fs, f"frozenset round-trip = {rt!r}"
assert isinstance(rt, frozenset), f"frozenset type preserved = {type(rt)!r}"

print("frozenset_roundtrip OK")
