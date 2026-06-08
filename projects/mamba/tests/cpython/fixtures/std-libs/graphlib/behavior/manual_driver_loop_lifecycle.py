# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "manual_driver_loop_lifecycle"
# subject = "graphlib.TopologicalSorter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
"""graphlib.TopologicalSorter: prepare/get_ready/done/is_active drive parallel scheduling: leaves come out first as a tuple, done() unblocks dependents, is_active stays True until every released node is done"""
import graphlib

# Node 1 depends on 2, 3, 4; node 2 also depends on 3.
ts = graphlib.TopologicalSorter()
ts.add(1, 2, 3, 4)
ts.add(2, 3)
ts.prepare()

# Leaves with no outstanding deps come out first, as a tuple.
first = ts.get_ready()
assert set(first) == {3, 4}, sorted(first)

# Nothing new is ready until we mark progress.
assert ts.get_ready() == (), "no new nodes before done"

# Completing 3 unblocks 2 (4 still outstanding for 1).
ts.done(3)
assert ts.get_ready() == (2,), "ready after done(3)"
assert ts.get_ready() == (), "drained again"

# Finish 4 and 2; that unblocks 1.
ts.done(4)
ts.done(2)
assert ts.get_ready() == (1,), "ready after done(4) and done(2)"
assert ts.get_ready() == (), "drained again"

# is_active stays True until every released node is marked done.
assert ts.is_active() is True, "still active before final done"
ts.done(1)
assert ts.get_ready() == (), "nothing left"
assert ts.is_active() is False, "inactive once fully drained"

print("manual_driver_loop_lifecycle OK")
