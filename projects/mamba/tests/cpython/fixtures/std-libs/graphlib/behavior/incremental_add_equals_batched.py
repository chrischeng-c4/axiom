# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "incremental_add_equals_batched"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: repeated add(1,dep) calls accumulate to the same static_order as a single add of the dep set {2,3,4,5}"""
import graphlib

incremental = graphlib.TopologicalSorter()
incremental.add(1, 2)
incremental.add(1, 3)
incremental.add(1, 4)
incremental.add(1, 5)
batched = graphlib.TopologicalSorter({1: {2, 3, 4, 5}})
incremental_order = list(incremental.static_order())
assert incremental_order == list(batched.static_order()), incremental_order

print("incremental_add_equals_batched OK")
