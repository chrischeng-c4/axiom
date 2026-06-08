# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "shared_object_memo"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: a structure that references the same list object twice survives a dumps+loads round-trip: both positions reconstruct equal to the shared value"""
import pickle

shared = [1, 2, 3]
container = [shared, shared]
rt = pickle.loads(pickle.dumps(container))
# Both positions must reconstruct equal to the shared value; CPython's memo
# additionally makes them identical, but equality is the portable contract.
assert rt[0] == [1, 2, 3], f"shared[0] = {rt[0]!r}"
assert rt[1] == [1, 2, 3], f"shared[1] = {rt[1]!r}"
assert rt[0] is rt[1], "memo preserves shared identity across both positions"

print("shared_object_memo OK")
