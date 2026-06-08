# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "deep_nested_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a deeply nested dict-of-dict-of-list-of-dict round-trips through dumps+loads equal to the original"""
import pickle

nested = {"a": {"b": {"c": [1, 2, {"d": 3}]}}}
rt = pickle.loads(pickle.dumps(nested))
assert rt == nested, f"deep nested round-trip = {rt!r}"

print("deep_nested_roundtrip OK")
