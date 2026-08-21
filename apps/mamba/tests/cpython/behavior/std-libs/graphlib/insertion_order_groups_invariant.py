# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "insertion_order_groups_invariant"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: the order edges are added in does not change the grouping produced by the parallel prepare/get_ready/done driver loop"""
import graphlib


def groups(ts):
    ts.prepare()
    out = []
    while ts.is_active():
        ready = ts.get_ready()
        ts.done(*ready)
        out.append(set(ready))
    return out


a = graphlib.TopologicalSorter()
a.add(3, 2, 1)
a.add(1, 0)
a.add(4, 5)
a.add(6, 7)
a.add(4, 7)

b = graphlib.TopologicalSorter()
b.add(1, 0)
b.add(3, 2, 1)
b.add(4, 7)
b.add(6, 7)
b.add(4, 5)

assert groups(a) == groups(b), "insertion order does not change groups"

print("insertion_order_groups_invariant OK")
