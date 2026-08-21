# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_is_stable_on_ties"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() is stable: records with equal keys keep their original per-stream input order within each key band"""
import heapq

# Tag each record with (key, source) so we can detect reordering of ties.
streams = [
    [(0, "a0"), (0, "a1"), (1, "a2")],
    [(0, "b0"), (1, "b1"), (1, "b2")],
    [(0, "c0"), (2, "c1")],
]
stable = list(heapq.merge(*streams, key=lambda r: r[0]))
# All key==0 records must precede key==1, which precede key==2, and within
# each key band the original per-stream order is preserved.
keys_only = [k for k, _ in stable]
assert keys_only == sorted(keys_only), f"merge not sorted by key = {keys_only!r}"
zero_band = [tag for k, tag in stable if k == 0]
assert zero_band == ["a0", "a1", "b0", "c0"], f"tie order lost = {zero_band!r}"
print("merge_is_stable_on_ties OK")
