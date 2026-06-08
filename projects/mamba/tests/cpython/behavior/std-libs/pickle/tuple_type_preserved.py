# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "tuple_type_preserved"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a tuple round-trips as a tuple (not a list): pickle.loads(pickle.dumps((1,'a',True))) is a tuple and equals the original"""
import pickle

t = (1, "a", True)
rt = pickle.loads(pickle.dumps(t))
assert isinstance(rt, tuple), f"tuple preserved = {type(rt)!r}"
assert rt == t, f"tuple equality = {rt!r}"

print("tuple_type_preserved OK")
